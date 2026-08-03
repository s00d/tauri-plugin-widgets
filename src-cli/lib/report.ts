export type ReportLevel = "ok" | "warn" | "bad";

export interface ReportItem {
  section: string;
  level: ReportLevel;
  msg: string;
}

export interface Report {
  items: ReportItem[];
  failures: number;
  warnings: number;
  /** Start a new named section — subsequent ok/warn/bad calls belong to it. */
  section(name: string): void;
  ok(msg: string): void;
  warn(msg: string): void;
  bad(msg: string): void;
  /** Print the report to the console in the classic `doctor` layout. */
  print(): void;
}

/**
 * Collects sectioned check results (✓ / ⚠ / ✗) without printing, so callers
 * (tests, `doctor` command) can inspect `failures` / `warnings` / `items`
 * before deciding whether/how to print.
 */
export function createReport(): Report {
  let current = "";
  const items: ReportItem[] = [];

  const report: Report = {
    items,
    failures: 0,
    warnings: 0,
    section(name: string) {
      current = name;
    },
    ok(msg: string) {
      items.push({ section: current, level: "ok", msg });
    },
    warn(msg: string) {
      report.warnings++;
      items.push({ section: current, level: "warn", msg });
    },
    bad(msg: string) {
      report.failures++;
      items.push({ section: current, level: "bad", msg });
    },
    print() {
      let lastSection: string | null = null;
      for (const item of items) {
        if (item.section !== lastSection) {
          // Section name is printed verbatim — callers embed their own leading
          // "\n" (or not) to match the original doctor console output exactly.
          console.log(item.section);
          lastSection = item.section;
        }
        const mark = item.level === "ok" ? "✓" : item.level === "warn" ? "⚠" : "✗";
        console.log(`  ${mark} ${item.msg}`);
      }
      console.log(`\n${report.failures ? "FAIL" : "OK"} — ${report.failures} error(s), ${report.warnings} warning(s)`);
    },
  };

  return report;
}
