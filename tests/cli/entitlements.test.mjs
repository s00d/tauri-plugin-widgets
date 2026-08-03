import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import {
  entitlementsPlist,
  mergeAppGroupIntoEntitlements,
  xmlEscapePlist,
} from "../../dist-cli/lib/entitlements.mjs";

describe("entitlements XML helpers", () => {
  it("xmlEscapePlist escapes markup delimiters", () => {
    assert.equal(xmlEscapePlist(`a&b<"'>`), "a&amp;b&lt;&quot;&apos;&gt;");
  });

  it("rejects control characters in appGroup", () => {
    assert.throws(() => entitlementsPlist({ appGroup: "group.bad\u0001id" }), /XML-unsafe/);
  });

  it("merge into self-closing array keeps $1 literally", () => {
    const dir = mkdtempSync(join(tmpdir(), "tpw-ents-"));
    const path = join(dir, "App.entitlements");
    writeFileSync(
      path,
      `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.application-groups</key>
    <array/>
</dict>
</plist>
`,
    );
    // Dollar tokens must not be interpreted as JS replacement patterns.
    mergeAppGroupIntoEntitlements(path, "group.$1.example");
    const raw = readFileSync(path, "utf-8");
    assert.match(raw, /<string>group\.\$1\.example<\/string>/);
    assert.doesNotMatch(raw, /group\.<array>/);
  });
});
