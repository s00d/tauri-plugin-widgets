#Requires -Version 5.1
<#
.SYNOPSIS
  Install VS Build Tools, Windows SDK pieces, and WinAppSDK tooling on UTM Windows.
#>
param(
  [switch]$SkipWinget,
  [switch]$CheckOnly
)

$ErrorActionPreference = "Stop"

function Test-LinkExe {
  $cmd = Get-Command link.exe -ErrorAction SilentlyContinue
  if ($null -ne $cmd) { return $true }
  $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
  if (Test-Path $vswhere) {
    $install = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if (-not $install) {
      $install = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.ARM64 -property installationPath 2>$null
    }
    if ($install) {
      $candidates = Get-ChildItem -Path $install -Recurse -Filter link.exe -ErrorAction SilentlyContinue |
        Select-Object -First 1
      if ($candidates) { return $true }
    }
  }
  return $false
}

function Write-Status {
  Write-Host "==> bootstrap status"
  Write-Host ("  arch: " + $env:PROCESSOR_ARCHITECTURE)
  Write-Host ("  link.exe: " + (Test-LinkExe))
  try { Write-Host ("  dotnet: " + (dotnet --version)) } catch { Write-Host "  dotnet: missing" }
  $makeappx = Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter makeappx.exe -ErrorAction SilentlyContinue |
    Select-Object -First 1
  Write-Host ("  makeappx: " + ($null -ne $makeappx))
}

Write-Status

if ($CheckOnly) {
  if (Test-LinkExe) { Write-Host "bootstrap: OK (check-only)"; exit 0 }
  Write-Host "bootstrap: MISSING tooling"
  exit 1
}

function Add-MsvcToPath {
  $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
  if (-not (Test-Path $vswhere)) { return $false }
  $install = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
  if (-not $install) {
    $install = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.ARM64 -property installationPath 2>$null
  }
  if (-not $install) { return $false }
  $msvc = Get-ChildItem (Join-Path $install "VC\Tools\MSVC") -Directory -ErrorAction SilentlyContinue |
    Sort-Object Name -Descending | Select-Object -First 1
  if (-not $msvc) { return $false }
  $hostArch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" }
  $bin = Join-Path $msvc.FullName ("bin\Host" + $hostArch + "\" + $hostArch)
  if (-not (Test-Path $bin)) {
    $bin = Join-Path $msvc.FullName "bin\Hostx64\x64"
  }
  if (-not (Test-Path $bin)) { return $false }
  $env:Path = $bin + ";" + $env:Path
  # Persist for subsequent remote/shell sessions.
  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if ($userPath -notlike "*$bin*") {
    [Environment]::SetEnvironmentVariable("Path", ($bin + ";" + $userPath), "User")
  }
  Write-Host ("  PATH append: " + $bin)
  return $true
}

if (Test-LinkExe) {
  # Ensure PATH is refreshed even when link.exe was found via vswhere recurse.
  if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
    [void](Add-MsvcToPath)
  }
  Write-Host "bootstrap: already have link.exe - skip install"
  Write-Status
  Write-Host "bootstrap: OK"
  exit 0
}

if (-not $SkipWinget) {
  $winget = Get-Command winget -ErrorAction SilentlyContinue
  if ($null -eq $winget) {
    Write-Host "ERROR: winget not found. Install App Installer from Microsoft Store, or re-run with -SkipWinget after manual VS install."
    exit 1
  }

  Write-Host "==> winget: Visual Studio 2022 Build Tools (this can take a long time)"
  # VC tools + MSBuild + Windows 10/11 SDK. ARM64 UTM needs ARM64 VC tools when present.
  $vsArgs = @(
    "--quiet", "--wait", "--norestart",
    "--add", "Microsoft.VisualStudio.Workload.VCTools",
    "--add", "Microsoft.VisualStudio.Workload.MSBuildTools",
    "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22621",
    "--includeRecommended"
  )
  winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-package-agreements --accept-source-agreements --override ($vsArgs -join " ")
  if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne -1978335189) {
    Write-Host ("winget VS Build Tools exit: " + $LASTEXITCODE + " (continuing to verify)")
  }

  Write-Host "==> winget: Windows App Runtime"
  winget install --id Microsoft.WindowsAppRuntime.1.6 -e --accept-package-agreements --accept-source-agreements
  if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne -1978335189) {
    Write-Host ("ERROR: winget WinAppRuntime exit: " + $LASTEXITCODE)
    exit $LASTEXITCODE
  }
}

# Refresh PATH from vswhere for current session
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
  $install = & $vswhere -latest -products * -property installationPath
  if ($install) {
    $msvc = Get-ChildItem -Path (Join-Path $install "VC\Tools\MSVC") -Directory -ErrorAction SilentlyContinue |
      Sort-Object Name -Descending | Select-Object -First 1
    if ($msvc) {
      $hostArch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" }
      $bin = Join-Path $msvc.FullName ("bin\Host" + $hostArch + "\" + $hostArch)
      if (-not (Test-Path $bin)) {
        $bin = Join-Path $msvc.FullName "bin\Hostx64\x64"
      }
      if (Test-Path $bin) {
        $env:Path = $bin + ";" + $env:Path
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$bin*") {
          [Environment]::SetEnvironmentVariable("Path", ($bin + ";" + $userPath), "User")
        }
        Write-Host ("  PATH append: " + $bin)
      }
    }
  }
}

Write-Status
if (-not (Test-LinkExe)) {
  Write-Host "ERROR: link.exe still missing after bootstrap. Open 'Developer PowerShell for VS' or reboot and re-run."
  exit 1
}

Write-Host "bootstrap: OK"
