import { defineCommand } from "citty";
import { startPreviewServer } from "../lib/preview.js";

export const previewCmd = defineCommand({
  meta: {
    name: "preview",
    description: "Serve widget.html + config JSON with live reload (no native build)",
  },
  args: {
    config: {
      type: "positional",
      description: "Path to widget config JSON",
      required: true,
    },
    size: { type: "string", description: "small | medium | large", default: "medium" },
    port: { type: "string", description: "HTTP port", default: "4177" },
    watch: { type: "boolean", description: "Reload on file change", default: true },
    open: { type: "boolean", description: "Open browser", default: false },
  },
  async run({ args }) {
    const port = Number(args.port) || 4177;
    await startPreviewServer({
      configPath: args.config,
      size: args.size || "medium",
      port,
      watchFile: args.watch !== false,
      open: Boolean(args.open),
    });
    await new Promise(() => {});
  },
});
