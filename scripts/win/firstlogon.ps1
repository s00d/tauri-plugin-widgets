#Requires -Version 5.1
<#
.SYNOPSIS
  First logon of the unattended UTM test VM (autounattend.xml, scripts/sh/win-vm.sh).
  Everything comes from the setup ISO the host prepared - no Windows Update, no web:
  guest tools (NIC driver) -> OpenSSH MSI with the host key -> .NET 8 SDK zip (PreviewHost)
  -> limit what Windows writes to disk. Logs to C:\work\firstlogon.log; writes
  C:\work\firstlogon.done at the end (the host waits for it over SSH).
#>
param([string]$Media = "D:")

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path C:\work | Out-Null
Start-Transcript -Path C:\work\firstlogon.log -Append | Out-Null

function Step($msg) { Write-Host ("==> " + (Get-Date -Format HH:mm:ss) + " " + $msg) }
function FromMedia($filter) {
  $f = Get-ChildItem -Path $Media -Filter $filter | Select-Object -First 1
  if ($null -eq $f) { throw "$filter missing on $Media" }
  return $f.FullName
}

# 1. UTM guest tools: VirtIO NIC / display drivers + SPICE agent (silent NSIS installer).
Step "guest tools"
Start-Process -FilePath (FromMedia "utm-guest-tools-*.exe") -ArgumentList "/S" -Wait

# 2. OpenSSH server (Win32-OpenSSH MSI), key-only for the admin account.
Step "OpenSSH server"
Start-Process msiexec.exe -ArgumentList @("/i", ('"' + (FromMedia "OpenSSH-*.msi") + '"'), "/qn", "/norestart") -Wait
Set-Service -Name sshd -StartupType Automatic
Start-Service sshd
$keys = "C:\ProgramData\ssh\administrators_authorized_keys"
Copy-Item (FromMedia "authorized_keys") $keys -Force
icacls $keys /inheritance:r /grant "Administrators:F" /grant "SYSTEM:F" | Out-Null
$cfg = "C:\ProgramData\ssh\sshd_config"
(Get-Content $cfg) -replace '^#?PasswordAuthentication .*', 'PasswordAuthentication no' | Set-Content $cfg
# scripts/win/sync.sh and the recipes speak PowerShell over SSH (OpenSSH defaults to cmd).
New-ItemProperty -Path "HKLM:\SOFTWARE\OpenSSH" -Name DefaultShell -PropertyType String -Force `
  -Value "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" | Out-Null
Restart-Service sshd
if (-not (Get-NetFirewallRule -Name "OpenSSH-Server-In-TCP" -ErrorAction SilentlyContinue)) {
  New-NetFirewallRule -Name "OpenSSH-Server-In-TCP" -DisplayName "OpenSSH Server (sshd)" -Enabled True -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22 | Out-Null
}

# 3. .NET 8 SDK (PreviewHost targets net8.0-windows + WPF), machine-wide PATH.
Step ".NET 8 SDK"
$dn = Join-Path $env:ProgramFiles "dotnet"
Expand-Archive -Path (FromMedia "dotnet-sdk-*-win-arm64.zip") -DestinationPath $dn -Force
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($machinePath -notlike "*$dn*") { [Environment]::SetEnvironmentVariable("Path", ($machinePath + ";" + $dn), "Machine") }
[Environment]::SetEnvironmentVariable("DOTNET_CLI_TELEMETRY_OPTOUT", "1", "Machine")
& (Join-Path $dn "dotnet.exe") --list-sdks

# 4. Stop the disk from growing (cheap, instant): no hibernation file, small page file,
#    no Windows Update downloads. No compaction pass: the host qcow2 is sparse and capped
#    by WIN_VM_DISK_MIB, so a cleanup after install only costs time.
Step "limit disk growth"
powercfg /hibernate off
Get-CimInstance Win32_ComputerSystem | Set-CimInstance -Property @{ AutomaticManagedPagefile = $false }
# Win32_PageFileSetting rejects sizes ("Value out of range", it needs SeCreatePagefilePrivilege);
# its documented backing store is this registry value: "<path> <initial MB> <max MB>", next boot.
Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management" `
  -Name PagingFiles -Type MultiString -Value @("C:\pagefile.sys 512 1024")
Stop-Service wuauserv -Force -ErrorAction SilentlyContinue
Set-Service wuauserv -StartupType Disabled

Step "done"
Set-Content -Path C:\work\firstlogon.done -Value (Get-Date -Format o)
Stop-Transcript | Out-Null
