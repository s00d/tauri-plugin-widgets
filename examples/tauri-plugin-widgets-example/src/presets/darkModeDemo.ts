import type { PresetDef } from "./types";

export const darkModeDemo: PresetDef = {
  icon: "\u{1F305}",
  name: "Dark Mode Demo",
  onAction(action, _payload, log) {
    if (action === "toggle_theme") log("Theme toggle is handled by OS");
    if (action === "demo_action") log("Demo action triggered");
  },
  config: {
    small: {
      type: "vstack", padding: 14, spacing: 8, cornerRadius: 16,
      background: "systemBackground",
      children: [
        { type: "text", content: "Adaptive Widget", textStyle: "headline", color: "label" },
        { type: "text", content: "Colors adapt to system theme automatically", textStyle: "caption", color: "secondaryLabel" },
        { type: "spacer" },
        { type: "hstack", spacing: 8, children: [
          { type: "container", contentAlignment: "center", padding: 8,
            background: { light: "#DBEAFE", dark: "#1E3A5F" }, cornerRadius: 8, clipShape: "capsule", flex: 1,
            children: [
              { type: "text", content: "Label", textStyle: "caption2", fontWeight: "semibold", color: { light: "#1D4ED8", dark: "#93C5FD" } },
            ],
          },
          { type: "container", contentAlignment: "center", padding: 8,
            background: { light: "#FEE2E2", dark: "#5F1E1E" }, cornerRadius: 8, clipShape: "capsule", flex: 1,
            children: [
              { type: "text", content: "Badge", textStyle: "caption2", fontWeight: "semibold", color: { light: "#DC2626", dark: "#FCA5A5" } },
            ],
          },
        ]},
      ],
    },

    medium: {
      type: "hstack", padding: 16, spacing: 14, cornerRadius: 16,
      background: "systemBackground",
      children: [
        { type: "vstack", spacing: 8, alignment: "center", children: [
          { type: "container", contentAlignment: "center",
            background: "accent", cornerRadius: 24, clipShape: "circle",
            frame: { width: 48, height: 48 },
            children: [
              { type: "image", systemName: "paintpalette.fill", color: "#ffffff", size: 24 },
            ],
          },
          { type: "text", content: "Theming", textStyle: "footnote", fontWeight: "semibold", color: "label" },
        ]},
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
          { type: "text", content: "Semantic Colors", textStyle: "subheadline", fontWeight: "semibold", color: "label" },
          { type: "hstack", spacing: 6, children: [
            { type: "container", contentAlignment: "center", padding: 4,
              background: "systemRed", cornerRadius: 4, frame: { width: 24, height: 24 },
              children: [{ type: "text", content: "R", fontSize: 10, fontWeight: "bold", color: "#fff" }]},
            { type: "container", contentAlignment: "center", padding: 4,
              background: "systemGreen", cornerRadius: 4, frame: { width: 24, height: 24 },
              children: [{ type: "text", content: "G", fontSize: 10, fontWeight: "bold", color: "#fff" }]},
            { type: "container", contentAlignment: "center", padding: 4,
              background: "systemBlue", cornerRadius: 4, frame: { width: 24, height: 24 },
              children: [{ type: "text", content: "B", fontSize: 10, fontWeight: "bold", color: "#fff" }]},
            { type: "container", contentAlignment: "center", padding: 4,
              background: "systemOrange", cornerRadius: 4, frame: { width: 24, height: 24 },
              children: [{ type: "text", content: "O", fontSize: 10, fontWeight: "bold", color: "#fff" }]},
            { type: "container", contentAlignment: "center", padding: 4,
              background: "systemPurple", cornerRadius: 4, frame: { width: 24, height: 24 },
              children: [{ type: "text", content: "P", fontSize: 10, fontWeight: "bold", color: "#fff" }]},
          ]},
          { type: "text", content: "These colors auto-adjust for light/dark mode", textStyle: "caption2", color: "secondaryLabel" },
          { type: "progress", value: 0.7, tint: "accent", label: "Completion", color: "secondaryLabel" },
        ]},
      ],
    },

    large: {
      type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
      background: "systemBackground",
      children: [
        { type: "hstack", spacing: 10, children: [
          { type: "container", contentAlignment: "center",
            background: { gradientType: "linear", colors: ["#667eea", "#764ba2"], direction: "topLeadingToBottomTrailing" },
            cornerRadius: 24, clipShape: "circle",
            frame: { width: 48, height: 48 },
            children: [
              { type: "image", systemName: "moon.stars.fill", color: "#ffffff", size: 24 },
            ],
          },
          { type: "vstack", spacing: 2, alignment: "leading", flex: 1, children: [
            { type: "text", content: "Dark Mode Demo", textStyle: "title3", fontWeight: "bold", color: "label" },
            { type: "text", content: "All new styling features in action", textStyle: "footnote", color: "secondaryLabel" },
          ]},
        ]},
        { type: "divider", color: "separator" },

        { type: "text", content: "Text Styles (semantic typography)", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "hstack", spacing: 8, children: [
          { type: "container", contentAlignment: "center", padding: 8, flex: 1,
            background: { light: "#F3F4F6", dark: "#27272A" }, cornerRadius: 8,
            children: [
              { type: "vstack", spacing: 4, alignment: "center", children: [
                { type: "text", content: "Title", textStyle: "title3", color: "label" },
                { type: "text", content: ".title3", textStyle: "caption2", color: "secondaryLabel" },
              ]},
            ],
          },
          { type: "container", contentAlignment: "center", padding: 8, flex: 1,
            background: { light: "#F3F4F6", dark: "#27272A" }, cornerRadius: 8,
            children: [
              { type: "vstack", spacing: 4, alignment: "center", children: [
                { type: "text", content: "Body", textStyle: "body", color: "label" },
                { type: "text", content: ".body", textStyle: "caption2", color: "secondaryLabel" },
              ]},
            ],
          },
          { type: "container", contentAlignment: "center", padding: 8, flex: 1,
            background: { light: "#F3F4F6", dark: "#27272A" }, cornerRadius: 8,
            children: [
              { type: "vstack", spacing: 4, alignment: "center", children: [
                { type: "text", content: "Caption", textStyle: "caption", color: "label" },
                { type: "text", content: ".caption", textStyle: "caption2", color: "secondaryLabel" },
              ]},
            ],
          },
        ]},

        { type: "text", content: "Clip Shapes", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "hstack", spacing: 12, children: [
          { type: "vstack", spacing: 4, alignment: "center", flex: 1, children: [
            { type: "container", contentAlignment: "center", clipShape: "circle",
              background: { light: "#DBEAFE", dark: "#1E3A5F" },
              frame: { width: 48, height: 48 },
              children: [
                { type: "image", systemName: "person.fill", color: { light: "#2563EB", dark: "#93C5FD" }, size: 20 },
              ],
            },
            { type: "text", content: "circle", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 4, alignment: "center", flex: 1, children: [
            { type: "container", contentAlignment: "center", clipShape: "capsule",
              background: { light: "#D1FAE5", dark: "#064E3B" },
              frame: { width: 80, height: 32 },
              children: [
                { type: "text", content: "Status", textStyle: "caption", fontWeight: "semibold", color: { light: "#059669", dark: "#6EE7B7" } },
              ],
            },
            { type: "text", content: "capsule", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 4, alignment: "center", flex: 1, children: [
            { type: "container", contentAlignment: "center", clipShape: "rectangle",
              background: { light: "#FEF3C7", dark: "#78350F" }, cornerRadius: 8,
              frame: { width: 48, height: 48 },
              children: [
                { type: "image", systemName: "star.fill", color: "#F59E0B", size: 20 },
              ],
            },
            { type: "text", content: "rectangle", textStyle: "caption2", color: "secondaryLabel" },
          ]},
        ]},

        { type: "text", content: "Flex Layout (equal distribution)", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "hstack", spacing: 6, children: [
          { type: "container", contentAlignment: "center", padding: 8, flex: 1,
            background: "systemRed", cornerRadius: 8,
            children: [{ type: "text", content: "flex: 1", textStyle: "caption2", fontWeight: "bold", color: "#ffffff" }]},
          { type: "container", contentAlignment: "center", padding: 8, flex: 2,
            background: "systemBlue", cornerRadius: 8,
            children: [{ type: "text", content: "flex: 2", textStyle: "caption2", fontWeight: "bold", color: "#ffffff" }]},
          { type: "container", contentAlignment: "center", padding: 8, flex: 1,
            background: "systemGreen", cornerRadius: 8,
            children: [{ type: "text", content: "flex: 1", textStyle: "caption2", fontWeight: "bold", color: "#ffffff" }]},
        ]},

        { type: "hstack", spacing: 8, children: [
          { type: "button", label: "Demo Action", action: "demo_action",
            backgroundColor: "accent", color: "#ffffff", fontSize: 12, cornerRadius: 8 },
          { type: "spacer" },
          { type: "text", content: "Switch system theme to test", textStyle: "caption2", color: "secondaryLabel" },
        ]},
      ],
    },
  },
};
