#Requires -Version 5.1
# Build WidgetProvider MSIX-ish package layout (requires bootstrap / Windows SDK).
param(
  [string]$Repo = "C:\work\tauri-plugin-widgets",
  [string]$OutDir = "C:\work\tauri-plugin-widgets\out\windows"
)

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$src = Join-Path $Repo "templates\windows-widget"
$work = Join-Path $env:TEMP "tpw-msix-pack"
if (Test-Path $work) { Remove-Item -Recurse -Force $work }
Copy-Item -Recurse $src $work
Get-ChildItem -Path $work -Recurse -Force -Filter "._*" | Remove-Item -Force -ErrorAction SilentlyContinue

$clsid = [guid]::NewGuid().ToString()
$replacements = @{
  "{{WIDGET_PROVIDER_CLSID}}" = $clsid
  "{{WIDGET_DISPLAY_NAME}}" = "TauriWidgets"
  "{{PACKAGE_NAME}}" = "TauriWidgets"
  "{{WIDGET_EXTENSION_ID}}" = "TauriWidgets.Widgets"
  "{{WIDGET_DEFINITION_ID}}" = "TauriWidgets.Widget"
}
Get-ChildItem -Path $work -Recurse -File | ForEach-Object {
  $text = Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue
  if ($null -eq $text) { return }
  foreach ($k in $replacements.Keys) {
    if ($text.Contains($k)) { $text = $text.Replace($k, $replacements[$k]) }
  }
  Set-Content -Path $_.FullName -Value $text -NoNewline
}

$rid = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "win-arm64" } else { "win-x64" }
$platform = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "ARM64" } else { "x64" }
$proj = Join-Path $work "WidgetProvider\WidgetProvider.csproj"

$smoke = $true
if (Get-Command link.exe -ErrorAction SilentlyContinue) {
  # Try full build first; fall back to Smoke.
  Write-Host "==> dotnet build WidgetProvider (full)"
  dotnet build $proj -c Release -p:Platform=$platform -p:RuntimeIdentifier=$rid -p:Smoke=false
  if ($LASTEXITCODE -ne 0) {
    Write-Host "full build failed - Smoke=true"
    $smoke = $true
  } else {
    $smoke = $false
  }
}

if ($smoke) {
  Write-Host "==> dotnet build WidgetProvider (Smoke=true)"
  dotnet build $proj -c Release -p:Platform=$platform -p:RuntimeIdentifier=$rid -p:Smoke=true
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$publishDir = Join-Path $OutDir "WidgetProvider-publish"
New-Item -ItemType Directory -Force -Path $publishDir | Out-Null
dotnet publish $proj -c Release -p:Platform=$platform -p:RuntimeIdentifier=$rid -p:Smoke=$smoke -o $publishDir
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Copy-Item (Join-Path $work "Package.appxmanifest.fragment.xml") (Join-Path $OutDir "Package.appxmanifest.fragment.xml") -Force
Copy-Item (Join-Path $work "Assets") (Join-Path $OutDir "Assets") -Recurse -Force -ErrorAction SilentlyContinue

$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter makeappx.exe -ErrorAction SilentlyContinue |
  Select-Object -First 1
if ($null -ne $makeappx) {
  Write-Host ("==> makeappx found: " + $makeappx.FullName)
  Write-Host "    (full Package.appxmanifest assembly is app-specific - fragment + publish dir ready)"
} else {
  Write-Host "makeappx: not found - publish dir is still usable; install Windows SDK via bootstrap.ps1"
}

Write-Host ("pack: OK → " + $publishDir)
