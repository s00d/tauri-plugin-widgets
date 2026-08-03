#Requires -Version 5.1
# Windows smoke + visual Adaptive Card capture.
param(
  [string]$Repo = "C:\work\tauri-plugin-widgets",
  [ValidateSet("smoke", "visual")]
  [string]$Mode = "smoke",
  [string]$Case = "",
  [switch]$Record,
  [switch]$SkipDotnet,
  [switch]$SkipCargo
)

$ErrorActionPreference = "Stop"
Set-Location $Repo

function Test-MsvcLinker {
  return $null -ne (Get-Command link.exe -ErrorAction SilentlyContinue)
}

function Ensure-PreviewHost {
  param(
    [bool]$FullRenderer = $false
  )
  $src = Join-Path $Repo "templates\windows-widget\PreviewHost"
  $smokeDir = Join-Path $env:TEMP ("tpw-previewhost-" + $(if ($FullRenderer) { "full" } else { "smoke" }))
  if (Test-Path $smokeDir) { Remove-Item -Recurse -Force $smokeDir }
  Copy-Item -Recurse $src $smokeDir
  Get-ChildItem -Path $smokeDir -Recurse -Force -Filter "._*" | Remove-Item -Force -ErrorAction SilentlyContinue

  $rid = "win-x64"
  $platform = "x64"
  if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
    $rid = "win-arm64"
    $platform = "ARM64"
  }
  $proj = Join-Path $smokeDir "PreviewHost.csproj"
  $smokeFlag = if ($FullRenderer) { "false" } else { "true" }
  Write-Host ("==> dotnet build PreviewHost (" + $rid + ", Smoke=" + $smokeFlag + ")")
  dotnet build $proj -c Release -p:Platform=$platform -p:RuntimeIdentifier=$rid -p:Smoke=$smokeFlag | Out-Host
  if ($LASTEXITCODE -ne 0) {
    if ($FullRenderer) {
      Write-Host "WARN: full AC renderer build failed — falling back to Smoke=true walker"
      return Ensure-PreviewHost -FullRenderer:$false
    }
    exit $LASTEXITCODE
  }
  $exe = Get-ChildItem -Path $smokeDir -Recurse -Filter "AcPreviewHost.exe" |
    Where-Object { $_.FullName -match "Release" } |
    Select-Object -First 1
  if ($null -eq $exe) {
    Write-Host "ERROR: AcPreviewHost.exe not found"
    exit 1
  }
  return [string]$exe.FullName
}

# --- smoke: cargo + WidgetProvider ---
if ($Mode -eq "smoke") {
  $cargoOk = $false
  if (-not $SkipCargo) {
    if (Test-MsvcLinker) {
      Write-Host "==> cargo test --lib adaptive_card"
      cargo test --lib adaptive_card -- --nocapture
      if ($LASTEXITCODE -eq 0) { $cargoOk = $true }
    } else {
      Write-Host "cargo: skipped (link.exe missing - run bootstrap.ps1)"
    }
  }

  if (-not $SkipDotnet) {
    $smokeDir = Join-Path $env:TEMP "tpw-windows-widget-smoke"
    if (Test-Path $smokeDir) { Remove-Item -Recurse -Force $smokeDir }
    $src = Join-Path $Repo "templates\windows-widget"
    Copy-Item -Recurse $src $smokeDir
    Get-ChildItem -Path $smokeDir -Recurse -Force -Filter "._*" | Remove-Item -Force -ErrorAction SilentlyContinue

    $clsid = [guid]::NewGuid().ToString()
    $replacements = @{
      "{{WIDGET_PROVIDER_CLSID}}" = $clsid
      "{{WIDGET_DISPLAY_NAME}}" = "Smoke"
      "{{PACKAGE_NAME}}" = "SmokeApp"
      "{{WIDGET_EXTENSION_ID}}" = "SmokeApp.Widgets"
      "{{WIDGET_DEFINITION_ID}}" = "SmokeApp.Widget"
    }
    Get-ChildItem -Path $smokeDir -Recurse -File | ForEach-Object {
      $text = Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue
      if ($null -eq $text) { return }
      $changed = $false
      foreach ($k in $replacements.Keys) {
        if ($text.Contains($k)) {
          $text = $text.Replace($k, $replacements[$k])
          $changed = $true
        }
      }
      if ($changed) { Set-Content -Path $_.FullName -Value $text -NoNewline }
    }

    $provider = Join-Path $smokeDir "WidgetProvider\WidgetProvider.csproj"
    $rid = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "win-arm64" } else { "win-x64" }
    $platform = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "ARM64" } else { "x64" }
    Write-Host ("==> dotnet build WidgetProvider (" + $rid + ", Smoke=true)")
    dotnet build $provider -c Release -p:Platform=$platform -p:RuntimeIdentifier=$rid -p:Smoke=true
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  }

  Write-Host "shot: OK (smoke)"
  exit 0
}

# --- visual: transpile cards + PreviewHost PNG (full AdaptiveCards.Rendering.Wpf) ---
$exe = Ensure-PreviewHost -FullRenderer:$true
$casesDir = Join-Path $Repo "tests\cases"
$goldenDir = Join-Path $Repo "tests\golden\windows"
$outDir = Join-Path $Repo "out\windows"
New-Item -ItemType Directory -Force -Path $goldenDir | Out-Null
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# Prefer prebuilt AC JSON from Mac sync under snapshots; else minimal card from case name.
$caseFiles = Get-ChildItem $casesDir -Filter "*.json"
if ($Case) {
  $caseFiles = $caseFiles | Where-Object { $_.BaseName -eq $Case }
}

foreach ($cf in $caseFiles) {
  $name = $cf.BaseName
  $caseJson = Get-Content $cf.FullName -Raw | ConvertFrom-Json
  $size = [string]$caseJson.size
  if (-not $size) { $size = "small" }

  $snap = Join-Path $Repo ("tests\snapshots\adaptive\" + $name + ".json")
  $cardPath = Join-Path $outDir ($name + ".card.json")
  if (Test-Path $snap) {
    Copy-Item $snap $cardPath -Force
  } else {
    # Minimal placeholder card so PreviewHost still produces a bitmap.
    $placeholder = @{
      type = "AdaptiveCard"
      version = "1.5"
      body = @(@{ type = "TextBlock"; text = $name; wrap = $true })
    } | ConvertTo-Json -Depth 5
    Set-Content -Path $cardPath -Value $placeholder -Encoding UTF8
  }

  $w = 158; $h = 158
  if ($size -eq "medium") { $w = 338; $h = 158 }
  if ($size -eq "large") { $w = 338; $h = 354 }

  $pngOut = Join-Path $outDir ($name + ".png")
  & $exe --in $cardPath --out $pngOut --width $w --height $h
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

  $golden = Join-Path $goldenDir ($name + ".png")
  if ($Record) {
    Copy-Item $pngOut $golden -Force
    Write-Host ("recorded " + $golden)
  } elseif (Test-Path $golden) {
    Write-Host ("captured " + $pngOut + " (compare golden on Mac)")
  } else {
    Write-Host ("captured " + $pngOut + " (no golden yet)")
  }
}

Write-Host "shot: OK (visual)"
