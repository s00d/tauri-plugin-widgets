import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  entitlementsPlist,
  xmlEscapePlist,
} from "../../dist-cli/lib/entitlements.mjs";

describe("entitlements XML helpers", () => {
  it("xmlEscapePlist escapes markup delimiters", () => {
    assert.equal(xmlEscapePlist(`a&b<"'>`), "a&amp;b&lt;&quot;&apos;&gt;");
  });

  it("rejects control characters in appGroup", () => {
    assert.throws(() => entitlementsPlist({ appGroup: "group.bad\u0001id" }), /XML-unsafe/);
  });

});
