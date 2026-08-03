import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  loadCapabilities,
  validateWidgetConfig,
  validateWidgetShape,
} from "../../dist-cli/lib/validate.mjs";

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

  it("accepts list items without element type", () => {
    const findings = validateWidgetShape({
      small: {
        type: "list",
        items: [{ text: "One", checked: false, action: "toggle", payload: "0" }],
      },
    });
    assert.equal(
      findings.filter((f) => f.level === "error").length,
      0,
      JSON.stringify(findings),
    );
  });
});

describe("validateWidgetConfig list rows", () => {
  it("does not treat list items as element children", () => {
    const { data: caps } = loadCapabilities();
    const findings = validateWidgetConfig(
      {
        small: {
          type: "list",
          items: [{ text: "One", checked: false, action: "toggle", payload: "0" }],
        },
      },
      ["android"],
      caps,
    );
    assert.equal(
      findings.filter((f) => /child must have string type/.test(f.note || "")).length,
      0,
      JSON.stringify(findings),
    );
  });
});
