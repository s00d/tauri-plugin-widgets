#!/usr/bin/env bash
# Build (or delete) the UTM Windows 11 ARM64 test VM end to end, no browser and no clicks:
#
#   scripts/sh/win-vm.sh create    # ISO (msdl) + guest tools + unattended install + SSH + .NET 8
#   scripts/sh/win-vm.sh destroy   # stop and delete the VM and the downloaded ISOs
#
# Then the existing recipes work: just win-up / win-sync / record-windows <case>.
#
# Needs: UTM (utmctl, AppleScript), msdl (brew; or WIN_ISO=<path>), ~/.ssh/utm_win(.pub),
# and `Host utm-win` -> 127.0.0.1:2222 user pavel in ~/.ssh/config.
# Disk: a sparse 24 GiB NVMe qcow2 (setup's storage check is bypassed) — the host file
# holds only what Windows writes (~14 GiB), plus ~6 GiB ISO until `destroy`.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIR="${WIN_VM_DIR:-$HOME/vm/tauri-widgets-win}"
VM="${UTM_WIN_VM:-Windows}"
HOST="${UTM_WIN_HOST:-utm-win}"
KEY="${UTM_WIN_KEY:-$HOME/.ssh/utm_win}"
DISK_MIB="${WIN_VM_DISK_MIB:-24576}"
RAM_MIB="${WIN_VM_RAM_MIB:-4096}"
CPUS="${WIN_VM_CPUS:-4}"
# Windows 11 Arm64 multi-edition ISO in msdl / microsoft.com terms.
MSDL_ID="${WIN_MSDL_ID:-3816}"
BUNDLE="$HOME/Library/Containers/com.utmapp.UTM/Data/Documents/${VM}.utm"

log() { echo "win-vm: $*"; }
vm_exists() { utmctl list 2>/dev/null | awk 'NR>1{ $1=""; $2=""; sub(/^  /,""); print }' | grep -qx "$VM"; }
ssh_vm() { ssh -o BatchMode=yes -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new "$HOST" "$@"; }

create() {
  command -v utmctl >/dev/null || { log "utmctl missing (install UTM)"; exit 1; }
  [[ -f "$KEY.pub" ]] || { log "missing $KEY.pub (ssh-keygen -t ed25519 -f $KEY)"; exit 1; }
  if vm_exists; then log "VM '$VM' already exists — run destroy first"; exit 1; fi
  mkdir -p "$DIR"

  local iso="${WIN_ISO:-$DIR/Win11_arm64_en-us.iso}"
  if [[ ! -s "$iso" ]]; then
    command -v msdl >/dev/null || { log "msdl missing (brew install msdl) or set WIN_ISO=<path>"; exit 1; }
    log "resolving the Windows 11 Arm64 ISO link (msdl --id $MSDL_ID)"
    local url
    url="$(MSDL_NO_TELEMETRY=1 msdl --id "$MSDL_ID" --lang "English" --no-contribute | grep -Eo 'https://[^ ]+\.iso[^ ]*' | head -1)"
    [[ -n "$url" ]] || { log "msdl printed no ISO link"; exit 1; }
    log "downloading ISO (~6 GiB) -> $iso"
    curl -fL --retry 5 -C - -o "$iso" "$url"
  fi

  local tools="$DIR/utm-guest-tools.iso"
  if [[ ! -s "$tools" ]]; then
    log "downloading UTM guest tools"
    curl -fL --retry 5 -o "$tools" https://getutm.app/downloads/utm-guest-tools-latest.iso
  fi

  # Everything the guest needs is downloaded here, so first logon never waits on the web or
  # Windows Update: .NET 8 SDK zip (Microsoft releases.json, SHA-512) and the Win32-OpenSSH
  # ARM64 MSI (GitHub release, SHA-256).
  local cache="$DIR/cache"
  mkdir -p "$cache"
  if ! ls "$cache"/dotnet-sdk-*-win-arm64.zip >/dev/null 2>&1; then
    log "downloading .NET 8 SDK (win-arm64)"
    local meta url sha name
    meta="$(curl -fsSL https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/8.0/releases.json)"
    read -r url sha < <(python3 -c 'import json,sys
r=json.loads(sys.stdin.read())["releases"][0]["sdk"]
f=[f for f in r["files"] if f.get("rid")=="win-arm64" and f["name"].endswith(".zip")][0]
print(f["url"], f["hash"])' <<<"$meta")
    name="$(basename "$url")"
    curl -fL --retry 5 -o "$cache/$name.part" "$url"
    [[ "$(shasum -a 512 "$cache/$name.part" | cut -d' ' -f1)" == "$sha" ]] || { log "SHA-512 mismatch: $name"; exit 1; }
    mv "$cache/$name.part" "$cache/$name"
  fi
  if ! ls "$cache"/OpenSSH-*.msi >/dev/null 2>&1; then
    log "downloading Win32-OpenSSH (ARM64 MSI)"
    local url sha name
    read -r url sha < <(curl -fsSL https://api.github.com/repos/PowerShell/Win32-OpenSSH/releases/latest | python3 -c 'import json,sys
a=[a for a in json.load(sys.stdin)["assets"] if a["name"].startswith("OpenSSH-ARM64") and a["name"].endswith(".msi")][0]
print(a["browser_download_url"], (a.get("digest") or "").removeprefix("sha256:"))')
    name="$(basename "$url")"
    curl -fL --retry 5 -o "$cache/$name.part" "$url"
    if [[ -n "$sha" ]]; then
      [[ "$(shasum -a 256 "$cache/$name.part" | cut -d' ' -f1)" == "$sha" ]] || { log "SHA-256 mismatch: $name"; exit 1; }
    fi
    mv "$cache/$name.part" "$cache/$name"
  fi

  log "building the unattended-setup ISO"
  local stage="$DIR/unattend"
  rm -rf "$stage" "$DIR/unattend.iso"
  mkdir -p "$stage"
  cp "$ROOT/scripts/win/autounattend.xml" "$ROOT/scripts/win/firstlogon.ps1" "$stage/"
  cp "$KEY.pub" "$stage/authorized_keys"
  cp "$cache"/dotnet-sdk-*-win-arm64.zip "$cache"/OpenSSH-*.msi "$stage/"
  # Only the installer from the guest-tools ISO: that ISO carries its own Autounattend.xml,
  # and Windows Setup must see exactly one (ours).
  local mnt
  mnt="$(hdiutil attach -nobrowse -readonly "$tools" | awk '/\/Volumes/{print substr($0, index($0,"/Volumes"))}')"
  cp "$mnt"/utm-guest-tools-*.exe "$stage/"
  hdiutil detach -quiet "$mnt"
  hdiutil makehybrid -quiet -iso -joliet -default-volume-name UNATTEND -o "$DIR/unattend.iso" "$stage"

  # protocol: raw code of UTM.sdef's "network protocol" TCP enumerator — a bare `TCP` in
  # osascript does not resolve to it and the whole configuration record is rejected.
  log "creating UTM VM '$VM' (${RAM_MIB} MiB RAM, ${CPUS} CPUs, ${DISK_MIB} MiB sparse NVMe)"
  osascript <<EOF
tell application "UTM"
  set winIso to POSIX file "$iso"
  set setupIso to POSIX file "$DIR/unattend.iso"
  make new virtual machine with properties {backend:qemu, configuration:{name:"$VM", architecture:"aarch64", memory:$RAM_MIB, cpu cores:$CPUS, uefi:true, hypervisor:true, drives:{{removable:true, source:winIso}, {removable:true, source:setupIso}, {interface:NVMe, guest size:$DISK_MIB}}, network interfaces:{{mode:emulated, hardware:"virtio-net-pci", port forwards:{{protocol:«constant NtPrTcPp», host address:"127.0.0.1", host port:2222, guest address:"", guest port:22}}}}, displays:{{hardware:"virtio-ramfb"}}}}
end tell
EOF

  # A new VM gets a new host key on the same forwarded port.
  ssh-keygen -R "[127.0.0.1]:2222" >/dev/null 2>&1 || true

  log "starting; pressing a key for the CD's 'Press any key to boot'"
  utmctl start "$VM"
  for _ in $(seq 1 20); do
    osascript -e "tell application \"UTM\" to input keystroke (virtual machine named \"$VM\") text \" \"" >/dev/null 2>&1 || true
    sleep 1
  done

  log "unattended install running (watch it in the UTM window); waiting for SSH (up to 90 min)"
  local deadline=$((SECONDS + 5400))
  until ssh_vm 'Test-Path C:\work\firstlogon.done' 2>/dev/null | grep -q True; do
    (( SECONDS < deadline )) || { log "timeout — see C:\\work\\firstlogon.log in the VM"; exit 1; }
    sleep 30
  done

  log "ready: .NET $(ssh_vm dotnet --version | tr -d '\r')"
  log "host disk: $(du -sh "$BUNDLE" | cut -f1) in $BUNDLE"
  log "next: just win-sync && just record-windows <case>   (or scripts/sh/win-vm.sh destroy)"
}

destroy() {
  if vm_exists; then
    log "stopping and deleting VM '$VM'"
    utmctl stop "$VM" >/dev/null 2>&1 || true
    sleep 3
    osascript -e "tell application \"UTM\" to delete (virtual machine named \"$VM\")"
  else
    log "no VM '$VM'"
  fi
  ssh-keygen -R "[127.0.0.1]:2222" >/dev/null 2>&1 || true
  if [[ -d "$DIR" ]]; then
    log "removing $DIR (ISOs, setup media)"
    rm -rf "$DIR"
  fi
}

case "${1:-}" in
  create) create ;;
  destroy) destroy ;;
  *) echo "usage: $0 create|destroy" >&2; exit 2 ;;
esac
