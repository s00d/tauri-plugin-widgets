import { spawnSync } from "node:child_process";

export type IdentityKind = "distribution" | "development" | "other";

export interface Identity {
  hash: string;
  name: string;
  team: string | null;
  kind: IdentityKind;
}

/** Parse `security find-identity -v -p codesigning` (macOS only). */
export function codesignIdentities(): Identity[] {
  if (process.platform !== "darwin") return [];
  const r = spawnSync("security", ["find-identity", "-v", "-p", "codesigning"], {
    encoding: "utf-8",
  });
  const out = `${r.stdout || ""}\n${r.stderr || ""}`;
  const re = /\d+\)\s+([0-9A-F]{40})\s+"([^"]+)"/gi;
  const found: Identity[] = [];
  for (let m: RegExpExecArray | null; (m = re.exec(out)); ) {
    const name = m[2];
    const team = (name.match(/\(([A-Z0-9]{10})\)\s*$/) || [])[1] || null;
    const kind: IdentityKind = /Developer ID/i.test(name)
      ? "distribution"
      : /Apple Development|iPhone Developer|Mac Developer/i.test(name)
        ? "development"
        : "other";
    found.push({ hash: m[1].toUpperCase(), name, team, kind });
  }
  return found;
}

export function identitiesHaveTeam(identities: Identity[]): boolean {
  return identities.some((i) => Boolean(i.team));
}

/** Recommend host transport from available codesign identities (macOS). */
export function recommendTransport(
  identities: Identity[],
  { forIos = false }: { forIos?: boolean } = {},
): string {
  if (forIos) return "appGroup";
  return identitiesHaveTeam(identities) ? "appGroup" : "widgetContainer";
}

export function pickPreferredIdentity(identities: Identity[]): Identity | null {
  if (!identities.length) return null;
  return (
    identities.find((i) => i.kind === "distribution") ||
    identities.find((i) => i.kind === "development") ||
    identities[0]
  );
}

export function codesignIsAdHoc(appPath: string): boolean | null {
  const r = spawnSync("codesign", ["-dv", "--verbose=2", appPath], {
    encoding: "utf-8",
  });
  const text = `${r.stdout || ""}\n${r.stderr || ""}`;
  if (/Signature=adhoc|Authority=\(ad hoc\)|flags=0x2\(adhoc\)/i.test(text)) return true;
  if (/Authority=|TeamIdentifier=/i.test(text)) return false;
  return null;
}

export function windowsDevCertScript(
  publisher: string,
  { pfxPath = "dev.pfx", password = "dev" }: { pfxPath?: string; password?: string } = {},
): string {
  return `#Requires -Version 5.1
# Self-signed cert for MSIX sideload / Widgets Board.
# Subject MUST equal Package.appxmanifest Identity/@Publisher exactly.
# Run elevated PowerShell on Windows.
param(
  [string]$Publisher = "${publisher.replace(/"/g, '`"')}",
  [string]$PfxPath = "${pfxPath}",
  [string]$Password = "${password}",
  [string]$MsixPath = ""
)

$ErrorActionPreference = "Stop"
$cert = New-SelfSignedCertificate -Type Custom -Subject $Publisher \`
  -KeyUsage DigitalSignature -CertStoreLocation "Cert:\\CurrentUser\\My" \`
  -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
$pwd = ConvertTo-SecureString $Password -Force -AsPlainText
Export-PfxCertificate -Cert "Cert:\\CurrentUser\\My\\$($cert.Thumbprint)" -FilePath $PfxPath -Password $pwd | Out-Null
Import-PfxCertificate -FilePath $PfxPath -CertStoreLocation Cert:\\LocalMachine\\TrustedPeople -Password $pwd | Out-Null
Write-Host "Thumbprint: $($cert.Thumbprint)"
Write-Host "PFX: $PfxPath (password=$Password)"
Write-Host "Imported into LocalMachine\\TrustedPeople"
if ($MsixPath) {
  $signtool = Get-Command signtool -ErrorAction SilentlyContinue
  if (-not $signtool) {
    Write-Error "signtool not on PATH. Install Windows SDK / use Developer Command Prompt."
  }
  & signtool sign /fd SHA256 /f $PfxPath /p $Password $MsixPath
  Write-Host "Signed: $MsixPath"
} else {
  Write-Host "Sign later: signtool sign /fd SHA256 /f $PfxPath /p $Password path\\to\\app.msix"
}
`;
}
