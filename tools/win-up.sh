#!/usr/bin/env bash
# Healthcheck for the UTM Windows VM via SSH host alias `utm-win`.
# Optional: start the VM with utmctl if the host is down.
set -euo pipefail

HOST="${UTM_WIN_HOST:-utm-win}"
VM_NAME="${UTM_WIN_VM:-Windows}"

ssh_ok() {
  ssh -o BatchMode=yes -o ConnectTimeout=5 "$HOST" "echo ok" >/dev/null 2>&1
}

if ! ssh_ok; then
  if command -v utmctl >/dev/null 2>&1; then
    echo "SSH to $HOST failed — trying utmctl start '$VM_NAME'…"
    utmctl start "$VM_NAME" >/dev/null 2>&1 || true
    for _ in $(seq 1 30); do
      if ssh_ok; then
        break
      fi
      sleep 2
    done
  fi
fi

if ! ssh_ok; then
  echo "ERROR: cannot reach $HOST (ssh). Is the UTM VM running and sshd up?" >&2
  exit 1
fi

echo "==> $HOST reachable"

# Entire remote PowerShell is single-quoted for bash (no local expansion).
ssh "$HOST" powershell -NoProfile -Command \
  'Write-Host ("PSVersion: " + $PSVersionTable.PSVersion.ToString()); try { Write-Host ("dotnet: " + (dotnet --version)) } catch { Write-Host "dotnet: missing" }; try { Write-Host ("git: " + (git --version)) } catch { Write-Host "git: missing" }; $sshd = Get-Service sshd -ErrorAction SilentlyContinue; if ($null -ne $sshd) { Write-Host ("sshd: " + $sshd.Status.ToString() + " / " + $sshd.StartType.ToString()) } else { Write-Host "sshd: not found" }'

echo "win-up: OK"

# Soft signal when MSVC tooling is missing (Phase 4 needs it for full builds).
ssh "$HOST" powershell -NoProfile -ExecutionPolicy Bypass -Command \
  'if (Get-Command link.exe -ErrorAction SilentlyContinue) { Write-Host "link.exe: present" } else { Write-Host "link.exe: MISSING — run: just win-bootstrap" }'

