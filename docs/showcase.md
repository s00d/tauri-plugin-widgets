---
title: Showcase
---

# Showcase

Presets from the visual test suite. Screenshots are the committed goldens — the same PNGs snapshot tests assert. Configs open in `<details>`; edit live in the playground.

<div class="doc-illust">

![Cross-platform widget showcase](/illustrations/platforms.jpg)

</div>

<!-- generated:showcase — do not edit; run `pnpm docs:generate` -->

## android-list

Sizes: large

### large

<ShotGrid case="android-list.large" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": null,
  "medium": null,
  "large": {
    "type": "list",
    "items": [
      {
        "text": "Alpha",
        "checked": true,
        "action": "row",
        "payload": "a"
      },
      {
        "text": "Beta",
        "checked": false,
        "action": "row",
        "payload": "b"
      },
      {
        "text": "Gamma",
        "checked": null,
        "action": null,
        "payload": null
      }
    ],
    "spacing": 2,
    "fontSize": 14,
    "color": null,
    "padding": null,
    "background": null,
    "cornerRadius": null,
    "opacity": null,
    "frame": null,
    "border": null,
    "shadow": null,
    "clipShape": null,
    "flex": null
  }
}
```

</details>

<Playground case="android-list.large" size="large" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiBudWxsLAogICJtZWRpdW0iOiBudWxsLAogICJsYXJnZSI6IHsKICAgICJ0eXBlIjogImxpc3QiLAogICAgIml0ZW1zIjogWwogICAgICB7CiAgICAgICAgInRleHQiOiAiQWxwaGEiLAogICAgICAgICJjaGVja2VkIjogdHJ1ZSwKICAgICAgICAiYWN0aW9uIjogInJvdyIsCiAgICAgICAgInBheWxvYWQiOiAiYSIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0ZXh0IjogIkJldGEiLAogICAgICAgICJjaGVja2VkIjogZmFsc2UsCiAgICAgICAgImFjdGlvbiI6ICJyb3ciLAogICAgICAgICJwYXlsb2FkIjogImIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJHYW1tYSIsCiAgICAgICAgImNoZWNrZWQiOiBudWxsLAogICAgICAgICJhY3Rpb24iOiBudWxsLAogICAgICAgICJwYXlsb2FkIjogbnVsbAogICAgICB9CiAgICBdLAogICAgInNwYWNpbmciOiAyLAogICAgImZvbnRTaXplIjogMTQsCiAgICAiY29sb3IiOiBudWxsLAogICAgInBhZGRpbmciOiBudWxsLAogICAgImJhY2tncm91bmQiOiBudWxsLAogICAgImNvcm5lclJhZGl1cyI6IG51bGwsCiAgICAib3BhY2l0eSI6IG51bGwsCiAgICAiZnJhbWUiOiBudWxsLAogICAgImJvcmRlciI6IG51bGwsCiAgICAic2hhZG93IjogbnVsbCwKICAgICJjbGlwU2hhcGUiOiBudWxsLAogICAgImZsZXgiOiBudWxsCiAgfQp9" />

## calculator

Sizes: small

### small

<ShotGrid case="calculator.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 6,
    "spacing": 2,
    "cornerRadius": 14,
    "background": "#1c1c1e",
    "children": [
      {
        "type": "hstack",
        "spacing": 4,
        "alignment": "center",
        "children": [
          {
            "type": "text",
            "content": " ",
            "fontSize": 10,
            "color": "#f09a36",
            "alignment": "leading",
            "lineLimit": 1
          },
          {
            "type": "spacer"
          },
          {
            "type": "text",
            "content": "0",
            "fontSize": 20,
            "fontWeight": "bold",
            "color": "#ffffff",
            "alignment": "trailing",
            "lineLimit": 1
          }
        ]
      },
      {
        "type": "divider",
        "color": "#3a3a3c",
        "thickness": 1
      },
      {
        "type": "grid",
        "columns": 4,
        "spacing": 2,
        "rowSpacing": 2,
        "children": [
          {
            "type": "button",
            "label": "C",
            "action": "calc:C",
            "backgroundColor": "#636366",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "±",
            "action": "calc:+-",
            "backgroundColor": "#636366",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "⌫",
            "action": "calc:BS",
            "backgroundColor": "#636366",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "÷",
            "action": "calc:/",
            "backgroundColor": "#f09a36",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "7",
            "action": "calc:7",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "8",
            "action": "calc:8",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "9",
            "action": "calc:9",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "×",
            "action": "calc:*",
            "backgroundColor": "#f09a36",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "4",
            "action": "calc:4",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "5",
            "action": "calc:5",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "6",
            "action": "calc:6",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "−",
            "action": "calc:-",
            "backgroundColor": "#f09a36",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "1",
            "action": "calc:1",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "2",
            "action": "calc:2",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "3",
            "action": "calc:3",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "+",
            "action": "calc:+",
            "backgroundColor": "#f09a36",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "%",
            "action": "calc:%",
            "backgroundColor": "#636366",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "0",
            "action": "calc:0",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": ".",
            "action": "calc:.",
            "backgroundColor": "#3a3a3c",
            "color": "#fff",
            "fontSize": 11,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          },
          {
            "type": "button",
            "label": "=",
            "action": "calc:=",
            "backgroundColor": "#f09a36",
            "color": "#fff",
            "fontSize": 10,
            "cornerRadius": 6,
            "padding": {
              "top": 1,
              "bottom": 1,
              "leading": 2,
              "trailing": 2
            }
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="calculator.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiA2LAogICAgInNwYWNpbmciOiAyLAogICAgImNvcm5lclJhZGl1cyI6IDE0LAogICAgImJhY2tncm91bmQiOiAiIzFjMWMxZSIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICJzcGFjaW5nIjogNCwKICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiICIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDEwLAogICAgICAgICAgICAiY29sb3IiOiAiI2YwOWEzNiIsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJzcGFjZXIiCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiMCIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDIwLAogICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgImNvbG9yIjogIiNmZmZmZmYiLAogICAgICAgICAgICAiYWxpZ25tZW50IjogInRyYWlsaW5nIiwKICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJkaXZpZGVyIiwKICAgICAgICAiY29sb3IiOiAiIzNhM2EzYyIsCiAgICAgICAgInRoaWNrbmVzcyI6IDEKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogImdyaWQiLAogICAgICAgICJjb2x1bW5zIjogNCwKICAgICAgICAic3BhY2luZyI6IDIsCiAgICAgICAgInJvd1NwYWNpbmciOiAyLAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIkMiLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6QyIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzYzNjM2NiIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiwrEiLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6Ky0iLAogICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiM2MzYzNjYiLAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDEwLAogICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNiwKICAgICAgICAgICAgInBhZGRpbmciOiB7CiAgICAgICAgICAgICAgInRvcCI6IDEsCiAgICAgICAgICAgICAgImJvdHRvbSI6IDEsCiAgICAgICAgICAgICAgImxlYWRpbmciOiAyLAogICAgICAgICAgICAgICJ0cmFpbGluZyI6IDIKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIuKMqyIsCiAgICAgICAgICAgICJhY3Rpb24iOiAiY2FsYzpCUyIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzYzNjM2NiIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiw7ciLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6LyIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiI2YwOWEzNiIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiNyIsCiAgICAgICAgICAgICJhY3Rpb24iOiAiY2FsYzo3IiwKICAgICAgICAgICAgImJhY2tncm91bmRDb2xvciI6ICIjM2EzYTNjIiwKICAgICAgICAgICAgImNvbG9yIjogIiNmZmYiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYsCiAgICAgICAgICAgICJwYWRkaW5nIjogewogICAgICAgICAgICAgICJ0b3AiOiAxLAogICAgICAgICAgICAgICJib3R0b20iOiAxLAogICAgICAgICAgICAgICJsZWFkaW5nIjogMiwKICAgICAgICAgICAgICAidHJhaWxpbmciOiAyCiAgICAgICAgICAgIH0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImJ1dHRvbiIsCiAgICAgICAgICAgICJsYWJlbCI6ICI4IiwKICAgICAgICAgICAgImFjdGlvbiI6ICJjYWxjOjgiLAogICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiMzYTNhM2MiLAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNiwKICAgICAgICAgICAgInBhZGRpbmciOiB7CiAgICAgICAgICAgICAgInRvcCI6IDEsCiAgICAgICAgICAgICAgImJvdHRvbSI6IDEsCiAgICAgICAgICAgICAgImxlYWRpbmciOiAyLAogICAgICAgICAgICAgICJ0cmFpbGluZyI6IDIKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIjkiLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6OSIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzNhM2EzYyIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiw5ciLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6KiIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiI2YwOWEzNiIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiNCIsCiAgICAgICAgICAgICJhY3Rpb24iOiAiY2FsYzo0IiwKICAgICAgICAgICAgImJhY2tncm91bmRDb2xvciI6ICIjM2EzYTNjIiwKICAgICAgICAgICAgImNvbG9yIjogIiNmZmYiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYsCiAgICAgICAgICAgICJwYWRkaW5nIjogewogICAgICAgICAgICAgICJ0b3AiOiAxLAogICAgICAgICAgICAgICJib3R0b20iOiAxLAogICAgICAgICAgICAgICJsZWFkaW5nIjogMiwKICAgICAgICAgICAgICAidHJhaWxpbmciOiAyCiAgICAgICAgICAgIH0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImJ1dHRvbiIsCiAgICAgICAgICAgICJsYWJlbCI6ICI1IiwKICAgICAgICAgICAgImFjdGlvbiI6ICJjYWxjOjUiLAogICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiMzYTNhM2MiLAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNiwKICAgICAgICAgICAgInBhZGRpbmciOiB7CiAgICAgICAgICAgICAgInRvcCI6IDEsCiAgICAgICAgICAgICAgImJvdHRvbSI6IDEsCiAgICAgICAgICAgICAgImxlYWRpbmciOiAyLAogICAgICAgICAgICAgICJ0cmFpbGluZyI6IDIKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIjYiLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6NiIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzNhM2EzYyIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAi4oiSIiwKICAgICAgICAgICAgImFjdGlvbiI6ICJjYWxjOi0iLAogICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiNmMDlhMzYiLAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDEwLAogICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNiwKICAgICAgICAgICAgInBhZGRpbmciOiB7CiAgICAgICAgICAgICAgInRvcCI6IDEsCiAgICAgICAgICAgICAgImJvdHRvbSI6IDEsCiAgICAgICAgICAgICAgImxlYWRpbmciOiAyLAogICAgICAgICAgICAgICJ0cmFpbGluZyI6IDIKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIjEiLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6MSIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzNhM2EzYyIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiMiIsCiAgICAgICAgICAgICJhY3Rpb24iOiAiY2FsYzoyIiwKICAgICAgICAgICAgImJhY2tncm91bmRDb2xvciI6ICIjM2EzYTNjIiwKICAgICAgICAgICAgImNvbG9yIjogIiNmZmYiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYsCiAgICAgICAgICAgICJwYWRkaW5nIjogewogICAgICAgICAgICAgICJ0b3AiOiAxLAogICAgICAgICAgICAgICJib3R0b20iOiAxLAogICAgICAgICAgICAgICJsZWFkaW5nIjogMiwKICAgICAgICAgICAgICAidHJhaWxpbmciOiAyCiAgICAgICAgICAgIH0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImJ1dHRvbiIsCiAgICAgICAgICAgICJsYWJlbCI6ICIzIiwKICAgICAgICAgICAgImFjdGlvbiI6ICJjYWxjOjMiLAogICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiMzYTNhM2MiLAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNiwKICAgICAgICAgICAgInBhZGRpbmciOiB7CiAgICAgICAgICAgICAgInRvcCI6IDEsCiAgICAgICAgICAgICAgImJvdHRvbSI6IDEsCiAgICAgICAgICAgICAgImxlYWRpbmciOiAyLAogICAgICAgICAgICAgICJ0cmFpbGluZyI6IDIKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIisiLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6KyIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiI2YwOWEzNiIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiJSIsCiAgICAgICAgICAgICJhY3Rpb24iOiAiY2FsYzolIiwKICAgICAgICAgICAgImJhY2tncm91bmRDb2xvciI6ICIjNjM2MzY2IiwKICAgICAgICAgICAgImNvbG9yIjogIiNmZmYiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYsCiAgICAgICAgICAgICJwYWRkaW5nIjogewogICAgICAgICAgICAgICJ0b3AiOiAxLAogICAgICAgICAgICAgICJib3R0b20iOiAxLAogICAgICAgICAgICAgICJsZWFkaW5nIjogMiwKICAgICAgICAgICAgICAidHJhaWxpbmciOiAyCiAgICAgICAgICAgIH0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImJ1dHRvbiIsCiAgICAgICAgICAgICJsYWJlbCI6ICIwIiwKICAgICAgICAgICAgImFjdGlvbiI6ICJjYWxjOjAiLAogICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiMzYTNhM2MiLAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNiwKICAgICAgICAgICAgInBhZGRpbmciOiB7CiAgICAgICAgICAgICAgInRvcCI6IDEsCiAgICAgICAgICAgICAgImJvdHRvbSI6IDEsCiAgICAgICAgICAgICAgImxlYWRpbmciOiAyLAogICAgICAgICAgICAgICJ0cmFpbGluZyI6IDIKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIi4iLAogICAgICAgICAgICAiYWN0aW9uIjogImNhbGM6LiIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzNhM2EzYyIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2LAogICAgICAgICAgICAicGFkZGluZyI6IHsKICAgICAgICAgICAgICAidG9wIjogMSwKICAgICAgICAgICAgICAiYm90dG9tIjogMSwKICAgICAgICAgICAgICAibGVhZGluZyI6IDIsCiAgICAgICAgICAgICAgInRyYWlsaW5nIjogMgogICAgICAgICAgICB9CiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAibGFiZWwiOiAiPSIsCiAgICAgICAgICAgICJhY3Rpb24iOiAiY2FsYzo9IiwKICAgICAgICAgICAgImJhY2tncm91bmRDb2xvciI6ICIjZjA5YTM2IiwKICAgICAgICAgICAgImNvbG9yIjogIiNmZmYiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYsCiAgICAgICAgICAgICJwYWRkaW5nIjogewogICAgICAgICAgICAgICJ0b3AiOiAxLAogICAgICAgICAgICAgICJib3R0b20iOiAxLAogICAgICAgICAgICAgICJsZWFkaW5nIjogMiwKICAgICAgICAgICAgICAidHJhaWxpbmciOiAyCiAgICAgICAgICAgIH0KICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## canvas

Sizes: small

### small

<ShotGrid case="canvas.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "canvas",
    "width": 120,
    "height": 120,
    "elements": [
      {
        "draw": "circle",
        "cx": 60,
        "cy": 60,
        "r": 40,
        "fill": "#1a1a2e",
        "stroke": "#fff",
        "strokeWidth": 2
      }
    ],
    "padding": null,
    "background": null,
    "cornerRadius": null,
    "opacity": null,
    "frame": null,
    "border": null,
    "shadow": null,
    "clipShape": null,
    "flex": null
  },
  "medium": null,
  "large": null
}
```

</details>

<Playground case="canvas.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJjYW52YXMiLAogICAgIndpZHRoIjogMTIwLAogICAgImhlaWdodCI6IDEyMCwKICAgICJlbGVtZW50cyI6IFsKICAgICAgewogICAgICAgICJkcmF3IjogImNpcmNsZSIsCiAgICAgICAgImN4IjogNjAsCiAgICAgICAgImN5IjogNjAsCiAgICAgICAgInIiOiA0MCwKICAgICAgICAiZmlsbCI6ICIjMWExYTJlIiwKICAgICAgICAic3Ryb2tlIjogIiNmZmYiLAogICAgICAgICJzdHJva2VXaWR0aCI6IDIKICAgICAgfQogICAgXSwKICAgICJwYWRkaW5nIjogbnVsbCwKICAgICJiYWNrZ3JvdW5kIjogbnVsbCwKICAgICJjb3JuZXJSYWRpdXMiOiBudWxsLAogICAgIm9wYWNpdHkiOiBudWxsLAogICAgImZyYW1lIjogbnVsbCwKICAgICJib3JkZXIiOiBudWxsLAogICAgInNoYWRvdyI6IG51bGwsCiAgICAiY2xpcFNoYXBlIjogbnVsbCwKICAgICJmbGV4IjogbnVsbAogIH0sCiAgIm1lZGl1bSI6IG51bGwsCiAgImxhcmdlIjogbnVsbAp9" />

## canvas-draws

Sizes: small

### small

<ShotGrid case="canvas-draws.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "canvas",
    "width": 146,
    "height": 146,
    "background": "#0b1220",
    "cornerRadius": 12,
    "elements": [
      {
        "draw": "rect",
        "x": 12,
        "y": 12,
        "width": 48,
        "height": 32,
        "fill": "#1e293b",
        "stroke": "#38bdf8",
        "strokeWidth": 2,
        "cornerRadius": 6
      },
      {
        "draw": "circle",
        "cx": 110,
        "cy": 36,
        "r": 18,
        "fill": "#22c55e",
        "stroke": "#86efac",
        "strokeWidth": 2
      },
      {
        "draw": "arc",
        "cx": 48,
        "cy": 90,
        "r": 28,
        "startAngle": -90,
        "endAngle": 120,
        "stroke": "#f59e0b",
        "strokeWidth": 4
      },
      {
        "draw": "path",
        "d": "M90 70 L130 110 L90 110 Z",
        "fill": "#a855f7",
        "stroke": "#e9d5ff",
        "strokeWidth": 1
      },
      {
        "draw": "text",
        "x": 73,
        "y": 136,
        "content": "draw",
        "fontSize": 12,
        "color": "#e2e8f0",
        "anchor": "middle"
      }
    ]
  }
}
```

</details>

<Playground case="canvas-draws.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJjYW52YXMiLAogICAgIndpZHRoIjogMTQ2LAogICAgImhlaWdodCI6IDE0NiwKICAgICJiYWNrZ3JvdW5kIjogIiMwYjEyMjAiLAogICAgImNvcm5lclJhZGl1cyI6IDEyLAogICAgImVsZW1lbnRzIjogWwogICAgICB7CiAgICAgICAgImRyYXciOiAicmVjdCIsCiAgICAgICAgIngiOiAxMiwKICAgICAgICAieSI6IDEyLAogICAgICAgICJ3aWR0aCI6IDQ4LAogICAgICAgICJoZWlnaHQiOiAzMiwKICAgICAgICAiZmlsbCI6ICIjMWUyOTNiIiwKICAgICAgICAic3Ryb2tlIjogIiMzOGJkZjgiLAogICAgICAgICJzdHJva2VXaWR0aCI6IDIsCiAgICAgICAgImNvcm5lclJhZGl1cyI6IDYKICAgICAgfSwKICAgICAgewogICAgICAgICJkcmF3IjogImNpcmNsZSIsCiAgICAgICAgImN4IjogMTEwLAogICAgICAgICJjeSI6IDM2LAogICAgICAgICJyIjogMTgsCiAgICAgICAgImZpbGwiOiAiIzIyYzU1ZSIsCiAgICAgICAgInN0cm9rZSI6ICIjODZlZmFjIiwKICAgICAgICAic3Ryb2tlV2lkdGgiOiAyCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAiZHJhdyI6ICJhcmMiLAogICAgICAgICJjeCI6IDQ4LAogICAgICAgICJjeSI6IDkwLAogICAgICAgICJyIjogMjgsCiAgICAgICAgInN0YXJ0QW5nbGUiOiAtOTAsCiAgICAgICAgImVuZEFuZ2xlIjogMTIwLAogICAgICAgICJzdHJva2UiOiAiI2Y1OWUwYiIsCiAgICAgICAgInN0cm9rZVdpZHRoIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgImRyYXciOiAicGF0aCIsCiAgICAgICAgImQiOiAiTTkwIDcwIEwxMzAgMTEwIEw5MCAxMTAgWiIsCiAgICAgICAgImZpbGwiOiAiI2E4NTVmNyIsCiAgICAgICAgInN0cm9rZSI6ICIjZTlkNWZmIiwKICAgICAgICAic3Ryb2tlV2lkdGgiOiAxCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAiZHJhdyI6ICJ0ZXh0IiwKICAgICAgICAieCI6IDczLAogICAgICAgICJ5IjogMTM2LAogICAgICAgICJjb250ZW50IjogImRyYXciLAogICAgICAgICJmb250U2l6ZSI6IDEyLAogICAgICAgICJjb2xvciI6ICIjZTJlOGYwIiwKICAgICAgICAiYW5jaG9yIjogIm1pZGRsZSIKICAgICAgfQogICAgXQogIH0KfQ==" />

## canvas-scale

Sizes: small

### small

<ShotGrid case="canvas-scale.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 4,
    "background": "#111",
    "alignment": "center",
    "children": [
      {
        "type": "canvas",
        "width": 60,
        "height": 60,
        "elements": [
          {
            "draw": "circle",
            "cx": 30,
            "cy": 30,
            "r": 25,
            "fill": "#1e40af",
            "stroke": "#fff",
            "strokeWidth": 2
          },
          {
            "draw": "line",
            "x1": 30,
            "y1": 30,
            "x2": 30,
            "y2": 10,
            "stroke": "#fff",
            "strokeWidth": 2
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="canvas-scale.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiA0LAogICAgImJhY2tncm91bmQiOiAiIzExMSIsCiAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJjYW52YXMiLAogICAgICAgICJ3aWR0aCI6IDYwLAogICAgICAgICJoZWlnaHQiOiA2MCwKICAgICAgICAiZWxlbWVudHMiOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJkcmF3IjogImNpcmNsZSIsCiAgICAgICAgICAgICJjeCI6IDMwLAogICAgICAgICAgICAiY3kiOiAzMCwKICAgICAgICAgICAgInIiOiAyNSwKICAgICAgICAgICAgImZpbGwiOiAiIzFlNDBhZiIsCiAgICAgICAgICAgICJzdHJva2UiOiAiI2ZmZiIsCiAgICAgICAgICAgICJzdHJva2VXaWR0aCI6IDIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJkcmF3IjogImxpbmUiLAogICAgICAgICAgICAieDEiOiAzMCwKICAgICAgICAgICAgInkxIjogMzAsCiAgICAgICAgICAgICJ4MiI6IDMwLAogICAgICAgICAgICAieTIiOiAxMCwKICAgICAgICAgICAgInN0cm9rZSI6ICIjZmZmIiwKICAgICAgICAgICAgInN0cm9rZVdpZHRoIjogMgogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgXQogIH0KfQ==" />

## chart-mix

Sizes: medium

### medium

<ShotGrid case="chart-mix.medium" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "medium": {
    "type": "hstack",
    "padding": 10,
    "spacing": 10,
    "cornerRadius": 14,
    "background": "#111827",
    "alignment": "center",
    "children": [
      {
        "type": "vstack",
        "spacing": 4,
        "flex": 1,
        "children": [
          {
            "type": "text",
            "content": "Bar",
            "fontSize": 11,
            "color": "#94a3b8"
          },
          {
            "type": "chart",
            "chartType": "bar",
            "tint": "#22c55e",
            "chartData": [
              {
                "label": "M",
                "value": 3,
                "color": "#22c55e"
              },
              {
                "label": "T",
                "value": 5,
                "color": "#3b82f6"
              },
              {
                "label": "W",
                "value": 2,
                "color": "#f59e0b"
              },
              {
                "label": "T",
                "value": 7,
                "color": "#ef4444"
              }
            ]
          }
        ]
      },
      {
        "type": "vstack",
        "spacing": 4,
        "flex": 1,
        "children": [
          {
            "type": "text",
            "content": "Line",
            "fontSize": 11,
            "color": "#94a3b8"
          },
          {
            "type": "chart",
            "chartType": "line",
            "tint": "#38bdf8",
            "chartData": [
              {
                "label": "1",
                "value": 2
              },
              {
                "label": "2",
                "value": 4
              },
              {
                "label": "3",
                "value": 3
              },
              {
                "label": "4",
                "value": 6
              },
              {
                "label": "5",
                "value": 5
              }
            ]
          }
        ]
      },
      {
        "type": "vstack",
        "spacing": 4,
        "alignment": "center",
        "flex": 1,
        "children": [
          {
            "type": "text",
            "content": "Pie",
            "fontSize": 11,
            "color": "#94a3b8"
          },
          {
            "type": "chart",
            "chartType": "pie",
            "chartData": [
              {
                "label": "A",
                "value": 40,
                "color": "#3b82f6"
              },
              {
                "label": "B",
                "value": 35,
                "color": "#22c55e"
              },
              {
                "label": "C",
                "value": 25,
                "color": "#f97316"
              }
            ]
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="chart-mix.medium" size="medium" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibWVkaXVtIjogewogICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICJwYWRkaW5nIjogMTAsCiAgICAic3BhY2luZyI6IDEwLAogICAgImNvcm5lclJhZGl1cyI6IDE0LAogICAgImJhY2tncm91bmQiOiAiIzExMTgyNyIsCiAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICJzcGFjaW5nIjogNCwKICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiQmFyIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiY2hhcnQiLAogICAgICAgICAgICAiY2hhcnRUeXBlIjogImJhciIsCiAgICAgICAgICAgICJ0aW50IjogIiMyMmM1NWUiLAogICAgICAgICAgICAiY2hhcnREYXRhIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJsYWJlbCI6ICJNIiwKICAgICAgICAgICAgICAgICJ2YWx1ZSI6IDMsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzIyYzU1ZSIKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJsYWJlbCI6ICJUIiwKICAgICAgICAgICAgICAgICJ2YWx1ZSI6IDUsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzNiODJmNiIKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJsYWJlbCI6ICJXIiwKICAgICAgICAgICAgICAgICJ2YWx1ZSI6IDIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiI2Y1OWUwYiIKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJsYWJlbCI6ICJUIiwKICAgICAgICAgICAgICAgICJ2YWx1ZSI6IDcsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiI2VmNDQ0NCIKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICJzcGFjaW5nIjogNCwKICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiTGluZSIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAiY29sb3IiOiAiIzk0YTNiOCIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImNoYXJ0IiwKICAgICAgICAgICAgImNoYXJ0VHlwZSI6ICJsaW5lIiwKICAgICAgICAgICAgInRpbnQiOiAiIzM4YmRmOCIsCiAgICAgICAgICAgICJjaGFydERhdGEiOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgImxhYmVsIjogIjEiLAogICAgICAgICAgICAgICAgInZhbHVlIjogMgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgImxhYmVsIjogIjIiLAogICAgICAgICAgICAgICAgInZhbHVlIjogNAogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgImxhYmVsIjogIjMiLAogICAgICAgICAgICAgICAgInZhbHVlIjogMwogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgImxhYmVsIjogIjQiLAogICAgICAgICAgICAgICAgInZhbHVlIjogNgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgImxhYmVsIjogIjUiLAogICAgICAgICAgICAgICAgInZhbHVlIjogNQogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgInNwYWNpbmciOiA0LAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiUGllIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiY2hhcnQiLAogICAgICAgICAgICAiY2hhcnRUeXBlIjogInBpZSIsCiAgICAgICAgICAgICJjaGFydERhdGEiOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgImxhYmVsIjogIkEiLAogICAgICAgICAgICAgICAgInZhbHVlIjogNDAsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzNiODJmNiIKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJsYWJlbCI6ICJCIiwKICAgICAgICAgICAgICAgICJ2YWx1ZSI6IDM1LAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMyMmM1NWUiCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAibGFiZWwiOiAiQyIsCiAgICAgICAgICAgICAgICAidmFsdWUiOiAyNSwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjZjk3MzE2IgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgXQogIH0KfQ==" />

## container-card

Sizes: small

### small

<ShotGrid case="container-card.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "container",
    "contentAlignment": "center",
    "padding": 12,
    "cornerRadius": 16,
    "background": "#1C1C1E",
    "frame": {
      "width": 146,
      "height": 146
    },
    "children": [
      {
        "type": "vstack",
        "spacing": 8,
        "alignment": "center",
        "children": [
          {
            "type": "shape",
            "shapeType": "circle",
            "size": 36,
            "fill": "#3b82f6"
          },
          {
            "type": "text",
            "content": "Card",
            "fontSize": 15,
            "fontWeight": "semibold",
            "color": "#FFFFFF"
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="container-card.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJjb250YWluZXIiLAogICAgImNvbnRlbnRBbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICJwYWRkaW5nIjogMTIsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6ICIjMUMxQzFFIiwKICAgICJmcmFtZSI6IHsKICAgICAgIndpZHRoIjogMTQ2LAogICAgICAiaGVpZ2h0IjogMTQ2CiAgICB9LAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDgsCiAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAic2hhcGVUeXBlIjogImNpcmNsZSIsCiAgICAgICAgICAgICJzaXplIjogMzYsCiAgICAgICAgICAgICJmaWxsIjogIiMzYjgyZjYiCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiQ2FyZCIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDE1LAogICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgXQogIH0KfQ==" />

## date-timer

Sizes: medium

### medium

<ShotGrid case="date-timer.medium" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "medium": {
    "type": "vstack",
    "padding": 12,
    "spacing": 8,
    "cornerRadius": 14,
    "background": "#0f172a",
    "children": [
      {
        "type": "text",
        "content": "Schedule",
        "fontSize": 15,
        "fontWeight": "semibold",
        "color": "#f8fafc"
      },
      {
        "type": "hstack",
        "spacing": 16,
        "alignment": "center",
        "children": [
          {
            "type": "vstack",
            "spacing": 4,
            "alignment": "leading",
            "children": [
              {
                "type": "text",
                "content": "Time",
                "fontSize": 11,
                "color": "#94a3b8"
              },
              {
                "type": "date",
                "date": "2099-06-15T14:30:00Z",
                "dateStyle": "time",
                "fontSize": 14,
                "color": "#e2e8f0"
              }
            ]
          },
          {
            "type": "vstack",
            "spacing": 4,
            "alignment": "leading",
            "children": [
              {
                "type": "text",
                "content": "Relative",
                "fontSize": 11,
                "color": "#94a3b8"
              },
              {
                "type": "date",
                "date": "2099-12-25T00:00:00Z",
                "dateStyle": "relative",
                "fontSize": 14,
                "color": "#e2e8f0"
              }
            ]
          },
          {
            "type": "vstack",
            "spacing": 4,
            "alignment": "leading",
            "children": [
              {
                "type": "text",
                "content": "Countdown",
                "fontSize": 11,
                "color": "#94a3b8"
              },
              {
                "type": "timer",
                "targetDate": "2099-01-01T00:00:00Z",
                "counting": "down",
                "fontSize": 14,
                "fontWeight": "semibold",
                "color": "#38bdf8"
              }
            ]
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="date-timer.medium" size="medium" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibWVkaXVtIjogewogICAgInR5cGUiOiAidnN0YWNrIiwKICAgICJwYWRkaW5nIjogMTIsCiAgICAic3BhY2luZyI6IDgsCiAgICAiY29ybmVyUmFkaXVzIjogMTQsCiAgICAiYmFja2dyb3VuZCI6ICIjMGYxNzJhIiwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICJjb250ZW50IjogIlNjaGVkdWxlIiwKICAgICAgICAiZm9udFNpemUiOiAxNSwKICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgImNvbG9yIjogIiNmOGZhZmMiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICJzcGFjaW5nIjogMTYsCiAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiA0LAogICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICJUaW1lIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiM5NGEzYjgiCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJkYXRlIiwKICAgICAgICAgICAgICAgICJkYXRlIjogIjIwOTktMDYtMTVUMTQ6MzA6MDBaIiwKICAgICAgICAgICAgICAgICJkYXRlU3R5bGUiOiAidGltZSIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxNCwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjZTJlOGYwIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiA0LAogICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICJSZWxhdGl2ZSIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAiZGF0ZSIsCiAgICAgICAgICAgICAgICAiZGF0ZSI6ICIyMDk5LTEyLTI1VDAwOjAwOjAwWiIsCiAgICAgICAgICAgICAgICAiZGF0ZVN0eWxlIjogInJlbGF0aXZlIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDE0LAogICAgICAgICAgICAgICAgImNvbG9yIjogIiNlMmU4ZjAiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDQsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIkNvdW50ZG93biIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGltZXIiLAogICAgICAgICAgICAgICAgInRhcmdldERhdGUiOiAiMjA5OS0wMS0wMVQwMDowMDowMFoiLAogICAgICAgICAgICAgICAgImNvdW50aW5nIjogImRvd24iLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTQsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzM4YmRmOCIKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## fitness

Sizes: small, medium

### small

<ShotGrid case="fitness.small" />

### medium

<ShotGrid case="fitness.medium" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 12,
    "spacing": 8,
    "cornerRadius": 16,
    "background": "#0d1117",
    "children": [
      {
        "type": "text",
        "content": "Activity",
        "fontSize": 15,
        "fontWeight": "semibold",
        "color": "#fff"
      },
      {
        "type": "gauge",
        "value": 0.72,
        "min": 0,
        "max": 1,
        "currentValueLabel": "72%",
        "label": "Steps",
        "gaugeStyle": "circular",
        "tint": "#ff6b6b"
      },
      {
        "type": "hstack",
        "spacing": 8,
        "children": [
          {
            "type": "progress",
            "value": 0.45,
            "tint": "#4ecdc4",
            "label": "Cal",
            "color": "#94a3b8"
          },
          {
            "type": "progress",
            "value": 0.88,
            "tint": "#ffe66d",
            "label": "H2O",
            "color": "#94a3b8"
          }
        ]
      }
    ]
  },
  "medium": {
    "type": "hstack",
    "padding": 14,
    "spacing": 16,
    "cornerRadius": 16,
    "background": "#0d1117",
    "children": [
      {
        "type": "vstack",
        "spacing": 6,
        "alignment": "center",
        "children": [
          {
            "type": "gauge",
            "value": 0.72,
            "min": 0,
            "max": 1,
            "currentValueLabel": "72%",
            "label": "Steps",
            "gaugeStyle": "circular",
            "tint": "#ff6b6b"
          },
          {
            "type": "text",
            "content": "7,200 / 10k",
            "fontSize": 11,
            "color": "#94a3b8"
          }
        ]
      },
      {
        "type": "divider",
        "color": "#334155"
      },
      {
        "type": "vstack",
        "spacing": 6,
        "children": [
          {
            "type": "progress",
            "value": 0.45,
            "tint": "#4ecdc4",
            "label": "Calories",
            "color": "#94a3b8"
          },
          {
            "type": "progress",
            "value": 0.88,
            "tint": "#ffe66d",
            "label": "Water",
            "color": "#94a3b8"
          },
          {
            "type": "progress",
            "value": 0.33,
            "tint": "#ff6b6b",
            "label": "Exercise",
            "color": "#94a3b8"
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="fitness.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJzcGFjaW5nIjogOCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNiwKICAgICJiYWNrZ3JvdW5kIjogIiMwZDExMTciLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiQWN0aXZpdHkiLAogICAgICAgICJmb250U2l6ZSI6IDE1LAogICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAiY29sb3IiOiAiI2ZmZiIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogImdhdWdlIiwKICAgICAgICAidmFsdWUiOiAwLjcyLAogICAgICAgICJtaW4iOiAwLAogICAgICAgICJtYXgiOiAxLAogICAgICAgICJjdXJyZW50VmFsdWVMYWJlbCI6ICI3MiUiLAogICAgICAgICJsYWJlbCI6ICJTdGVwcyIsCiAgICAgICAgImdhdWdlU3R5bGUiOiAiY2lyY3VsYXIiLAogICAgICAgICJ0aW50IjogIiNmZjZiNmIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICJzcGFjaW5nIjogOCwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInByb2dyZXNzIiwKICAgICAgICAgICAgInZhbHVlIjogMC40NSwKICAgICAgICAgICAgInRpbnQiOiAiIzRlY2RjNCIsCiAgICAgICAgICAgICJsYWJlbCI6ICJDYWwiLAogICAgICAgICAgICAiY29sb3IiOiAiIzk0YTNiOCIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInByb2dyZXNzIiwKICAgICAgICAgICAgInZhbHVlIjogMC44OCwKICAgICAgICAgICAgInRpbnQiOiAiI2ZmZTY2ZCIsCiAgICAgICAgICAgICJsYWJlbCI6ICJIMk8iLAogICAgICAgICAgICAiY29sb3IiOiAiIzk0YTNiOCIKICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0KICAgIF0KICB9LAogICJtZWRpdW0iOiB7CiAgICAidHlwZSI6ICJoc3RhY2siLAogICAgInBhZGRpbmciOiAxNCwKICAgICJzcGFjaW5nIjogMTYsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6ICIjMGQxMTE3IiwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgInNwYWNpbmciOiA2LAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImdhdWdlIiwKICAgICAgICAgICAgInZhbHVlIjogMC43MiwKICAgICAgICAgICAgIm1pbiI6IDAsCiAgICAgICAgICAgICJtYXgiOiAxLAogICAgICAgICAgICAiY3VycmVudFZhbHVlTGFiZWwiOiAiNzIlIiwKICAgICAgICAgICAgImxhYmVsIjogIlN0ZXBzIiwKICAgICAgICAgICAgImdhdWdlU3R5bGUiOiAiY2lyY3VsYXIiLAogICAgICAgICAgICAidGludCI6ICIjZmY2YjZiIgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICJjb250ZW50IjogIjcsMjAwIC8gMTBrIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogImRpdmlkZXIiLAogICAgICAgICJjb2xvciI6ICIjMzM0MTU1IgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJwcm9ncmVzcyIsCiAgICAgICAgICAgICJ2YWx1ZSI6IDAuNDUsCiAgICAgICAgICAgICJ0aW50IjogIiM0ZWNkYzQiLAogICAgICAgICAgICAibGFiZWwiOiAiQ2Fsb3JpZXMiLAogICAgICAgICAgICAiY29sb3IiOiAiIzk0YTNiOCIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInByb2dyZXNzIiwKICAgICAgICAgICAgInZhbHVlIjogMC44OCwKICAgICAgICAgICAgInRpbnQiOiAiI2ZmZTY2ZCIsCiAgICAgICAgICAgICJsYWJlbCI6ICJXYXRlciIsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICAgICAidmFsdWUiOiAwLjMzLAogICAgICAgICAgICAidGludCI6ICIjZmY2YjZiIiwKICAgICAgICAgICAgImxhYmVsIjogIkV4ZXJjaXNlIiwKICAgICAgICAgICAgImNvbG9yIjogIiM5NGEzYjgiCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9CiAgICBdCiAgfQp9" />

## flex-stroke-grid

Sizes: medium

### medium

<ShotGrid case="flex-stroke-grid.medium" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "medium": {
    "type": "vstack",
    "padding": {
      "top": 10,
      "leading": 12,
      "bottom": 10,
      "trailing": 12
    },
    "spacing": 8,
    "cornerRadius": 14,
    "background": "#0f172a",
    "children": [
      {
        "type": "text",
        "content": "Flex + stroke",
        "fontSize": 13,
        "fontWeight": "semibold",
        "color": "#f8fafc",
        "frame": {
          "maxWidth": 200
        }
      },
      {
        "type": "grid",
        "columns": 3,
        "spacing": 8,
        "children": [
          {
            "type": "shape",
            "shapeType": "rectangle",
            "size": 36,
            "fill": "#1e293b",
            "stroke": "#22c55e",
            "strokeWidth": 2,
            "cornerRadius": 8,
            "flex": 1
          },
          {
            "type": "shape",
            "shapeType": "circle",
            "size": 36,
            "fill": "transparent",
            "stroke": "#38bdf8",
            "strokeWidth": 3,
            "flex": 1
          },
          {
            "type": "shape",
            "shapeType": "capsule",
            "size": 18,
            "fill": "#312e81",
            "stroke": "#a855f7",
            "strokeWidth": 2,
            "flex": 1
          },
          {
            "type": "text",
            "content": "A",
            "fontSize": 12,
            "color": "#94a3b8",
            "alignment": "center",
            "flex": 1
          },
          {
            "type": "text",
            "content": "B",
            "fontSize": 12,
            "color": "#94a3b8",
            "alignment": "center",
            "flex": 1
          },
          {
            "type": "text",
            "content": "C",
            "fontSize": 12,
            "color": "#94a3b8",
            "alignment": "center",
            "flex": 1
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="flex-stroke-grid.medium" size="medium" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibWVkaXVtIjogewogICAgInR5cGUiOiAidnN0YWNrIiwKICAgICJwYWRkaW5nIjogewogICAgICAidG9wIjogMTAsCiAgICAgICJsZWFkaW5nIjogMTIsCiAgICAgICJib3R0b20iOiAxMCwKICAgICAgInRyYWlsaW5nIjogMTIKICAgIH0sCiAgICAic3BhY2luZyI6IDgsCiAgICAiY29ybmVyUmFkaXVzIjogMTQsCiAgICAiYmFja2dyb3VuZCI6ICIjMGYxNzJhIiwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICJjb250ZW50IjogIkZsZXggKyBzdHJva2UiLAogICAgICAgICJmb250U2l6ZSI6IDEzLAogICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAiY29sb3IiOiAiI2Y4ZmFmYyIsCiAgICAgICAgImZyYW1lIjogewogICAgICAgICAgIm1heFdpZHRoIjogMjAwCiAgICAgICAgfQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiZ3JpZCIsCiAgICAgICAgImNvbHVtbnMiOiAzLAogICAgICAgICJzcGFjaW5nIjogOCwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAic2l6ZSI6IDM2LAogICAgICAgICAgICAiZmlsbCI6ICIjMWUyOTNiIiwKICAgICAgICAgICAgInN0cm9rZSI6ICIjMjJjNTVlIiwKICAgICAgICAgICAgInN0cm9rZVdpZHRoIjogMiwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDgsCiAgICAgICAgICAgICJmbGV4IjogMQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAic2hhcGVUeXBlIjogImNpcmNsZSIsCiAgICAgICAgICAgICJzaXplIjogMzYsCiAgICAgICAgICAgICJmaWxsIjogInRyYW5zcGFyZW50IiwKICAgICAgICAgICAgInN0cm9rZSI6ICIjMzhiZGY4IiwKICAgICAgICAgICAgInN0cm9rZVdpZHRoIjogMywKICAgICAgICAgICAgImZsZXgiOiAxCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICJzaGFwZVR5cGUiOiAiY2Fwc3VsZSIsCiAgICAgICAgICAgICJzaXplIjogMTgsCiAgICAgICAgICAgICJmaWxsIjogIiMzMTJlODEiLAogICAgICAgICAgICAic3Ryb2tlIjogIiNhODU1ZjciLAogICAgICAgICAgICAic3Ryb2tlV2lkdGgiOiAyLAogICAgICAgICAgICAiZmxleCI6IDEKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAiY29udGVudCI6ICJBIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTIsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiZmxleCI6IDEKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAiY29udGVudCI6ICJCIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTIsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiZmxleCI6IDEKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAiY29udGVudCI6ICJDIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTIsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiZmxleCI6IDEKICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## gauge-square

Sizes: small

### small

<ShotGrid case="gauge-square.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 8,
    "background": "#0f172a",
    "alignment": "center",
    "children": [
      {
        "type": "gauge",
        "value": 0.72,
        "min": 0,
        "max": 1,
        "label": "CPU",
        "currentValueLabel": "72%",
        "tint": "#7aa2f7",
        "gaugeStyle": "circular"
      }
    ]
  }
}
```

</details>

<Playground case="gauge-square.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiA4LAogICAgImJhY2tncm91bmQiOiAiIzBmMTcyYSIsCiAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJnYXVnZSIsCiAgICAgICAgInZhbHVlIjogMC43MiwKICAgICAgICAibWluIjogMCwKICAgICAgICAibWF4IjogMSwKICAgICAgICAibGFiZWwiOiAiQ1BVIiwKICAgICAgICAiY3VycmVudFZhbHVlTGFiZWwiOiAiNzIlIiwKICAgICAgICAidGludCI6ICIjN2FhMmY3IiwKICAgICAgICAiZ2F1Z2VTdHlsZSI6ICJjaXJjdWxhciIKICAgICAgfQogICAgXQogIH0KfQ==" />

## hstack-divider

Sizes: medium

### medium

<ShotGrid case="hstack-divider.medium" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "medium": {
    "type": "hstack",
    "spacing": 8,
    "padding": 12,
    "background": "#0f172a",
    "children": [
      {
        "type": "text",
        "content": "L",
        "fontSize": 16,
        "color": "#fff"
      },
      {
        "type": "divider",
        "thickness": 2,
        "color": "#64748b"
      },
      {
        "type": "text",
        "content": "R",
        "fontSize": 16,
        "color": "#fff"
      }
    ]
  }
}
```

</details>

<Playground case="hstack-divider.medium" size="medium" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibWVkaXVtIjogewogICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICJzcGFjaW5nIjogOCwKICAgICJwYWRkaW5nIjogMTIsCiAgICAiYmFja2dyb3VuZCI6ICIjMGYxNzJhIiwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICJjb250ZW50IjogIkwiLAogICAgICAgICJmb250U2l6ZSI6IDE2LAogICAgICAgICJjb2xvciI6ICIjZmZmIgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiZGl2aWRlciIsCiAgICAgICAgInRoaWNrbmVzcyI6IDIsCiAgICAgICAgImNvbG9yIjogIiM2NDc0OGIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICJSIiwKICAgICAgICAiZm9udFNpemUiOiAxNiwKICAgICAgICAiY29sb3IiOiAiI2ZmZiIKICAgICAgfQogICAgXQogIH0KfQ==" />

## image-sources

Sizes: medium

### medium

<ShotGrid case="image-sources.medium" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "medium": {
    "type": "hstack",
    "padding": 12,
    "spacing": 16,
    "cornerRadius": 14,
    "background": "#0f172a",
    "alignment": "center",
    "children": [
      {
        "type": "vstack",
        "spacing": 4,
        "alignment": "center",
        "children": [
          {
            "type": "text",
            "content": "SF",
            "fontSize": 10,
            "color": "#94a3b8"
          },
          {
            "type": "image",
            "systemName": "star.fill",
            "size": 32,
            "color": "#fbbf24"
          }
        ]
      },
      {
        "type": "vstack",
        "spacing": 4,
        "alignment": "center",
        "children": [
          {
            "type": "text",
            "content": "Data",
            "fontSize": 10,
            "color": "#94a3b8"
          },
          {
            "type": "image",
            "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGO4o6b2HwAFMgIozFXxQQAAAABJRU5ErkJggg==",
            "size": 32,
            "contentMode": "fill",
            "clipShape": "circle"
          }
        ]
      },
      {
        "type": "vstack",
        "spacing": 4,
        "alignment": "center",
        "children": [
          {
            "type": "text",
            "content": "URL",
            "fontSize": 10,
            "color": "#94a3b8"
          },
          {
            "type": "image",
            "url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGOw2PvjPwAGCwLtgxz7OwAAAABJRU5ErkJggg==",
            "size": 32,
            "contentMode": "fit"
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="image-sources.medium" size="medium" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibWVkaXVtIjogewogICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICJwYWRkaW5nIjogMTIsCiAgICAic3BhY2luZyI6IDE2LAogICAgImNvcm5lclJhZGl1cyI6IDE0LAogICAgImJhY2tncm91bmQiOiAiIzBmMTcyYSIsCiAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICJzcGFjaW5nIjogNCwKICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiU0YiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgImNvbG9yIjogIiM5NGEzYjgiCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJpbWFnZSIsCiAgICAgICAgICAgICJzeXN0ZW1OYW1lIjogInN0YXIuZmlsbCIsCiAgICAgICAgICAgICJzaXplIjogMzIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmJiZjI0IgogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgInNwYWNpbmciOiA0LAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAiY29udGVudCI6ICJEYXRhIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiaW1hZ2UiLAogICAgICAgICAgICAiZGF0YSI6ICJpVkJPUncwS0dnb0FBQUFOU1VoRVVnQUFBQUVBQUFBQkNBWUFBQUFmRmNTSkFBQUFEVWxFUVZSNG5HTzRvNmIySHdBRk1nSW96Rlh4UVFBQUFBQkpSVTVFcmtKZ2dnPT0iLAogICAgICAgICAgICAic2l6ZSI6IDMyLAogICAgICAgICAgICAiY29udGVudE1vZGUiOiAiZmlsbCIsCiAgICAgICAgICAgICJjbGlwU2hhcGUiOiAiY2lyY2xlIgogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgInNwYWNpbmciOiA0LAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAiY29udGVudCI6ICJVUkwiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgImNvbG9yIjogIiM5NGEzYjgiCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJpbWFnZSIsCiAgICAgICAgICAgICJ1cmwiOiAiZGF0YTppbWFnZS9wbmc7YmFzZTY0LGlWQk9SdzBLR2dvQUFBQU5TVWhFVWdBQUFBRUFBQUFCQ0FZQUFBQWZGY1NKQUFBQURVbEVRVlI0bkdPdzJQdmpQd0FHQ3dMdGd4ejdPd0FBQUFCSlJVNUVya0pnZ2c9PSIsCiAgICAgICAgICAgICJzaXplIjogMzIsCiAgICAgICAgICAgICJjb250ZW50TW9kZSI6ICJmaXQiCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9CiAgICBdCiAgfQp9" />

## layout

Sizes: small

### small

<ShotGrid case="layout.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "spacing": 8,
    "padding": 12,
    "background": "#1a1a2e",
    "children": [
      {
        "type": "text",
        "content": "Hello",
        "fontSize": 18,
        "fontWeight": "bold",
        "color": "#fff",
        "alignment": "center"
      },
      {
        "type": "hstack",
        "spacing": 4,
        "children": [
          {
            "type": "shape",
            "shapeType": "capsule",
            "size": 8,
            "fill": "#4CAF50"
          },
          {
            "type": "divider"
          },
          {
            "type": "spacer"
          },
          {
            "type": "progress",
            "value": 0.5,
            "tint": "#4CAF50",
            "label": "OK"
          }
        ]
      },
      {
        "type": "button",
        "label": "Go",
        "action": "go",
        "backgroundColor": "#2196F3",
        "color": "#fff"
      }
    ]
  }
}
```

</details>

<Playground case="layout.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInNwYWNpbmciOiA4LAogICAgInBhZGRpbmciOiAxMiwKICAgICJiYWNrZ3JvdW5kIjogIiMxYTFhMmUiLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiSGVsbG8iLAogICAgICAgICJmb250U2l6ZSI6IDE4LAogICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgInNwYWNpbmciOiA0LAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAic2hhcGVUeXBlIjogImNhcHN1bGUiLAogICAgICAgICAgICAic2l6ZSI6IDgsCiAgICAgICAgICAgICJmaWxsIjogIiM0Q0FGNTAiCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJkaXZpZGVyIgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic3BhY2VyIgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICAgICAidmFsdWUiOiAwLjUsCiAgICAgICAgICAgICJ0aW50IjogIiM0Q0FGNTAiLAogICAgICAgICAgICAibGFiZWwiOiAiT0siCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAibGFiZWwiOiAiR28iLAogICAgICAgICJhY3Rpb24iOiAiZ28iLAogICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzIxOTZGMyIsCiAgICAgICAgImNvbG9yIjogIiNmZmYiCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## link-chip

Sizes: small

### small

<ShotGrid case="link-chip.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 14,
    "spacing": 8,
    "cornerRadius": 14,
    "background": "#0b1220",
    "alignment": "center",
    "children": [
      {
        "type": "link",
        "action": "open_settings",
        "padding": 8,
        "cornerRadius": 999,
        "background": "#1e293b",
        "children": [
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "image",
                "systemName": "gear",
                "size": 14,
                "color": "#38bdf8"
              },
              {
                "type": "text",
                "content": "Settings",
                "fontSize": 13,
                "fontWeight": "medium",
                "color": "#e2e8f0"
              }
            ]
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="link-chip.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxNCwKICAgICJzcGFjaW5nIjogOCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNCwKICAgICJiYWNrZ3JvdW5kIjogIiMwYjEyMjAiLAogICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAibGluayIsCiAgICAgICAgImFjdGlvbiI6ICJvcGVuX3NldHRpbmdzIiwKICAgICAgICAicGFkZGluZyI6IDgsCiAgICAgICAgImNvcm5lclJhZGl1cyI6IDk5OSwKICAgICAgICAiYmFja2dyb3VuZCI6ICIjMWUyOTNiIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgICAgICJzcGFjaW5nIjogNiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAiaW1hZ2UiLAogICAgICAgICAgICAgICAgInN5c3RlbU5hbWUiOiAiZ2VhciIsCiAgICAgICAgICAgICAgICAic2l6ZSI6IDE0LAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzOGJkZjgiCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIlNldHRpbmdzIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDEzLAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAibWVkaXVtIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjZTJlOGYwIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgXQogIH0KfQ==" />

## list-20-items

Sizes: large

### large

<ShotGrid case="list-20-items.large" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "large": {
    "type": "list",
    "spacing": 2,
    "fontSize": 12,
    "color": "#e2e8f0",
    "background": "#0f172a",
    "padding": 8,
    "items": [
      {
        "text": "Item 01",
        "checked": false
      },
      {
        "text": "Item 02",
        "checked": true
      },
      {
        "text": "Item 03"
      },
      {
        "text": "Item 04"
      },
      {
        "text": "Item 05"
      },
      {
        "text": "Item 06"
      },
      {
        "text": "Item 07"
      },
      {
        "text": "Item 08"
      },
      {
        "text": "Item 09"
      },
      {
        "text": "Item 10"
      },
      {
        "text": "Item 11"
      },
      {
        "text": "Item 12"
      },
      {
        "text": "Item 13"
      },
      {
        "text": "Item 14"
      },
      {
        "text": "Item 15"
      },
      {
        "text": "Item 16"
      },
      {
        "text": "Item 17"
      },
      {
        "text": "Item 18"
      },
      {
        "text": "Item 19"
      },
      {
        "text": "Item 20"
      }
    ]
  }
}
```

</details>

<Playground case="list-20-items.large" size="large" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibGFyZ2UiOiB7CiAgICAidHlwZSI6ICJsaXN0IiwKICAgICJzcGFjaW5nIjogMiwKICAgICJmb250U2l6ZSI6IDEyLAogICAgImNvbG9yIjogIiNlMmU4ZjAiLAogICAgImJhY2tncm91bmQiOiAiIzBmMTcyYSIsCiAgICAicGFkZGluZyI6IDgsCiAgICAiaXRlbXMiOiBbCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDAxIiwKICAgICAgICAiY2hlY2tlZCI6IGZhbHNlCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDAyIiwKICAgICAgICAiY2hlY2tlZCI6IHRydWUKICAgICAgfSwKICAgICAgewogICAgICAgICJ0ZXh0IjogIkl0ZW0gMDMiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDA0IgogICAgICB9LAogICAgICB7CiAgICAgICAgInRleHQiOiAiSXRlbSAwNSIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0ZXh0IjogIkl0ZW0gMDYiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDA3IgogICAgICB9LAogICAgICB7CiAgICAgICAgInRleHQiOiAiSXRlbSAwOCIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0ZXh0IjogIkl0ZW0gMDkiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDEwIgogICAgICB9LAogICAgICB7CiAgICAgICAgInRleHQiOiAiSXRlbSAxMSIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0ZXh0IjogIkl0ZW0gMTIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDEzIgogICAgICB9LAogICAgICB7CiAgICAgICAgInRleHQiOiAiSXRlbSAxNCIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0ZXh0IjogIkl0ZW0gMTUiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDE2IgogICAgICB9LAogICAgICB7CiAgICAgICAgInRleHQiOiAiSXRlbSAxNyIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0ZXh0IjogIkl0ZW0gMTgiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidGV4dCI6ICJJdGVtIDE5IgogICAgICB9LAogICAgICB7CiAgICAgICAgInRleHQiOiAiSXRlbSAyMCIKICAgICAgfQogICAgXQogIH0KfQ==" />

## music

Sizes: medium

### medium

<ShotGrid case="music.medium" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "medium": {
    "type": "hstack",
    "padding": 14,
    "spacing": 14,
    "cornerRadius": 16,
    "background": "#16213e",
    "children": [
      {
        "type": "shape",
        "shapeType": "circle",
        "fill": "#e94560",
        "size": 56
      },
      {
        "type": "vstack",
        "spacing": 6,
        "alignment": "leading",
        "children": [
          {
            "type": "text",
            "content": "Bohemian Rhapsody",
            "fontSize": 17,
            "fontWeight": "semibold",
            "color": "#fff",
            "lineLimit": 1
          },
          {
            "type": "text",
            "content": "Queen • A Night at the Opera",
            "fontSize": 12,
            "color": "#94a3b8"
          },
          {
            "type": "progress",
            "value": 0.625,
            "tint": "#e94560",
            "label": "3:42 / 5:55",
            "color": "#94a3b8"
          },
          {
            "type": "hstack",
            "spacing": 20,
            "children": [
              {
                "type": "button",
                "label": "⏮",
                "action": "music_prev",
                "backgroundColor": "#1a2744",
                "color": "#e2e8f0",
                "fontSize": 14,
                "textAlignment": "center"
              },
              {
                "type": "button",
                "label": "⏸",
                "action": "music_toggle",
                "backgroundColor": "#e94560",
                "color": "#ffffff",
                "fontSize": 14,
                "textAlignment": "center"
              },
              {
                "type": "button",
                "label": "⏭",
                "action": "music_next",
                "backgroundColor": "#1a2744",
                "color": "#e2e8f0",
                "fontSize": 14,
                "textAlignment": "center"
              }
            ]
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="music.medium" size="medium" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibWVkaXVtIjogewogICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICJwYWRkaW5nIjogMTQsCiAgICAic3BhY2luZyI6IDE0LAogICAgImNvcm5lclJhZGl1cyI6IDE2LAogICAgImJhY2tncm91bmQiOiAiIzE2MjEzZSIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgInNoYXBlVHlwZSI6ICJjaXJjbGUiLAogICAgICAgICJmaWxsIjogIiNlOTQ1NjAiLAogICAgICAgICJzaXplIjogNTYKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgInNwYWNpbmciOiA2LAogICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiQm9oZW1pYW4gUmhhcHNvZHkiLAogICAgICAgICAgICAiZm9udFNpemUiOiAxNywKICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAic2VtaWJvbGQiLAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgImNvbnRlbnQiOiAiUXVlZW4g4oCiIEEgTmlnaHQgYXQgdGhlIE9wZXJhIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTIsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICAgICAidmFsdWUiOiAwLjYyNSwKICAgICAgICAgICAgInRpbnQiOiAiI2U5NDU2MCIsCiAgICAgICAgICAgICJsYWJlbCI6ICIzOjQyIC8gNTo1NSIsCiAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiAyMCwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogImJ1dHRvbiIsCiAgICAgICAgICAgICAgICAibGFiZWwiOiAi4o+uIiwKICAgICAgICAgICAgICAgICJhY3Rpb24iOiAibXVzaWNfcHJldiIsCiAgICAgICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiMxYTI3NDQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiNlMmU4ZjAiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTQsCiAgICAgICAgICAgICAgICAidGV4dEFsaWdubWVudCI6ICJjZW50ZXIiCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAgICAgImxhYmVsIjogIuKPuCIsCiAgICAgICAgICAgICAgICAiYWN0aW9uIjogIm11c2ljX3RvZ2dsZSIsCiAgICAgICAgICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiNlOTQ1NjAiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiNmZmZmZmYiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTQsCiAgICAgICAgICAgICAgICAidGV4dEFsaWdubWVudCI6ICJjZW50ZXIiCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICAgICAgICAgImxhYmVsIjogIuKPrSIsCiAgICAgICAgICAgICAgICAiYWN0aW9uIjogIm11c2ljX25leHQiLAogICAgICAgICAgICAgICAgImJhY2tncm91bmRDb2xvciI6ICIjMWEyNzQ0IiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjZTJlOGYwIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDE0LAogICAgICAgICAgICAgICAgInRleHRBbGlnbm1lbnQiOiAiY2VudGVyIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgXQogIH0KfQ==" />

## nested-dashboard

Sizes: large

### large

<ShotGrid case="nested-dashboard.large" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "large": {
    "type": "vstack",
    "padding": 12,
    "spacing": 8,
    "cornerRadius": 16,
    "background": "#0f172a",
    "children": [
      {
        "type": "hstack",
        "spacing": 10,
        "alignment": "center",
        "children": [
          {
            "type": "zstack",
            "alignment": "center",
            "children": [
              {
                "type": "shape",
                "shapeType": "circle",
                "fill": "#334155",
                "size": 44
              },
              {
                "type": "text",
                "content": "42",
                "fontSize": 16,
                "fontWeight": "bold",
                "color": "#fff"
              }
            ]
          },
          {
            "type": "vstack",
            "spacing": 2,
            "alignment": "leading",
            "children": [
              {
                "type": "text",
                "content": "Nested Dashboard",
                "fontSize": 16,
                "fontWeight": "bold",
                "color": "#f8fafc",
                "lineLimit": 1
              },
              {
                "type": "text",
                "content": "hstack → zstack + grid + list",
                "fontSize": 11,
                "color": "#94a3b8",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "spacer"
          },
          {
            "type": "button",
            "label": "Sync",
            "action": "dash_sync",
            "backgroundColor": "#2563eb",
            "color": "#fff",
            "fontSize": 12,
            "cornerRadius": 8
          }
        ]
      },
      {
        "type": "divider",
        "color": "#334155"
      },
      {
        "type": "grid",
        "columns": 2,
        "spacing": 8,
        "rowSpacing": 8,
        "children": [
          {
            "type": "vstack",
            "padding": 8,
            "spacing": 4,
            "cornerRadius": 10,
            "background": "#1e293b",
            "children": [
              {
                "type": "text",
                "content": "CPU",
                "fontSize": 11,
                "color": "#94a3b8"
              },
              {
                "type": "text",
                "content": "38%",
                "fontSize": 22,
                "fontWeight": "bold",
                "color": "#22c55e"
              },
              {
                "type": "progress",
                "value": 0.38,
                "tint": "#22c55e"
              }
            ]
          },
          {
            "type": "vstack",
            "padding": 8,
            "spacing": 4,
            "cornerRadius": 10,
            "background": "#1e293b",
            "children": [
              {
                "type": "text",
                "content": "RAM",
                "fontSize": 11,
                "color": "#94a3b8"
              },
              {
                "type": "text",
                "content": "62%",
                "fontSize": 22,
                "fontWeight": "bold",
                "color": "#f59e0b"
              },
              {
                "type": "progress",
                "value": 0.62,
                "tint": "#f59e0b"
              }
            ]
          },
          {
            "type": "vstack",
            "padding": 8,
            "spacing": 4,
            "cornerRadius": 10,
            "background": "#1e293b",
            "children": [
              {
                "type": "text",
                "content": "Disk",
                "fontSize": 11,
                "color": "#94a3b8"
              },
              {
                "type": "text",
                "content": "81%",
                "fontSize": 22,
                "fontWeight": "bold",
                "color": "#ef4444"
              },
              {
                "type": "progress",
                "value": 0.81,
                "tint": "#ef4444"
              }
            ]
          },
          {
            "type": "vstack",
            "padding": 8,
            "spacing": 4,
            "cornerRadius": 10,
            "background": "#1e293b",
            "children": [
              {
                "type": "text",
                "content": "Net",
                "fontSize": 11,
                "color": "#94a3b8"
              },
              {
                "type": "text",
                "content": "12MB/s",
                "fontSize": 18,
                "fontWeight": "bold",
                "color": "#38bdf8"
              },
              {
                "type": "progress",
                "value": 0.24,
                "tint": "#38bdf8"
              }
            ]
          }
        ]
      },
      {
        "type": "divider",
        "color": "#334155"
      },
      {
        "type": "list",
        "spacing": 3,
        "fontSize": 12,
        "color": "#e2e8f0",
        "items": [
          {
            "text": "api-gateway",
            "checked": true
          },
          {
            "text": "worker-pool",
            "checked": true
          },
          {
            "text": "cache-redis",
            "checked": false
          },
          {
            "text": "queue-amqp"
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="nested-dashboard.large" size="large" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAibGFyZ2UiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJzcGFjaW5nIjogOCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNiwKICAgICJiYWNrZ3JvdW5kIjogIiMwZjE3MmEiLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDEwLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICJzaGFwZVR5cGUiOiAiY2lyY2xlIiwKICAgICAgICAgICAgICAgICJmaWxsIjogIiMzMzQxNTUiLAogICAgICAgICAgICAgICAgInNpemUiOiA0NAogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICI0MiIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxNiwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiNmZmYiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDIsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIk5lc3RlZCBEYXNoYm9hcmQiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTYsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjZjhmYWZjIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogImhzdGFjayDihpIgenN0YWNrICsgZ3JpZCArIGxpc3QiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzk0YTNiOCIsCiAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic3BhY2VyIgogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiYnV0dG9uIiwKICAgICAgICAgICAgImxhYmVsIjogIlN5bmMiLAogICAgICAgICAgICAiYWN0aW9uIjogImRhc2hfc3luYyIsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kQ29sb3IiOiAiIzI1NjNlYiIsCiAgICAgICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAgICAgImZvbnRTaXplIjogMTIsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA4CiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiZGl2aWRlciIsCiAgICAgICAgImNvbG9yIjogIiMzMzQxNTUiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJncmlkIiwKICAgICAgICAiY29sdW1ucyI6IDIsCiAgICAgICAgInNwYWNpbmciOiA4LAogICAgICAgICJyb3dTcGFjaW5nIjogOCwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgICAgICJwYWRkaW5nIjogOCwKICAgICAgICAgICAgInNwYWNpbmciOiA0LAogICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogMTAsCiAgICAgICAgICAgICJiYWNrZ3JvdW5kIjogIiMxZTI5M2IiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICJDUFUiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzk0YTNiOCIKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiMzglIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDIyLAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzIyYzU1ZSIKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInByb2dyZXNzIiwKICAgICAgICAgICAgICAgICJ2YWx1ZSI6IDAuMzgsCiAgICAgICAgICAgICAgICAidGludCI6ICIjMjJjNTVlIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgInBhZGRpbmciOiA4LAogICAgICAgICAgICAic3BhY2luZyI6IDQsCiAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiAxMCwKICAgICAgICAgICAgImJhY2tncm91bmQiOiAiIzFlMjkzYiIsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIlJBTSIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICI2MiUiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMjIsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjZjU5ZTBiIgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICAgICAgICAgInZhbHVlIjogMC42MiwKICAgICAgICAgICAgICAgICJ0aW50IjogIiNmNTllMGIiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAicGFkZGluZyI6IDgsCiAgICAgICAgICAgICJzcGFjaW5nIjogNCwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDEwLAogICAgICAgICAgICAiYmFja2dyb3VuZCI6ICIjMWUyOTNiIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiRGlzayIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjOTRhM2I4IgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICI4MSUiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMjIsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjZWY0NDQ0IgogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICAgICAgICAgInZhbHVlIjogMC44MSwKICAgICAgICAgICAgICAgICJ0aW50IjogIiNlZjQ0NDQiCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAicGFkZGluZyI6IDgsCiAgICAgICAgICAgICJzcGFjaW5nIjogNCwKICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDEwLAogICAgICAgICAgICAiYmFja2dyb3VuZCI6ICIjMWUyOTNiIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiTmV0IiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiM5NGEzYjgiCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIjEyTUIvcyIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxOCwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzOGJkZjgiCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJwcm9ncmVzcyIsCiAgICAgICAgICAgICAgICAidmFsdWUiOiAwLjI0LAogICAgICAgICAgICAgICAgInRpbnQiOiAiIzM4YmRmOCIKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJkaXZpZGVyIiwKICAgICAgICAiY29sb3IiOiAiIzMzNDE1NSIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogImxpc3QiLAogICAgICAgICJzcGFjaW5nIjogMywKICAgICAgICAiZm9udFNpemUiOiAxMiwKICAgICAgICAiY29sb3IiOiAiI2UyZThmMCIsCiAgICAgICAgIml0ZW1zIjogWwogICAgICAgICAgewogICAgICAgICAgICAidGV4dCI6ICJhcGktZ2F0ZXdheSIsCiAgICAgICAgICAgICJjaGVja2VkIjogdHJ1ZQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInRleHQiOiAid29ya2VyLXBvb2wiLAogICAgICAgICAgICAiY2hlY2tlZCI6IHRydWUKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0ZXh0IjogImNhY2hlLXJlZGlzIiwKICAgICAgICAgICAgImNoZWNrZWQiOiBmYWxzZQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInRleHQiOiAicXVldWUtYW1xcCIKICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## null-fields

Sizes: small

### small

<ShotGrid case="null-fields.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 8,
    "background": "#000",
    "children": [
      {
        "type": "button",
        "label": "Tap",
        "action": "go",
        "url": null,
        "color": "#fff",
        "backgroundColor": "#2563eb"
      },
      {
        "type": "text",
        "content": "ok",
        "fontSize": 12,
        "color": "#fff",
        "fontWeight": null
      },
      {
        "type": "progress",
        "value": 0.3,
        "label": null,
        "tint": "#22c55e"
      }
    ]
  }
}
```

</details>

<Playground case="null-fields.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiA4LAogICAgImJhY2tncm91bmQiOiAiIzAwMCIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJidXR0b24iLAogICAgICAgICJsYWJlbCI6ICJUYXAiLAogICAgICAgICJhY3Rpb24iOiAiZ28iLAogICAgICAgICJ1cmwiOiBudWxsLAogICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAiYmFja2dyb3VuZENvbG9yIjogIiMyNTYzZWIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICJvayIsCiAgICAgICAgImZvbnRTaXplIjogMTIsCiAgICAgICAgImNvbG9yIjogIiNmZmYiLAogICAgICAgICJmb250V2VpZ2h0IjogbnVsbAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICJ2YWx1ZSI6IDAuMywKICAgICAgICAibGFiZWwiOiBudWxsLAogICAgICAgICJ0aW50IjogIiMyMmM1NWUiCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## progress-gauge-variants

Sizes: small

### small

<ShotGrid case="progress-gauge-variants.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 10,
    "spacing": 8,
    "cornerRadius": 14,
    "background": "#111827",
    "children": [
      {
        "type": "hstack",
        "spacing": 12,
        "alignment": "center",
        "children": [
          {
            "type": "progress",
            "value": 0.65,
            "barStyle": "circular",
            "tint": "#22c55e"
          },
          {
            "type": "progress",
            "value": 0.4,
            "barStyle": "linear",
            "tint": "#38bdf8",
            "label": "Linear",
            "color": "#94a3b8",
            "flex": 1
          }
        ]
      },
      {
        "type": "hstack",
        "spacing": 16,
        "alignment": "center",
        "children": [
          {
            "type": "gauge",
            "value": 0.72,
            "min": 0,
            "max": 1,
            "currentValueLabel": "72%",
            "label": "Circ",
            "gaugeStyle": "circular",
            "tint": "#f59e0b",
            "color": "#e2e8f0"
          },
          {
            "type": "gauge",
            "value": 0.45,
            "min": 0,
            "max": 1,
            "currentValueLabel": "45%",
            "label": "Lin",
            "gaugeStyle": "linear",
            "tint": "#a855f7",
            "color": "#e2e8f0",
            "flex": 1
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="progress-gauge-variants.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMCwKICAgICJzcGFjaW5nIjogOCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNCwKICAgICJiYWNrZ3JvdW5kIjogIiMxMTE4MjciLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDEyLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInByb2dyZXNzIiwKICAgICAgICAgICAgInZhbHVlIjogMC42NSwKICAgICAgICAgICAgImJhclN0eWxlIjogImNpcmN1bGFyIiwKICAgICAgICAgICAgInRpbnQiOiAiIzIyYzU1ZSIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInByb2dyZXNzIiwKICAgICAgICAgICAgInZhbHVlIjogMC40LAogICAgICAgICAgICAiYmFyU3R5bGUiOiAibGluZWFyIiwKICAgICAgICAgICAgInRpbnQiOiAiIzM4YmRmOCIsCiAgICAgICAgICAgICJsYWJlbCI6ICJMaW5lYXIiLAogICAgICAgICAgICAiY29sb3IiOiAiIzk0YTNiOCIsCiAgICAgICAgICAgICJmbGV4IjogMQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgInNwYWNpbmciOiAxNiwKICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJnYXVnZSIsCiAgICAgICAgICAgICJ2YWx1ZSI6IDAuNzIsCiAgICAgICAgICAgICJtaW4iOiAwLAogICAgICAgICAgICAibWF4IjogMSwKICAgICAgICAgICAgImN1cnJlbnRWYWx1ZUxhYmVsIjogIjcyJSIsCiAgICAgICAgICAgICJsYWJlbCI6ICJDaXJjIiwKICAgICAgICAgICAgImdhdWdlU3R5bGUiOiAiY2lyY3VsYXIiLAogICAgICAgICAgICAidGludCI6ICIjZjU5ZTBiIiwKICAgICAgICAgICAgImNvbG9yIjogIiNlMmU4ZjAiCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJnYXVnZSIsCiAgICAgICAgICAgICJ2YWx1ZSI6IDAuNDUsCiAgICAgICAgICAgICJtaW4iOiAwLAogICAgICAgICAgICAibWF4IjogMSwKICAgICAgICAgICAgImN1cnJlbnRWYWx1ZUxhYmVsIjogIjQ1JSIsCiAgICAgICAgICAgICJsYWJlbCI6ICJMaW4iLAogICAgICAgICAgICAiZ2F1Z2VTdHlsZSI6ICJsaW5lYXIiLAogICAgICAgICAgICAidGludCI6ICIjYTg1NWY3IiwKICAgICAgICAgICAgImNvbG9yIjogIiNlMmU4ZjAiLAogICAgICAgICAgICAiZmxleCI6IDEKICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## shape-capsule

Sizes: small

### small

<ShotGrid case="shape-capsule.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 12,
    "background": "#0a0a0a",
    "alignment": "center",
    "children": [
      {
        "type": "shape",
        "shapeType": "capsule",
        "size": 12,
        "fill": "#22c55e"
      },
      {
        "type": "shape",
        "shapeType": "circle",
        "size": 24,
        "fill": "#3b82f6"
      }
    ]
  }
}
```

</details>

<Playground case="shape-capsule.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJiYWNrZ3JvdW5kIjogIiMwYTBhMGEiLAogICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICJzaGFwZVR5cGUiOiAiY2Fwc3VsZSIsCiAgICAgICAgInNpemUiOiAxMiwKICAgICAgICAiZmlsbCI6ICIjMjJjNTVlIgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICJzaGFwZVR5cGUiOiAiY2lyY2xlIiwKICAgICAgICAic2l6ZSI6IDI0LAogICAgICAgICJmaWxsIjogIiMzYjgyZjYiCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## style-chrome

Sizes: small

### small

<ShotGrid case="style-chrome.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 12,
    "spacing": 10,
    "cornerRadius": 16,
    "background": {
      "light": "#EEF2FF",
      "dark": "#0f172a"
    },
    "border": {
      "color": "#6366f1",
      "width": 2
    },
    "shadow": {
      "color": "rgba(99,102,241,0.45)",
      "radius": 10,
      "x": 0,
      "y": 4
    },
    "children": [
      {
        "type": "hstack",
        "spacing": 10,
        "alignment": "center",
        "children": [
          {
            "type": "image",
            "systemName": "person.fill",
            "size": 36,
            "color": "#fff",
            "clipShape": "circle",
            "background": "#6366f1",
            "frame": {
              "width": 36,
              "height": 36
            }
          },
          {
            "type": "text",
            "content": "Chrome",
            "fontSize": 15,
            "fontWeight": "semibold",
            "color": "label",
            "opacity": 0.85
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="style-chrome.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJzcGFjaW5nIjogMTAsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6IHsKICAgICAgImxpZ2h0IjogIiNFRUYyRkYiLAogICAgICAiZGFyayI6ICIjMGYxNzJhIgogICAgfSwKICAgICJib3JkZXIiOiB7CiAgICAgICJjb2xvciI6ICIjNjM2NmYxIiwKICAgICAgIndpZHRoIjogMgogICAgfSwKICAgICJzaGFkb3ciOiB7CiAgICAgICJjb2xvciI6ICJyZ2JhKDk5LDEwMiwyNDEsMC40NSkiLAogICAgICAicmFkaXVzIjogMTAsCiAgICAgICJ4IjogMCwKICAgICAgInkiOiA0CiAgICB9LAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDEwLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImltYWdlIiwKICAgICAgICAgICAgInN5c3RlbU5hbWUiOiAicGVyc29uLmZpbGwiLAogICAgICAgICAgICAic2l6ZSI6IDM2LAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJjbGlwU2hhcGUiOiAiY2lyY2xlIiwKICAgICAgICAgICAgImJhY2tncm91bmQiOiAiIzYzNjZmMSIsCiAgICAgICAgICAgICJmcmFtZSI6IHsKICAgICAgICAgICAgICAid2lkdGgiOiAzNiwKICAgICAgICAgICAgICAiaGVpZ2h0IjogMzYKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICJjb250ZW50IjogIkNocm9tZSIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDE1LAogICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgICAgICJvcGFjaXR5IjogMC44NQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgXQogIH0KfQ==" />

## style-chrome.light

Sizes: small

### small

<ShotGrid case="style-chrome.light.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 12,
    "spacing": 10,
    "cornerRadius": 16,
    "background": {
      "light": "#EEF2FF",
      "dark": "#0f172a"
    },
    "border": {
      "color": "#6366f1",
      "width": 2
    },
    "shadow": {
      "color": "rgba(99,102,241,0.45)",
      "radius": 10,
      "x": 0,
      "y": 4
    },
    "children": [
      {
        "type": "hstack",
        "spacing": 10,
        "alignment": "center",
        "children": [
          {
            "type": "image",
            "systemName": "person.fill",
            "size": 36,
            "color": "#fff",
            "clipShape": "circle",
            "background": "#6366f1",
            "frame": {
              "width": 36,
              "height": 36
            }
          },
          {
            "type": "text",
            "content": "Chrome",
            "fontSize": 15,
            "fontWeight": "semibold",
            "color": "label",
            "opacity": 0.85
          }
        ]
      }
    ]
  }
}
```

</details>

<Playground case="style-chrome.light.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJzcGFjaW5nIjogMTAsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6IHsKICAgICAgImxpZ2h0IjogIiNFRUYyRkYiLAogICAgICAiZGFyayI6ICIjMGYxNzJhIgogICAgfSwKICAgICJib3JkZXIiOiB7CiAgICAgICJjb2xvciI6ICIjNjM2NmYxIiwKICAgICAgIndpZHRoIjogMgogICAgfSwKICAgICJzaGFkb3ciOiB7CiAgICAgICJjb2xvciI6ICJyZ2JhKDk5LDEwMiwyNDEsMC40NSkiLAogICAgICAicmFkaXVzIjogMTAsCiAgICAgICJ4IjogMCwKICAgICAgInkiOiA0CiAgICB9LAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDEwLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImltYWdlIiwKICAgICAgICAgICAgInN5c3RlbU5hbWUiOiAicGVyc29uLmZpbGwiLAogICAgICAgICAgICAic2l6ZSI6IDM2LAogICAgICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgICAgICJjbGlwU2hhcGUiOiAiY2lyY2xlIiwKICAgICAgICAgICAgImJhY2tncm91bmQiOiAiIzYzNjZmMSIsCiAgICAgICAgICAgICJmcmFtZSI6IHsKICAgICAgICAgICAgICAid2lkdGgiOiAzNiwKICAgICAgICAgICAgICAiaGVpZ2h0IjogMzYKICAgICAgICAgICAgfQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICJjb250ZW50IjogIkNocm9tZSIsCiAgICAgICAgICAgICJmb250U2l6ZSI6IDE1LAogICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgICAgICJvcGFjaXR5IjogMC44NQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfQogICAgXQogIH0KfQ==" />

## tasks

Sizes: small

### small

<ShotGrid case="tasks.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "children": [
      {
        "type": "text",
        "content": "Tasks",
        "fontSize": 18,
        "fontWeight": "bold",
        "fontDesign": null,
        "textStyle": null,
        "color": null,
        "alignment": null,
        "lineLimit": null,
        "padding": null,
        "background": null,
        "cornerRadius": null,
        "opacity": null,
        "frame": null,
        "border": null,
        "shadow": null,
        "clipShape": null,
        "flex": null
      },
      {
        "type": "list",
        "items": [
          {
            "text": "Buy milk",
            "checked": false,
            "action": "toggle",
            "payload": null
          },
          {
            "text": "Ship 0.4",
            "checked": true,
            "action": null,
            "payload": null
          }
        ],
        "spacing": 4,
        "fontSize": null,
        "color": null,
        "padding": null,
        "background": null,
        "cornerRadius": null,
        "opacity": null,
        "frame": null,
        "border": null,
        "shadow": null,
        "clipShape": null,
        "flex": null
      }
    ],
    "spacing": 6,
    "alignment": null,
    "padding": 10,
    "background": null,
    "cornerRadius": null,
    "opacity": null,
    "frame": null,
    "border": null,
    "shadow": null,
    "clipShape": null,
    "flex": null
  },
  "medium": null,
  "large": null
}
```

</details>

<Playground case="tasks.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiVGFza3MiLAogICAgICAgICJmb250U2l6ZSI6IDE4LAogICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICJmb250RGVzaWduIjogbnVsbCwKICAgICAgICAidGV4dFN0eWxlIjogbnVsbCwKICAgICAgICAiY29sb3IiOiBudWxsLAogICAgICAgICJhbGlnbm1lbnQiOiBudWxsLAogICAgICAgICJsaW5lTGltaXQiOiBudWxsLAogICAgICAgICJwYWRkaW5nIjogbnVsbCwKICAgICAgICAiYmFja2dyb3VuZCI6IG51bGwsCiAgICAgICAgImNvcm5lclJhZGl1cyI6IG51bGwsCiAgICAgICAgIm9wYWNpdHkiOiBudWxsLAogICAgICAgICJmcmFtZSI6IG51bGwsCiAgICAgICAgImJvcmRlciI6IG51bGwsCiAgICAgICAgInNoYWRvdyI6IG51bGwsCiAgICAgICAgImNsaXBTaGFwZSI6IG51bGwsCiAgICAgICAgImZsZXgiOiBudWxsCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJsaXN0IiwKICAgICAgICAiaXRlbXMiOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0ZXh0IjogIkJ1eSBtaWxrIiwKICAgICAgICAgICAgImNoZWNrZWQiOiBmYWxzZSwKICAgICAgICAgICAgImFjdGlvbiI6ICJ0b2dnbGUiLAogICAgICAgICAgICAicGF5bG9hZCI6IG51bGwKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0ZXh0IjogIlNoaXAgMC40IiwKICAgICAgICAgICAgImNoZWNrZWQiOiB0cnVlLAogICAgICAgICAgICAiYWN0aW9uIjogbnVsbCwKICAgICAgICAgICAgInBheWxvYWQiOiBudWxsCiAgICAgICAgICB9CiAgICAgICAgXSwKICAgICAgICAic3BhY2luZyI6IDQsCiAgICAgICAgImZvbnRTaXplIjogbnVsbCwKICAgICAgICAiY29sb3IiOiBudWxsLAogICAgICAgICJwYWRkaW5nIjogbnVsbCwKICAgICAgICAiYmFja2dyb3VuZCI6IG51bGwsCiAgICAgICAgImNvcm5lclJhZGl1cyI6IG51bGwsCiAgICAgICAgIm9wYWNpdHkiOiBudWxsLAogICAgICAgICJmcmFtZSI6IG51bGwsCiAgICAgICAgImJvcmRlciI6IG51bGwsCiAgICAgICAgInNoYWRvdyI6IG51bGwsCiAgICAgICAgImNsaXBTaGFwZSI6IG51bGwsCiAgICAgICAgImZsZXgiOiBudWxsCiAgICAgIH0KICAgIF0sCiAgICAic3BhY2luZyI6IDYsCiAgICAiYWxpZ25tZW50IjogbnVsbCwKICAgICJwYWRkaW5nIjogMTAsCiAgICAiYmFja2dyb3VuZCI6IG51bGwsCiAgICAiY29ybmVyUmFkaXVzIjogbnVsbCwKICAgICJvcGFjaXR5IjogbnVsbCwKICAgICJmcmFtZSI6IG51bGwsCiAgICAiYm9yZGVyIjogbnVsbCwKICAgICJzaGFkb3ciOiBudWxsLAogICAgImNsaXBTaGFwZSI6IG51bGwsCiAgICAiZmxleCI6IG51bGwKICB9LAogICJtZWRpdW0iOiBudWxsLAogICJsYXJnZSI6IG51bGwKfQ==" />

## text-align

Sizes: small

### small

<ShotGrid case="text-align.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 8,
    "background": "#111",
    "children": [
      {
        "type": "text",
        "content": "Leading",
        "fontSize": 14,
        "color": "#fff",
        "alignment": "leading"
      },
      {
        "type": "text",
        "content": "Center",
        "fontSize": 14,
        "color": "#fff",
        "alignment": "center"
      },
      {
        "type": "text",
        "content": "Trailing",
        "fontSize": 14,
        "color": "#fff",
        "alignment": "trailing"
      }
    ]
  }
}
```

</details>

<Playground case="text-align.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiA4LAogICAgImJhY2tncm91bmQiOiAiIzExMSIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICJMZWFkaW5nIiwKICAgICAgICAiZm9udFNpemUiOiAxNCwKICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgImFsaWdubWVudCI6ICJsZWFkaW5nIgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiQ2VudGVyIiwKICAgICAgICAiZm9udFNpemUiOiAxNCwKICAgICAgICAiY29sb3IiOiAiI2ZmZiIsCiAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICJUcmFpbGluZyIsCiAgICAgICAgImZvbnRTaXplIjogMTQsCiAgICAgICAgImNvbG9yIjogIiNmZmYiLAogICAgICAgICJhbGlnbm1lbnQiOiAidHJhaWxpbmciCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## toggle-row

Sizes: small

### small

<ShotGrid case="toggle-row.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 12,
    "spacing": 10,
    "cornerRadius": 14,
    "background": "#1C1C1E",
    "children": [
      {
        "type": "toggle",
        "isOn": true,
        "label": "Wi-Fi",
        "tint": "#22c55e",
        "color": "#e2e8f0",
        "action": "toggle_wifi"
      },
      {
        "type": "toggle",
        "isOn": false,
        "label": "Bluetooth",
        "tint": "#3b82f6",
        "color": "#e2e8f0",
        "action": "toggle_bt"
      }
    ]
  }
}
```

</details>

<Playground case="toggle-row.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJzcGFjaW5nIjogMTAsCiAgICAiY29ybmVyUmFkaXVzIjogMTQsCiAgICAiYmFja2dyb3VuZCI6ICIjMUMxQzFFIiwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogInRvZ2dsZSIsCiAgICAgICAgImlzT24iOiB0cnVlLAogICAgICAgICJsYWJlbCI6ICJXaS1GaSIsCiAgICAgICAgInRpbnQiOiAiIzIyYzU1ZSIsCiAgICAgICAgImNvbG9yIjogIiNlMmU4ZjAiLAogICAgICAgICJhY3Rpb24iOiAidG9nZ2xlX3dpZmkiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0b2dnbGUiLAogICAgICAgICJpc09uIjogZmFsc2UsCiAgICAgICAgImxhYmVsIjogIkJsdWV0b290aCIsCiAgICAgICAgInRpbnQiOiAiIzNiODJmNiIsCiAgICAgICAgImNvbG9yIjogIiNlMmU4ZjAiLAogICAgICAgICJhY3Rpb24iOiAidG9nZ2xlX2J0IgogICAgICB9CiAgICBdCiAgfQp9" />

## type-styles

Sizes: small

### small

<ShotGrid case="type-styles.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 10,
    "spacing": 6,
    "cornerRadius": 14,
    "background": "#111827",
    "children": [
      {
        "type": "text",
        "content": "Headline",
        "textStyle": "headline",
        "fontWeight": "semibold",
        "color": {
          "light": "#0f172a",
          "dark": "#f8fafc"
        }
      },
      {
        "type": "text",
        "content": "mono 0123",
        "fontDesign": "monospaced",
        "fontSize": 13,
        "color": "#38bdf8"
      },
      {
        "type": "text",
        "content": "Serif body",
        "fontDesign": "serif",
        "fontSize": 14,
        "color": "#e2e8f0"
      },
      {
        "type": "text",
        "content": "Caption line",
        "textStyle": "caption",
        "color": "#94a3b8"
      }
    ]
  }
}
```

</details>

<Playground case="type-styles.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMCwKICAgICJzcGFjaW5nIjogNiwKICAgICJjb3JuZXJSYWRpdXMiOiAxNCwKICAgICJiYWNrZ3JvdW5kIjogIiMxMTE4MjciLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiSGVhZGxpbmUiLAogICAgICAgICJ0ZXh0U3R5bGUiOiAiaGVhZGxpbmUiLAogICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAiY29sb3IiOiB7CiAgICAgICAgICAibGlnaHQiOiAiIzBmMTcyYSIsCiAgICAgICAgICAiZGFyayI6ICIjZjhmYWZjIgogICAgICAgIH0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICJjb250ZW50IjogIm1vbm8gMDEyMyIsCiAgICAgICAgImZvbnREZXNpZ24iOiAibW9ub3NwYWNlZCIsCiAgICAgICAgImZvbnRTaXplIjogMTMsCiAgICAgICAgImNvbG9yIjogIiMzOGJkZjgiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICJTZXJpZiBib2R5IiwKICAgICAgICAiZm9udERlc2lnbiI6ICJzZXJpZiIsCiAgICAgICAgImZvbnRTaXplIjogMTQsCiAgICAgICAgImNvbG9yIjogIiNlMmU4ZjAiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICJDYXB0aW9uIGxpbmUiLAogICAgICAgICJ0ZXh0U3R5bGUiOiAiY2FwdGlvbiIsCiAgICAgICAgImNvbG9yIjogIiM5NGEzYjgiCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## upcoming-payments

Sizes: small, medium, large

### small

<ShotGrid case="upcoming-payments.small" />

### medium

<ShotGrid case="upcoming-payments.medium" />

### large

<ShotGrid case="upcoming-payments.large" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 14,
    "spacing": 10,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "divider",
        "color": "#3878FA",
        "thickness": 4
      },
      {
        "type": "hstack",
        "spacing": 10,
        "alignment": "center",
        "children": [
          {
            "type": "zstack",
            "alignment": "center",
            "children": [
              {
                "type": "shape",
                "shapeType": "rectangle",
                "fill": "#1F59E0",
                "size": 44,
                "cornerRadius": 10
              },
              {
                "type": "text",
                "content": "S",
                "fontSize": 18,
                "fontWeight": "bold",
                "fontDesign": "rounded",
                "color": "#FFFFFF"
              }
            ]
          },
          {
            "type": "vstack",
            "spacing": 2,
            "alignment": "leading",
            "flex": 1,
            "children": [
              {
                "type": "text",
                "content": "Streaming",
                "fontSize": 15,
                "fontWeight": "bold",
                "fontDesign": "rounded",
                "color": "label",
                "lineLimit": 1
              },
              {
                "type": "text",
                "content": "Jun 1",
                "fontSize": 11,
                "fontWeight": "semibold",
                "color": "secondaryLabel",
                "lineLimit": 1
              }
            ]
          }
        ]
      },
      {
        "type": "spacer"
      },
      {
        "type": "text",
        "content": "$12.50",
        "fontSize": 22,
        "fontWeight": "bold",
        "fontDesign": "rounded",
        "color": "#3878FA",
        "alignment": "leading",
        "lineLimit": 1
      }
    ]
  },
  "medium": {
    "type": "vstack",
    "padding": 10,
    "spacing": 4,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "label",
        "text": "Upcoming",
        "systemName": "calendar.badge.clock",
        "fontSize": 11,
        "fontWeight": "bold",
        "color": "label",
        "iconColor": "#3878FA",
        "spacing": 4
      },
      {
        "type": "grid",
        "columns": 2,
        "spacing": 8,
        "rowSpacing": 6,
        "children": [
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "S",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Streaming",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 1",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$12.50",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "C",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Cloud Pro",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Today",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$9.99",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "M",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Music Plus",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Tomorrow",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$4.99",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "N",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "News Daily",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 5",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$2.99",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          }
        ]
      },
      {
        "type": "spacer",
        "minLength": 2
      },
      {
        "type": "text",
        "content": "+2 more",
        "fontSize": 9,
        "fontWeight": "bold",
        "color": "secondaryLabel",
        "lineLimit": 1
      }
    ]
  },
  "large": {
    "type": "vstack",
    "padding": 12,
    "spacing": 4,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "label",
        "text": "Upcoming",
        "systemName": "calendar.badge.clock",
        "fontSize": 11,
        "fontWeight": "bold",
        "color": "label",
        "iconColor": "#3878FA",
        "spacing": 4
      },
      {
        "type": "grid",
        "columns": 2,
        "spacing": 10,
        "rowSpacing": 8,
        "children": [
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "S",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Streaming",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 1",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$12.50",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "C",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Cloud Pro",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Today",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$9.99",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "M",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Music Plus",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Tomorrow",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$4.99",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "N",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "News Daily",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 5",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$2.99",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "G",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Gym Club",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 8",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$29.00",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "V",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "VPN Secure",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 12",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$6.49",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          }
        ]
      },
      {
        "type": "spacer",
        "minLength": 4
      },
      {
        "type": "text",
        "content": "+4 more",
        "fontSize": 9,
        "fontWeight": "bold",
        "color": "secondaryLabel",
        "lineLimit": 1
      }
    ]
  }
}
```

</details>

<Playground case="upcoming-payments.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxNCwKICAgICJzcGFjaW5nIjogMTAsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6IHsKICAgICAgImxpZ2h0IjogIiNGMkYyRjciLAogICAgICAiZGFyayI6ICIjMUMxQzFFIgogICAgfSwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogImRpdmlkZXIiLAogICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAidGhpY2tuZXNzIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDEwLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICJzaGFwZVR5cGUiOiAicmVjdGFuZ2xlIiwKICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgInNpemUiOiA0NCwKICAgICAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiAxMAogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICJTIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDE4LAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAiZm9udERlc2lnbiI6ICJyb3VuZGVkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiAyLAogICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIlN0cmVhbWluZyIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxNSwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImZvbnREZXNpZ24iOiAicm91bmRlZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDEiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJzcGFjZXIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICIkMTIuNTAiLAogICAgICAgICJmb250U2l6ZSI6IDIyLAogICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICJmb250RGVzaWduIjogInJvdW5kZWQiLAogICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgIH0KICAgIF0KICB9LAogICJtZWRpdW0iOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMCwKICAgICJzcGFjaW5nIjogNCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNiwKICAgICJiYWNrZ3JvdW5kIjogewogICAgICAibGlnaHQiOiAiI0YyRjJGNyIsCiAgICAgICJkYXJrIjogIiMxQzFDMUUiCiAgICB9LAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAibGFiZWwiLAogICAgICAgICJ0ZXh0IjogIlVwY29taW5nIiwKICAgICAgICAic3lzdGVtTmFtZSI6ICJjYWxlbmRhci5iYWRnZS5jbG9jayIsCiAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAiaWNvbkNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICJzcGFjaW5nIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiZ3JpZCIsCiAgICAgICAgImNvbHVtbnMiOiAyLAogICAgICAgICJzcGFjaW5nIjogOCwKICAgICAgICAicm93U3BhY2luZyI6IDYsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjAsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDUKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIlMiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogIiNGRkZGRkYiCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgICAgICAgICAic3BhY2luZyI6IDAsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAgICAgImZsZXgiOiAxLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiU3RyZWFtaW5nIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDEiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDgsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIiQxMi41MCIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgICAgICJzcGFjaW5nIjogNiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAienN0YWNrIiwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICAgICAic2hhcGVUeXBlIjogInJlY3RhbmdsZSIsCiAgICAgICAgICAgICAgICAgICAgImZpbGwiOiAiIzFGNTlFMCIsCiAgICAgICAgICAgICAgICAgICAgInNpemUiOiAyMCwKICAgICAgICAgICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiQyIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJDbG91ZCBQcm8iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDEwLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJUb2RheSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOCwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDkuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjAsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDUKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk0iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogIiNGRkZGRkYiCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgICAgICAgICAic3BhY2luZyI6IDAsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAgICAgImZsZXgiOiAxLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiTXVzaWMgUGx1cyIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAic2VtaWJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIlRvbW9ycm93IiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiA4LAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICJzZWNvbmRhcnlMYWJlbCIsCiAgICAgICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICIkNC45OSIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgICAgICJzcGFjaW5nIjogNiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAienN0YWNrIiwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICAgICAic2hhcGVUeXBlIjogInJlY3RhbmdsZSIsCiAgICAgICAgICAgICAgICAgICAgImZpbGwiOiAiIzFGNTlFMCIsCiAgICAgICAgICAgICAgICAgICAgInNpemUiOiAyMCwKICAgICAgICAgICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiTiIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJOZXdzIERhaWx5IiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDUiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDgsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIiQyLjk5IiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDEwLAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzM4NzhGQSIsCiAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInNwYWNlciIsCiAgICAgICAgIm1pbkxlbmd0aCI6IDIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICJjb250ZW50IjogIisyIG1vcmUiLAogICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAibGluZUxpbWl0IjogMQogICAgICB9CiAgICBdCiAgfSwKICAibGFyZ2UiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJzcGFjaW5nIjogNCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNiwKICAgICJiYWNrZ3JvdW5kIjogewogICAgICAibGlnaHQiOiAiI0YyRjJGNyIsCiAgICAgICJkYXJrIjogIiMxQzFDMUUiCiAgICB9LAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAibGFiZWwiLAogICAgICAgICJ0ZXh0IjogIlVwY29taW5nIiwKICAgICAgICAic3lzdGVtTmFtZSI6ICJjYWxlbmRhci5iYWRnZS5jbG9jayIsCiAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAiaWNvbkNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICJzcGFjaW5nIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiZ3JpZCIsCiAgICAgICAgImNvbHVtbnMiOiAyLAogICAgICAgICJzcGFjaW5nIjogMTAsCiAgICAgICAgInJvd1NwYWNpbmciOiA4LAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiA2LAogICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ6c3RhY2siLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAgICAgICAgICJzaGFwZVR5cGUiOiAicmVjdGFuZ2xlIiwKICAgICAgICAgICAgICAgICAgICAiZmlsbCI6ICIjMUY1OUUwIiwKICAgICAgICAgICAgICAgICAgICAic2l6ZSI6IDI0LAogICAgICAgICAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2CiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJTIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJTdHJlYW1pbmciLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJKdW4gMSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDEyLjUwIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzM4NzhGQSIsCiAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiA2LAogICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ6c3RhY2siLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAgICAgICAgICJzaGFwZVR5cGUiOiAicmVjdGFuZ2xlIiwKICAgICAgICAgICAgICAgICAgICAiZmlsbCI6ICIjMUY1OUUwIiwKICAgICAgICAgICAgICAgICAgICAic2l6ZSI6IDI0LAogICAgICAgICAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2CiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJDIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJDbG91ZCBQcm8iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJUb2RheSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDkuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjQsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk0iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAgICAgInNwYWNpbmciOiAwLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJsZWFkaW5nIiwKICAgICAgICAgICAgICAgICJmbGV4IjogMSwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk11c2ljIFBsdXMiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJUb21vcnJvdyIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDQuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjQsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk4iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAgICAgInNwYWNpbmciOiAwLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJsZWFkaW5nIiwKICAgICAgICAgICAgICAgICJmbGV4IjogMSwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk5ld3MgRGFpbHkiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJKdW4gNSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDIuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjQsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIkciLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAgICAgInNwYWNpbmciOiAwLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJsZWFkaW5nIiwKICAgICAgICAgICAgICAgICJmbGV4IjogMSwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIkd5bSBDbHViIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDgiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIiQyOS4wMCIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgICAgICJzcGFjaW5nIjogNiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAienN0YWNrIiwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICAgICAic2hhcGVUeXBlIjogInJlY3RhbmdsZSIsCiAgICAgICAgICAgICAgICAgICAgImZpbGwiOiAiIzFGNTlFMCIsCiAgICAgICAgICAgICAgICAgICAgInNpemUiOiAyNCwKICAgICAgICAgICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNgogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiViIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogIiNGRkZGRkYiCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgICAgICAgICAic3BhY2luZyI6IDAsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAgICAgImZsZXgiOiAxLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiVlBOIFNlY3VyZSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAic2VtaWJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIkp1biAxMiIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDYuNDkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAic3BhY2VyIiwKICAgICAgICAibWluTGVuZ3RoIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiKzQgbW9yZSIsCiAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## upcoming-payments-empty

Sizes: small, medium, large

### small

<ShotGrid case="upcoming-payments-empty.small" />

### medium

<ShotGrid case="upcoming-payments-empty.medium" />

### large

<ShotGrid case="upcoming-payments-empty.large" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 14,
    "spacing": 10,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "spacer"
      },
      {
        "type": "zstack",
        "alignment": "center",
        "children": [
          {
            "type": "shape",
            "shapeType": "circle",
            "fill": "#263878FA",
            "size": 48
          },
          {
            "type": "image",
            "systemName": "checkmark.circle.fill",
            "size": 28,
            "color": "#3878FA"
          }
        ]
      },
      {
        "type": "text",
        "content": "No upcoming payments",
        "fontSize": 13,
        "fontWeight": "semibold",
        "fontDesign": "rounded",
        "color": "secondaryLabel",
        "alignment": "center",
        "lineLimit": 2
      },
      {
        "type": "spacer"
      }
    ]
  },
  "medium": {
    "type": "vstack",
    "padding": 12,
    "spacing": 8,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "label",
        "text": "Upcoming",
        "systemName": "calendar.badge.clock",
        "fontSize": 11,
        "fontWeight": "bold",
        "color": "label",
        "iconColor": "#3878FA",
        "spacing": 4
      },
      {
        "type": "spacer"
      },
      {
        "type": "zstack",
        "alignment": "center",
        "children": [
          {
            "type": "shape",
            "shapeType": "circle",
            "fill": "#263878FA",
            "size": 40
          },
          {
            "type": "image",
            "systemName": "checkmark.circle.fill",
            "size": 22,
            "color": "#3878FA"
          }
        ]
      },
      {
        "type": "text",
        "content": "No upcoming payments",
        "fontSize": 12,
        "fontWeight": "semibold",
        "color": "secondaryLabel",
        "alignment": "center",
        "lineLimit": 2
      },
      {
        "type": "spacer"
      }
    ]
  },
  "large": {
    "type": "vstack",
    "padding": 14,
    "spacing": 10,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "label",
        "text": "Upcoming",
        "systemName": "calendar.badge.clock",
        "fontSize": 11,
        "fontWeight": "bold",
        "color": "label",
        "iconColor": "#3878FA",
        "spacing": 4
      },
      {
        "type": "spacer"
      },
      {
        "type": "zstack",
        "alignment": "center",
        "children": [
          {
            "type": "shape",
            "shapeType": "circle",
            "fill": "#263878FA",
            "size": 52
          },
          {
            "type": "image",
            "systemName": "checkmark.circle.fill",
            "size": 30,
            "color": "#3878FA"
          }
        ]
      },
      {
        "type": "text",
        "content": "No upcoming payments",
        "fontSize": 14,
        "fontWeight": "semibold",
        "fontDesign": "rounded",
        "color": "secondaryLabel",
        "alignment": "center",
        "lineLimit": 2
      },
      {
        "type": "spacer"
      }
    ]
  }
}
```

</details>

<Playground case="upcoming-payments-empty.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxNCwKICAgICJzcGFjaW5nIjogMTAsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6IHsKICAgICAgImxpZ2h0IjogIiNGMkYyRjciLAogICAgICAiZGFyayI6ICIjMUMxQzFFIgogICAgfSwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogInNwYWNlciIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAic2hhcGVUeXBlIjogImNpcmNsZSIsCiAgICAgICAgICAgICJmaWxsIjogIiMyNjM4NzhGQSIsCiAgICAgICAgICAgICJzaXplIjogNDgKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImltYWdlIiwKICAgICAgICAgICAgInN5c3RlbU5hbWUiOiAiY2hlY2ttYXJrLmNpcmNsZS5maWxsIiwKICAgICAgICAgICAgInNpemUiOiAyOCwKICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiTm8gdXBjb21pbmcgcGF5bWVudHMiLAogICAgICAgICJmb250U2l6ZSI6IDEzLAogICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAiZm9udERlc2lnbiI6ICJyb3VuZGVkIiwKICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAibGluZUxpbWl0IjogMgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAic3BhY2VyIgogICAgICB9CiAgICBdCiAgfSwKICAibWVkaXVtIjogewogICAgInR5cGUiOiAidnN0YWNrIiwKICAgICJwYWRkaW5nIjogMTIsCiAgICAic3BhY2luZyI6IDgsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6IHsKICAgICAgImxpZ2h0IjogIiNGMkYyRjciLAogICAgICAiZGFyayI6ICIjMUMxQzFFIgogICAgfSwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogImxhYmVsIiwKICAgICAgICAidGV4dCI6ICJVcGNvbWluZyIsCiAgICAgICAgInN5c3RlbU5hbWUiOiAiY2FsZW5kYXIuYmFkZ2UuY2xvY2siLAogICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgImljb25Db2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAic3BhY2luZyI6IDQKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInNwYWNlciIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAic2hhcGVUeXBlIjogImNpcmNsZSIsCiAgICAgICAgICAgICJmaWxsIjogIiMyNjM4NzhGQSIsCiAgICAgICAgICAgICJzaXplIjogNDAKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImltYWdlIiwKICAgICAgICAgICAgInN5c3RlbU5hbWUiOiAiY2hlY2ttYXJrLmNpcmNsZS5maWxsIiwKICAgICAgICAgICAgInNpemUiOiAyMiwKICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiTm8gdXBjb21pbmcgcGF5bWVudHMiLAogICAgICAgICJmb250U2l6ZSI6IDEyLAogICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAibGluZUxpbWl0IjogMgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAic3BhY2VyIgogICAgICB9CiAgICBdCiAgfSwKICAibGFyZ2UiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxNCwKICAgICJzcGFjaW5nIjogMTAsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6IHsKICAgICAgImxpZ2h0IjogIiNGMkYyRjciLAogICAgICAiZGFyayI6ICIjMUMxQzFFIgogICAgfSwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogImxhYmVsIiwKICAgICAgICAidGV4dCI6ICJVcGNvbWluZyIsCiAgICAgICAgInN5c3RlbU5hbWUiOiAiY2FsZW5kYXIuYmFkZ2UuY2xvY2siLAogICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgImljb25Db2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAic3BhY2luZyI6IDQKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInNwYWNlciIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAic2hhcGVUeXBlIjogImNpcmNsZSIsCiAgICAgICAgICAgICJmaWxsIjogIiMyNjM4NzhGQSIsCiAgICAgICAgICAgICJzaXplIjogNTIKICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImltYWdlIiwKICAgICAgICAgICAgInN5c3RlbU5hbWUiOiAiY2hlY2ttYXJrLmNpcmNsZS5maWxsIiwKICAgICAgICAgICAgInNpemUiOiAzMCwKICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiTm8gdXBjb21pbmcgcGF5bWVudHMiLAogICAgICAgICJmb250U2l6ZSI6IDE0LAogICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAiZm9udERlc2lnbiI6ICJyb3VuZGVkIiwKICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAibGluZUxpbWl0IjogMgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAic3BhY2VyIgogICAgICB9CiAgICBdCiAgfQp9" />

## upcoming-payments.light

Sizes: small

### small

<ShotGrid case="upcoming-payments.light.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "padding": 14,
    "spacing": 10,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "divider",
        "color": "#3878FA",
        "thickness": 4
      },
      {
        "type": "hstack",
        "spacing": 10,
        "alignment": "center",
        "children": [
          {
            "type": "zstack",
            "alignment": "center",
            "children": [
              {
                "type": "shape",
                "shapeType": "rectangle",
                "fill": "#1F59E0",
                "size": 44,
                "cornerRadius": 10
              },
              {
                "type": "text",
                "content": "S",
                "fontSize": 18,
                "fontWeight": "bold",
                "fontDesign": "rounded",
                "color": "#FFFFFF"
              }
            ]
          },
          {
            "type": "vstack",
            "spacing": 2,
            "alignment": "leading",
            "flex": 1,
            "children": [
              {
                "type": "text",
                "content": "Streaming",
                "fontSize": 15,
                "fontWeight": "bold",
                "fontDesign": "rounded",
                "color": "label",
                "lineLimit": 1
              },
              {
                "type": "text",
                "content": "Jun 1",
                "fontSize": 11,
                "fontWeight": "semibold",
                "color": "secondaryLabel",
                "lineLimit": 1
              }
            ]
          }
        ]
      },
      {
        "type": "spacer"
      },
      {
        "type": "text",
        "content": "$12.50",
        "fontSize": 22,
        "fontWeight": "bold",
        "fontDesign": "rounded",
        "color": "#3878FA",
        "alignment": "leading",
        "lineLimit": 1
      }
    ]
  },
  "medium": {
    "type": "vstack",
    "padding": 10,
    "spacing": 4,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "label",
        "text": "Upcoming",
        "systemName": "calendar.badge.clock",
        "fontSize": 11,
        "fontWeight": "bold",
        "color": "label",
        "iconColor": "#3878FA",
        "spacing": 4
      },
      {
        "type": "grid",
        "columns": 2,
        "spacing": 8,
        "rowSpacing": 6,
        "children": [
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "S",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Streaming",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 1",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$12.50",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "C",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Cloud Pro",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Today",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$9.99",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "M",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Music Plus",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Tomorrow",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$4.99",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 20,
                    "cornerRadius": 5
                  },
                  {
                    "type": "text",
                    "content": "N",
                    "fontSize": 9,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "News Daily",
                    "fontSize": 10,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 5",
                    "fontSize": 8,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$2.99",
                "fontSize": 10,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          }
        ]
      },
      {
        "type": "spacer",
        "minLength": 2
      },
      {
        "type": "text",
        "content": "+2 more",
        "fontSize": 9,
        "fontWeight": "bold",
        "color": "secondaryLabel",
        "lineLimit": 1
      }
    ]
  },
  "large": {
    "type": "vstack",
    "padding": 12,
    "spacing": 4,
    "cornerRadius": 16,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1C1C1E"
    },
    "children": [
      {
        "type": "label",
        "text": "Upcoming",
        "systemName": "calendar.badge.clock",
        "fontSize": 11,
        "fontWeight": "bold",
        "color": "label",
        "iconColor": "#3878FA",
        "spacing": 4
      },
      {
        "type": "grid",
        "columns": 2,
        "spacing": 10,
        "rowSpacing": 8,
        "children": [
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "S",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Streaming",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 1",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$12.50",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "C",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Cloud Pro",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Today",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$9.99",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "M",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Music Plus",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Tomorrow",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$4.99",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "N",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "News Daily",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 5",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$2.99",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "G",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "Gym Club",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 8",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$29.00",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          },
          {
            "type": "hstack",
            "spacing": 6,
            "alignment": "center",
            "children": [
              {
                "type": "zstack",
                "alignment": "center",
                "children": [
                  {
                    "type": "shape",
                    "shapeType": "rectangle",
                    "fill": "#1F59E0",
                    "size": 24,
                    "cornerRadius": 6
                  },
                  {
                    "type": "text",
                    "content": "V",
                    "fontSize": 11,
                    "fontWeight": "bold",
                    "color": "#FFFFFF"
                  }
                ]
              },
              {
                "type": "vstack",
                "spacing": 0,
                "alignment": "leading",
                "flex": 1,
                "children": [
                  {
                    "type": "text",
                    "content": "VPN Secure",
                    "fontSize": 11,
                    "fontWeight": "semibold",
                    "color": "label",
                    "lineLimit": 1
                  },
                  {
                    "type": "text",
                    "content": "Jun 12",
                    "fontSize": 9,
                    "color": "secondaryLabel",
                    "lineLimit": 1
                  }
                ]
              },
              {
                "type": "text",
                "content": "$6.49",
                "fontSize": 11,
                "fontWeight": "bold",
                "color": "#3878FA",
                "lineLimit": 1
              }
            ]
          }
        ]
      },
      {
        "type": "spacer",
        "minLength": 4
      },
      {
        "type": "text",
        "content": "+4 more",
        "fontSize": 9,
        "fontWeight": "bold",
        "color": "secondaryLabel",
        "lineLimit": 1
      }
    ]
  }
}
```

</details>

<Playground case="upcoming-payments.light.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxNCwKICAgICJzcGFjaW5nIjogMTAsCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAiYmFja2dyb3VuZCI6IHsKICAgICAgImxpZ2h0IjogIiNGMkYyRjciLAogICAgICAiZGFyayI6ICIjMUMxQzFFIgogICAgfSwKICAgICJjaGlsZHJlbiI6IFsKICAgICAgewogICAgICAgICJ0eXBlIjogImRpdmlkZXIiLAogICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAidGhpY2tuZXNzIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAic3BhY2luZyI6IDEwLAogICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICJzaGFwZVR5cGUiOiAicmVjdGFuZ2xlIiwKICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgInNpemUiOiA0NCwKICAgICAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiAxMAogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICJTIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDE4LAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAiZm9udERlc2lnbiI6ICJyb3VuZGVkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiAyLAogICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIlN0cmVhbWluZyIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxNSwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImZvbnREZXNpZ24iOiAicm91bmRlZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDEiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0KICAgICAgICBdCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJzcGFjZXIiCiAgICAgIH0sCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICIkMTIuNTAiLAogICAgICAgICJmb250U2l6ZSI6IDIyLAogICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICJmb250RGVzaWduIjogInJvdW5kZWQiLAogICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgIH0KICAgIF0KICB9LAogICJtZWRpdW0iOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMCwKICAgICJzcGFjaW5nIjogNCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNiwKICAgICJiYWNrZ3JvdW5kIjogewogICAgICAibGlnaHQiOiAiI0YyRjJGNyIsCiAgICAgICJkYXJrIjogIiMxQzFDMUUiCiAgICB9LAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAibGFiZWwiLAogICAgICAgICJ0ZXh0IjogIlVwY29taW5nIiwKICAgICAgICAic3lzdGVtTmFtZSI6ICJjYWxlbmRhci5iYWRnZS5jbG9jayIsCiAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAiaWNvbkNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICJzcGFjaW5nIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiZ3JpZCIsCiAgICAgICAgImNvbHVtbnMiOiAyLAogICAgICAgICJzcGFjaW5nIjogOCwKICAgICAgICAicm93U3BhY2luZyI6IDYsCiAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjAsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDUKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIlMiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogIiNGRkZGRkYiCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgICAgICAgICAic3BhY2luZyI6IDAsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAgICAgImZsZXgiOiAxLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiU3RyZWFtaW5nIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDEiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDgsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIiQxMi41MCIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgICAgICJzcGFjaW5nIjogNiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAienN0YWNrIiwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICAgICAic2hhcGVUeXBlIjogInJlY3RhbmdsZSIsCiAgICAgICAgICAgICAgICAgICAgImZpbGwiOiAiIzFGNTlFMCIsCiAgICAgICAgICAgICAgICAgICAgInNpemUiOiAyMCwKICAgICAgICAgICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiQyIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJDbG91ZCBQcm8iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDEwLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJUb2RheSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOCwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDkuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjAsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDUKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk0iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogIiNGRkZGRkYiCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgICAgICAgICAic3BhY2luZyI6IDAsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAgICAgImZsZXgiOiAxLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiTXVzaWMgUGx1cyIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTAsCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAic2VtaWJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIlRvbW9ycm93IiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiA4LAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICJzZWNvbmRhcnlMYWJlbCIsCiAgICAgICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAiY29udGVudCI6ICIkNC45OSIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgICAgICJzcGFjaW5nIjogNiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAienN0YWNrIiwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICAgICAic2hhcGVUeXBlIjogInJlY3RhbmdsZSIsCiAgICAgICAgICAgICAgICAgICAgImZpbGwiOiAiIzFGNTlFMCIsCiAgICAgICAgICAgICAgICAgICAgInNpemUiOiAyMCwKICAgICAgICAgICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiTiIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJOZXdzIERhaWx5IiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMCwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDUiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDgsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIiQyLjk5IiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDEwLAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzM4NzhGQSIsCiAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfQogICAgICAgIF0KICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInNwYWNlciIsCiAgICAgICAgIm1pbkxlbmd0aCI6IDIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICJjb250ZW50IjogIisyIG1vcmUiLAogICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAibGluZUxpbWl0IjogMQogICAgICB9CiAgICBdCiAgfSwKICAibGFyZ2UiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInBhZGRpbmciOiAxMiwKICAgICJzcGFjaW5nIjogNCwKICAgICJjb3JuZXJSYWRpdXMiOiAxNiwKICAgICJiYWNrZ3JvdW5kIjogewogICAgICAibGlnaHQiOiAiI0YyRjJGNyIsCiAgICAgICJkYXJrIjogIiMxQzFDMUUiCiAgICB9LAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAibGFiZWwiLAogICAgICAgICJ0ZXh0IjogIlVwY29taW5nIiwKICAgICAgICAic3lzdGVtTmFtZSI6ICJjYWxlbmRhci5iYWRnZS5jbG9jayIsCiAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAiaWNvbkNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICJzcGFjaW5nIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAiZ3JpZCIsCiAgICAgICAgImNvbHVtbnMiOiAyLAogICAgICAgICJzcGFjaW5nIjogMTAsCiAgICAgICAgInJvd1NwYWNpbmciOiA4LAogICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiA2LAogICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ6c3RhY2siLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAgICAgICAgICJzaGFwZVR5cGUiOiAicmVjdGFuZ2xlIiwKICAgICAgICAgICAgICAgICAgICAiZmlsbCI6ICIjMUY1OUUwIiwKICAgICAgICAgICAgICAgICAgICAic2l6ZSI6IDI0LAogICAgICAgICAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2CiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJTIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJTdHJlYW1pbmciLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJKdW4gMSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDEyLjUwIiwKICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAiY29sb3IiOiAiIzM4NzhGQSIsCiAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgIH0KICAgICAgICAgICAgXQogICAgICAgICAgfSwKICAgICAgICAgIHsKICAgICAgICAgICAgInR5cGUiOiAiaHN0YWNrIiwKICAgICAgICAgICAgInNwYWNpbmciOiA2LAogICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ6c3RhY2siLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAic2hhcGUiLAogICAgICAgICAgICAgICAgICAgICJzaGFwZVR5cGUiOiAicmVjdGFuZ2xlIiwKICAgICAgICAgICAgICAgICAgICAiZmlsbCI6ICIjMUY1OUUwIiwKICAgICAgICAgICAgICAgICAgICAic2l6ZSI6IDI0LAogICAgICAgICAgICAgICAgICAgICJjb3JuZXJSYWRpdXMiOiA2CiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJDIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAiI0ZGRkZGRiIKICAgICAgICAgICAgICAgICAgfQogICAgICAgICAgICAgICAgXQogICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAidnN0YWNrIiwKICAgICAgICAgICAgICAgICJzcGFjaW5nIjogMCwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAibGVhZGluZyIsCiAgICAgICAgICAgICAgICAiZmxleCI6IDEsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJDbG91ZCBQcm8iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJUb2RheSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDkuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjQsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk0iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAgICAgInNwYWNpbmciOiAwLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJsZWFkaW5nIiwKICAgICAgICAgICAgICAgICJmbGV4IjogMSwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk11c2ljIFBsdXMiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJUb21vcnJvdyIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDQuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjQsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk4iLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAgICAgInNwYWNpbmciOiAwLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJsZWFkaW5nIiwKICAgICAgICAgICAgICAgICJmbGV4IjogMSwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIk5ld3MgRGFpbHkiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogInNlbWlib2xkIiwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAibGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0sCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICAgICAiY29udGVudCI6ICJKdW4gNSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDIuOTkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9LAogICAgICAgICAgewogICAgICAgICAgICAidHlwZSI6ICJoc3RhY2siLAogICAgICAgICAgICAic3BhY2luZyI6IDYsCiAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInpzdGFjayIsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIsCiAgICAgICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgICAgICAgICAgICAgInNoYXBlVHlwZSI6ICJyZWN0YW5nbGUiLAogICAgICAgICAgICAgICAgICAgICJmaWxsIjogIiMxRjU5RTAiLAogICAgICAgICAgICAgICAgICAgICJzaXplIjogMjQsCiAgICAgICAgICAgICAgICAgICAgImNvcm5lclJhZGl1cyI6IDYKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIkciLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDExLAogICAgICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICIjRkZGRkZGIgogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgICAgICAgICAgICAgInNwYWNpbmciOiAwLAogICAgICAgICAgICAgICAgImFsaWdubWVudCI6ICJsZWFkaW5nIiwKICAgICAgICAgICAgICAgICJmbGV4IjogMSwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIkd5bSBDbHViIiwKICAgICAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJzZW1pYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogImxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiSnVuIDgiLAogICAgICAgICAgICAgICAgICAgICJmb250U2l6ZSI6IDksCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogInNlY29uZGFyeUxhYmVsIiwKICAgICAgICAgICAgICAgICAgICAibGluZUxpbWl0IjogMQogICAgICAgICAgICAgICAgICB9CiAgICAgICAgICAgICAgICBdCiAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAgICAgICAgICJjb250ZW50IjogIiQyOS4wMCIsCiAgICAgICAgICAgICAgICAiZm9udFNpemUiOiAxMSwKICAgICAgICAgICAgICAgICJmb250V2VpZ2h0IjogImJvbGQiLAogICAgICAgICAgICAgICAgImNvbG9yIjogIiMzODc4RkEiLAogICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICB9CiAgICAgICAgICAgIF0KICAgICAgICAgIH0sCiAgICAgICAgICB7CiAgICAgICAgICAgICJ0eXBlIjogImhzdGFjayIsCiAgICAgICAgICAgICJzcGFjaW5nIjogNiwKICAgICAgICAgICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgICAgICAgICAiY2hpbGRyZW4iOiBbCiAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgInR5cGUiOiAienN0YWNrIiwKICAgICAgICAgICAgICAgICJhbGlnbm1lbnQiOiAiY2VudGVyIiwKICAgICAgICAgICAgICAgICJjaGlsZHJlbiI6IFsKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInNoYXBlIiwKICAgICAgICAgICAgICAgICAgICAic2hhcGVUeXBlIjogInJlY3RhbmdsZSIsCiAgICAgICAgICAgICAgICAgICAgImZpbGwiOiAiIzFGNTlFMCIsCiAgICAgICAgICAgICAgICAgICAgInNpemUiOiAyNCwKICAgICAgICAgICAgICAgICAgICAiY29ybmVyUmFkaXVzIjogNgogICAgICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiViIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgICAgICAgICAgICAgImNvbG9yIjogIiNGRkZGRkYiCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInZzdGFjayIsCiAgICAgICAgICAgICAgICAic3BhY2luZyI6IDAsCiAgICAgICAgICAgICAgICAiYWxpZ25tZW50IjogImxlYWRpbmciLAogICAgICAgICAgICAgICAgImZsZXgiOiAxLAogICAgICAgICAgICAgICAgImNoaWxkcmVuIjogWwogICAgICAgICAgICAgICAgICB7CiAgICAgICAgICAgICAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiVlBOIFNlY3VyZSIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAgICAgImZvbnRXZWlnaHQiOiAic2VtaWJvbGQiLAogICAgICAgICAgICAgICAgICAgICJjb2xvciI6ICJsYWJlbCIsCiAgICAgICAgICAgICAgICAgICAgImxpbmVMaW1pdCI6IDEKICAgICAgICAgICAgICAgICAgfSwKICAgICAgICAgICAgICAgICAgewogICAgICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgICAgICJjb250ZW50IjogIkp1biAxMiIsCiAgICAgICAgICAgICAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAgICAgICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgICAgIH0KICAgICAgICAgICAgICAgIF0KICAgICAgICAgICAgICB9LAogICAgICAgICAgICAgIHsKICAgICAgICAgICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICAgICAgICAgImNvbnRlbnQiOiAiJDYuNDkiLAogICAgICAgICAgICAgICAgImZvbnRTaXplIjogMTEsCiAgICAgICAgICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAgICAgICAgICJjb2xvciI6ICIjMzg3OEZBIiwKICAgICAgICAgICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgICAgICAgICAgfQogICAgICAgICAgICBdCiAgICAgICAgICB9CiAgICAgICAgXQogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAic3BhY2VyIiwKICAgICAgICAibWluTGVuZ3RoIjogNAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiKzQgbW9yZSIsCiAgICAgICAgImZvbnRTaXplIjogOSwKICAgICAgICAiZm9udFdlaWdodCI6ICJib2xkIiwKICAgICAgICAiY29sb3IiOiAic2Vjb25kYXJ5TGFiZWwiLAogICAgICAgICJsaW5lTGltaXQiOiAxCiAgICAgIH0KICAgIF0KICB9Cn0=" />

## vstack-spacer

Sizes: small

### small

<ShotGrid case="vstack-spacer.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "spacing": 4,
    "padding": 8,
    "background": "#111827",
    "children": [
      {
        "type": "text",
        "content": "Top",
        "fontSize": 14,
        "color": "#fff"
      },
      {
        "type": "spacer"
      },
      {
        "type": "text",
        "content": "Bottom",
        "fontSize": 14,
        "color": "#fff"
      }
    ]
  }
}
```

</details>

<Playground case="vstack-spacer.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgInNwYWNpbmciOiA0LAogICAgInBhZGRpbmciOiA4LAogICAgImJhY2tncm91bmQiOiAiIzExMTgyNyIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJ0ZXh0IiwKICAgICAgICAiY29udGVudCI6ICJUb3AiLAogICAgICAgICJmb250U2l6ZSI6IDE0LAogICAgICAgICJjb2xvciI6ICIjZmZmIgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAic3BhY2VyIgogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiQm90dG9tIiwKICAgICAgICAiZm9udFNpemUiOiAxNCwKICAgICAgICAiY29sb3IiOiAiI2ZmZiIKICAgICAgfQogICAgXQogIH0KfQ==" />

## weather

Sizes: small

### small

<ShotGrid case="weather.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "children": [
      {
        "type": "text",
        "content": "72°",
        "fontSize": 36,
        "fontWeight": "bold",
        "fontDesign": null,
        "textStyle": null,
        "color": "#fff",
        "alignment": null,
        "lineLimit": null,
        "padding": null,
        "background": null,
        "cornerRadius": null,
        "opacity": null,
        "frame": null,
        "border": null,
        "shadow": null,
        "clipShape": null,
        "flex": null
      },
      {
        "type": "progress",
        "value": 0.7,
        "total": 1,
        "label": "Humidity",
        "tint": "#4CAF50",
        "color": null,
        "bar_style": null,
        "padding": null,
        "background": null,
        "cornerRadius": null,
        "opacity": null,
        "frame": null,
        "border": null,
        "shadow": null,
        "clipShape": null,
        "flex": null
      }
    ],
    "spacing": 8,
    "alignment": null,
    "padding": 12,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1a1a2e"
    },
    "cornerRadius": 16,
    "opacity": null,
    "frame": null,
    "border": null,
    "shadow": null,
    "clipShape": null,
    "flex": null
  },
  "medium": null,
  "large": null
}
```

</details>

<Playground case="weather.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiNzLCsCIsCiAgICAgICAgImZvbnRTaXplIjogMzYsCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImZvbnREZXNpZ24iOiBudWxsLAogICAgICAgICJ0ZXh0U3R5bGUiOiBudWxsLAogICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAiYWxpZ25tZW50IjogbnVsbCwKICAgICAgICAibGluZUxpbWl0IjogbnVsbCwKICAgICAgICAicGFkZGluZyI6IG51bGwsCiAgICAgICAgImJhY2tncm91bmQiOiBudWxsLAogICAgICAgICJjb3JuZXJSYWRpdXMiOiBudWxsLAogICAgICAgICJvcGFjaXR5IjogbnVsbCwKICAgICAgICAiZnJhbWUiOiBudWxsLAogICAgICAgICJib3JkZXIiOiBudWxsLAogICAgICAgICJzaGFkb3ciOiBudWxsLAogICAgICAgICJjbGlwU2hhcGUiOiBudWxsLAogICAgICAgICJmbGV4IjogbnVsbAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICJ2YWx1ZSI6IDAuNywKICAgICAgICAidG90YWwiOiAxLAogICAgICAgICJsYWJlbCI6ICJIdW1pZGl0eSIsCiAgICAgICAgInRpbnQiOiAiIzRDQUY1MCIsCiAgICAgICAgImNvbG9yIjogbnVsbCwKICAgICAgICAiYmFyX3N0eWxlIjogbnVsbCwKICAgICAgICAicGFkZGluZyI6IG51bGwsCiAgICAgICAgImJhY2tncm91bmQiOiBudWxsLAogICAgICAgICJjb3JuZXJSYWRpdXMiOiBudWxsLAogICAgICAgICJvcGFjaXR5IjogbnVsbCwKICAgICAgICAiZnJhbWUiOiBudWxsLAogICAgICAgICJib3JkZXIiOiBudWxsLAogICAgICAgICJzaGFkb3ciOiBudWxsLAogICAgICAgICJjbGlwU2hhcGUiOiBudWxsLAogICAgICAgICJmbGV4IjogbnVsbAogICAgICB9CiAgICBdLAogICAgInNwYWNpbmciOiA4LAogICAgImFsaWdubWVudCI6IG51bGwsCiAgICAicGFkZGluZyI6IDEyLAogICAgImJhY2tncm91bmQiOiB7CiAgICAgICJsaWdodCI6ICIjRjJGMkY3IiwKICAgICAgImRhcmsiOiAiIzFhMWEyZSIKICAgIH0sCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAib3BhY2l0eSI6IG51bGwsCiAgICAiZnJhbWUiOiBudWxsLAogICAgImJvcmRlciI6IG51bGwsCiAgICAic2hhZG93IjogbnVsbCwKICAgICJjbGlwU2hhcGUiOiBudWxsLAogICAgImZsZXgiOiBudWxsCiAgfSwKICAibWVkaXVtIjogbnVsbCwKICAibGFyZ2UiOiBudWxsCn0=" />

## weather.light

Sizes: small

### small

<ShotGrid case="weather.light.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "vstack",
    "children": [
      {
        "type": "text",
        "content": "72°",
        "fontSize": 36,
        "fontWeight": "bold",
        "fontDesign": null,
        "textStyle": null,
        "color": "#fff",
        "alignment": null,
        "lineLimit": null,
        "padding": null,
        "background": null,
        "cornerRadius": null,
        "opacity": null,
        "frame": null,
        "border": null,
        "shadow": null,
        "clipShape": null,
        "flex": null
      },
      {
        "type": "progress",
        "value": 0.7,
        "total": 1,
        "label": "Humidity",
        "tint": "#4CAF50",
        "color": null,
        "bar_style": null,
        "padding": null,
        "background": null,
        "cornerRadius": null,
        "opacity": null,
        "frame": null,
        "border": null,
        "shadow": null,
        "clipShape": null,
        "flex": null
      }
    ],
    "spacing": 8,
    "alignment": null,
    "padding": 12,
    "background": {
      "light": "#F2F2F7",
      "dark": "#1a1a2e"
    },
    "cornerRadius": 16,
    "opacity": null,
    "frame": null,
    "border": null,
    "shadow": null,
    "clipShape": null,
    "flex": null
  },
  "medium": null,
  "large": null
}
```

</details>

<Playground case="weather.light.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ2c3RhY2siLAogICAgImNoaWxkcmVuIjogWwogICAgICB7CiAgICAgICAgInR5cGUiOiAidGV4dCIsCiAgICAgICAgImNvbnRlbnQiOiAiNzLCsCIsCiAgICAgICAgImZvbnRTaXplIjogMzYsCiAgICAgICAgImZvbnRXZWlnaHQiOiAiYm9sZCIsCiAgICAgICAgImZvbnREZXNpZ24iOiBudWxsLAogICAgICAgICJ0ZXh0U3R5bGUiOiBudWxsLAogICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAiYWxpZ25tZW50IjogbnVsbCwKICAgICAgICAibGluZUxpbWl0IjogbnVsbCwKICAgICAgICAicGFkZGluZyI6IG51bGwsCiAgICAgICAgImJhY2tncm91bmQiOiBudWxsLAogICAgICAgICJjb3JuZXJSYWRpdXMiOiBudWxsLAogICAgICAgICJvcGFjaXR5IjogbnVsbCwKICAgICAgICAiZnJhbWUiOiBudWxsLAogICAgICAgICJib3JkZXIiOiBudWxsLAogICAgICAgICJzaGFkb3ciOiBudWxsLAogICAgICAgICJjbGlwU2hhcGUiOiBudWxsLAogICAgICAgICJmbGV4IjogbnVsbAogICAgICB9LAogICAgICB7CiAgICAgICAgInR5cGUiOiAicHJvZ3Jlc3MiLAogICAgICAgICJ2YWx1ZSI6IDAuNywKICAgICAgICAidG90YWwiOiAxLAogICAgICAgICJsYWJlbCI6ICJIdW1pZGl0eSIsCiAgICAgICAgInRpbnQiOiAiIzRDQUY1MCIsCiAgICAgICAgImNvbG9yIjogbnVsbCwKICAgICAgICAiYmFyX3N0eWxlIjogbnVsbCwKICAgICAgICAicGFkZGluZyI6IG51bGwsCiAgICAgICAgImJhY2tncm91bmQiOiBudWxsLAogICAgICAgICJjb3JuZXJSYWRpdXMiOiBudWxsLAogICAgICAgICJvcGFjaXR5IjogbnVsbCwKICAgICAgICAiZnJhbWUiOiBudWxsLAogICAgICAgICJib3JkZXIiOiBudWxsLAogICAgICAgICJzaGFkb3ciOiBudWxsLAogICAgICAgICJjbGlwU2hhcGUiOiBudWxsLAogICAgICAgICJmbGV4IjogbnVsbAogICAgICB9CiAgICBdLAogICAgInNwYWNpbmciOiA4LAogICAgImFsaWdubWVudCI6IG51bGwsCiAgICAicGFkZGluZyI6IDEyLAogICAgImJhY2tncm91bmQiOiB7CiAgICAgICJsaWdodCI6ICIjRjJGMkY3IiwKICAgICAgImRhcmsiOiAiIzFhMWEyZSIKICAgIH0sCiAgICAiY29ybmVyUmFkaXVzIjogMTYsCiAgICAib3BhY2l0eSI6IG51bGwsCiAgICAiZnJhbWUiOiBudWxsLAogICAgImJvcmRlciI6IG51bGwsCiAgICAic2hhZG93IjogbnVsbCwKICAgICJjbGlwU2hhcGUiOiBudWxsLAogICAgImZsZXgiOiBudWxsCiAgfSwKICAibWVkaXVtIjogbnVsbCwKICAibGFyZ2UiOiBudWxsCn0=" />

## zstack-align

Sizes: small

### small

<ShotGrid case="zstack-align.small" />

<details>
<summary>Config JSON</summary>

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "version": 1,
  "small": {
    "type": "zstack",
    "alignment": "center",
    "background": "#1e293b",
    "children": [
      {
        "type": "shape",
        "shapeType": "circle",
        "size": 40,
        "fill": "#334155"
      },
      {
        "type": "text",
        "content": "Z",
        "fontSize": 18,
        "color": "#fff",
        "alignment": "center"
      }
    ]
  }
}
```

</details>

<Playground case="zstack-align.small" size="small" config-b64="ewogICIkc2NoZW1hIjogImh0dHBzOi8vczAwZC5naXRodWIuaW8vdGF1cmktcGx1Z2luLXdpZGdldHMvc2NoZW1hcy93aWRnZXQtY29uZmlnLnYxLmpzb24iLAogICJ2ZXJzaW9uIjogMSwKICAic21hbGwiOiB7CiAgICAidHlwZSI6ICJ6c3RhY2siLAogICAgImFsaWdubWVudCI6ICJjZW50ZXIiLAogICAgImJhY2tncm91bmQiOiAiIzFlMjkzYiIsCiAgICAiY2hpbGRyZW4iOiBbCiAgICAgIHsKICAgICAgICAidHlwZSI6ICJzaGFwZSIsCiAgICAgICAgInNoYXBlVHlwZSI6ICJjaXJjbGUiLAogICAgICAgICJzaXplIjogNDAsCiAgICAgICAgImZpbGwiOiAiIzMzNDE1NSIKICAgICAgfSwKICAgICAgewogICAgICAgICJ0eXBlIjogInRleHQiLAogICAgICAgICJjb250ZW50IjogIloiLAogICAgICAgICJmb250U2l6ZSI6IDE4LAogICAgICAgICJjb2xvciI6ICIjZmZmIiwKICAgICAgICAiYWxpZ25tZW50IjogImNlbnRlciIKICAgICAgfQogICAgXQogIH0KfQ==" />

<!-- /generated:showcase -->
