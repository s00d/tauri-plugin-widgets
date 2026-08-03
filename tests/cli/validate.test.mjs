import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { validateWidgetShape } from "../../dist-cli/lib/validate.mjs";

describe("validateWidgetShape", () => {
  it("rejects object-valued image data", () => {
    const findings = validateWidgetShape({
      small: { type: "image", data: { nested: true } },
    });
    assert.ok(
      findings.some((f) => f.level === "error" && /image/.test(f.note || "")),
      JSON.stringify(findings),
    );
  });

  it("rejects nested children without string type", () => {
    const findings = validateWidgetShape({
      small: {
        type: "vstack",
        children: [{ content: "no type" }, "bare-string"],
      },
    });
    assert.ok(
      findings.some((f) => f.level === "error" && /type/.test(f.note || "")),
      JSON.stringify(findings),
    );
  });
});
