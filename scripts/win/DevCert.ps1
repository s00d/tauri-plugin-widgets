#Requires -Version 5.1
# Self-signed cert for MSIX sideload / Widgets Board.
# Subject MUST equal Package.appxmanifest Identity/@Publisher exactly.
# Prefer regenerating via: npx tauri-widgets signing --dev-cert
param(
  [string]$Publisher = "CN=TauriWidgetsDev",
  [string]$PfxPath = "dev.pfx",
  [string]$Password = "dev",
  [string]$MsixPath = ""
)

$ErrorActionPreference = "Stop"
$cert = New-SelfSignedCertificate -Type Custom -Subject $Publisher `
  -KeyUsage DigitalSignature -CertStoreLocation "Cert:\CurrentUser\My" `
  -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
$pwd = ConvertTo-SecureString $Password -Force -AsPlainText
Export-PfxCertificate -Cert "Cert:\CurrentUser\My\$($cert.Thumbprint)" -FilePath $PfxPath -Password $pwd | Out-Null
Import-PfxCertificate -FilePath $PfxPath -CertStoreLocation Cert:\LocalMachine\TrustedPeople -Password $pwd | Out-Null
Write-Host "Thumbprint: $($cert.Thumbprint)"
Write-Host "PFX: $PfxPath (password=$Password)"
Write-Host "Imported into LocalMachine\TrustedPeople"
if ($MsixPath) {
  $signtool = Get-Command signtool -ErrorAction SilentlyContinue
  if (-not $signtool) {
    Write-Error "signtool not on PATH. Install Windows SDK / use Developer Command Prompt."
  }
  & signtool sign /fd SHA256 /f $PfxPath /p $Password $MsixPath
  Write-Host "Signed: $MsixPath"
} else {
  Write-Host "Sign later: signtool sign /fd SHA256 /f $PfxPath /p $Password path\to\app.msix"
}
