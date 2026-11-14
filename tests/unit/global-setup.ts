import { execFileSync } from "node:child_process";
import { join } from "node:path";

// The parity test runs the Rust reference binary; build it here, outside any test timeout
// (a fresh compile after Rust changes takes minutes).
export default function setup(): void {
  if (process.env.STRYKER_MUTATOR_WORKER) return;
  execFileSync("cargo", ["build", "-q", "--bin", "resolve-stdin"], { cwd: join(__dirname, "../.."), stdio: "inherit" });
}
