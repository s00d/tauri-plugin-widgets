#Requires -Version 5.1
# Sideload helper for a built AppX/MSIX (Developer Mode required).
param(
  [string]$PackagePath = "C:\work\tauri-plugin-widgets\out\windows\TauriWidgets.msix"
)

$ErrorActionPreference = "Stop"

$devMode = (Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" -Name AllowDevelopmentWithoutDevLicense -ErrorAction SilentlyContinue).AllowDevelopmentWithoutDevLicense
if ($devMode -ne 1) {
  Write-Host "WARNING: Developer Mode may be off. Enable Settings → Privacy & security → For developers → Developer Mode"
}

if (-not (Test-Path $PackagePath)) {
  Write-Host ("ERROR: package not found: " + $PackagePath)
  Write-Host "Run pack.ps1 first and complete MSIX packaging (fragment + identity) for your app."
  exit 1
}

Write-Host ("==> Add-AppxPackage " + $PackagePath)
Add-AppxPackage -Path $PackagePath -ForceApplicationShutdown
Write-Host "sideload: OK - open Widgets Board (Win+W) to pin the widget"
