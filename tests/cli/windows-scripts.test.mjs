// Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI: a UTF-8 em dash (E2 80 94) turns into
// a closing quote (0x94 = ”) and the whole script fails to parse. Keep scripts/win ASCII-only.
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const DIR = join(import.meta.dirname, "../../scripts/win");

describe("scripts/win PowerShell", () => {
  // Given every .ps1 the VM runs, when read as bytes, then none is outside ASCII.
  it("is ASCII-only", () => {
    for (const f of readdirSync(DIR).filter((n) => n.endsWith(".ps1"))) {
      const bad = [...readFileSync(join(DIR, f))].findIndex((b) => b > 0x7e);
      assert.equal(bad, -1, `${f}: non-ASCII byte at offset ${bad}`);
    }
  });
});
