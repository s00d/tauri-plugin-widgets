import { readFileSync, writeFileSync, chmodSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { cwd } from "node:process";
import typescript from "@rollup/plugin-typescript";

const pkg = JSON.parse(readFileSync(join(cwd(), "package.json"), "utf8"));
const runtimeDeps = Object.keys(pkg.dependencies || {});

/** Guest JS API (existing). */
const lib = {
  input: "guest-js/index.ts",
  output: [
    { file: pkg.exports.import, format: "esm" },
    { file: pkg.exports.require, format: "cjs" },
  ],
  plugins: [
    typescript({
      declaration: true,
      declarationDir: dirname(pkg.exports.import),
    }),
  ],
  external: [
    /^@tauri-apps\/api/,
    ...runtimeDeps,
    ...Object.keys(pkg.peerDependencies || {}),
  ],
};

/** Consumer CLI — preserveModules so commands/lib stay importable in tests. */
const cli = {
  input: "src-cli/cli.ts",
  output: {
    dir: "dist-cli",
    format: "esm",
    entryFileNames: "[name].mjs",
    chunkFileNames: "[name].mjs",
    preserveModules: true,
    preserveModulesRoot: "src-cli",
  },
  plugins: [
    typescript({
      tsconfig: "src-cli/tsconfig.json",
      declaration: false,
      // Avoid clobbering guest-js emit from the other build in the same rollup run
      outputToFilesystem: true,
    }),
    {
      name: "cli-shebang-chmod",
      generateBundle(_opts, bundle) {
        for (const [fileName, chunk] of Object.entries(bundle)) {
          if (chunk.type !== "chunk") continue;
          if (fileName === "cli.mjs" || fileName.endsWith("/cli.mjs")) {
            if (!chunk.code.startsWith("#!")) {
              chunk.code = "#!/usr/bin/env node\n" + chunk.code;
            }
          }
        }
      },
      writeBundle() {
        const main = join(cwd(), "dist-cli", "cli.mjs");
        if (existsSync(main)) chmodSync(main, 0o755);
      },
    },
  ],
  external: [/^node:/, ...runtimeDeps],
};

/** Desktop widget HTML — bundle widget-src/ → widget.html (Rust include_bytes!). */
const widgetShell = join(cwd(), "widget-src", "shell.html");
const widgetOut = join(cwd(), "widget.html");

const widget = {
  input: "widget-src/main.ts",
  output: {
    file: "widget-src/.bundle.js",
    format: "iife",
    strict: true,
  },
  plugins: [
    typescript({
      tsconfig: "widget-src/tsconfig.json",
      declaration: false,
      noEmitOnError: true,
      // Must share the directory of `output.file` (plugin constraint).
      compilerOptions: {
        noEmit: false,
        outDir: join(cwd(), "widget-src"),
      },
    }),
    {
      name: "widget-html",
      writeBundle(_opts, bundle) {
        const chunk = Object.values(bundle).find((c) => c.type === "chunk");
        if (!chunk) throw new Error("widget build: no output chunk");
        const shell = readFileSync(widgetShell, "utf8");
        const html = shell.replace(
          "<!--WIDGET_JS-->",
          "<script>\n" + chunk.code + "\n</script>"
        );
        writeFileSync(widgetOut, html);
      },
    },
  ],
};

const all = [lib, cli, widget];
export default process.env.WIDGET_ONLY ? [widget] : all;
