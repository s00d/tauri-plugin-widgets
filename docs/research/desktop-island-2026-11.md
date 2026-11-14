# Desktop "Dynamic Island" for live activities, November 2026

Goal: desktop live activities that look and behave like the iPhone Dynamic Island — at the
camera notch on macOS, an equivalent pill on Windows and Linux. Code read (shallow clones,
outside the repo): boring.notch, DynamicNotchKit, NotchDrop, MewNotch. Web sources for the
rest. Result: `activitySurface: "island"` (default), `src/desktop/island.rs`,
`widget-src/island.ts`, guide section [Desktop island](/guide/recipes/live-activities#desktop-island).

## macOS notch apps

| Project | Notch metrics | Window | Hover / click-through |
| --- | --- | --- | --- |
| [boring.notch](https://github.com/TheBoredTeam/boring.notch) (`sizing/matters.swift`, `components/Notch/BoringNotchWindow.swift`) | width = `frame.width - auxiliaryTopLeftArea.width - auxiliaryTopRightArea.width (+4)`, height = `safeAreaInsets.top` (or menu-bar height = `frame.maxY - visibleFrame.maxY`); 185 × 32 fallback | `NSPanel` `[.borderless, .nonactivatingPanel, .utilityWindow, .hudWindow]`, `level = .mainMenu + 3`, `collectionBehavior = [.fullScreenAuxiliary, .stationary, .canJoinAllSpaces, .ignoresCycle]`, clear background, no shadow, `canBecomeKey = false` | fixed 640 × 190 window, SwiftUI `onHover`; `NSEvent.mouseLocation` + global monitors for drag detection; closed corner radii 6 / 14, open 19 / 24 |
| [DynamicNotchKit](https://github.com/MrKai77/DynamicNotchKit) (`Utility/NSScreen+Extensions.swift`, `DynamicNotchPanel.swift`) | same formula; `hasNotch` = both auxiliary areas present | `[.borderless, .nonactivatingPanel]`, `level = .screenSaver`, `[.canJoinAllSpaces, .stationary]`; half-screen window anchored to the top | `.floating` style (pill under the menu bar) on screens without a notch; compact mode needs a notch; rebuilds on `didChangeScreenParametersNotification` |
| [NotchDrop](https://github.com/Lakr233/NotchDrop) (`Ext+NSScreen.swift`, `NotchWindow.swift`) | `safeAreaInsets.top`, auxiliary widths | `level = .statusBar + 8`, all-Spaces / stationary / full-screen auxiliary | drop target over the notch |
| [MewNotch](https://github.com/monuk7735/MewNotch) (`Utils/NotchUtils.swift`, `View/Common/MewWindow.swift`) | same formula | `level = .mainMenu + 1`, `.nonactivatingPanel`; a second panel with `ignoresMouseEvents = true` for the lock screen | per-screen windows for multiple displays |
| NotchNook / Alcove (closed source) | — | behaviour: hover grows the notch into a panel, collapses on leave; on non-notch Macs a fake notch under the menu bar | — |

Apple guidance on levels and Spaces: [NSWindow.CollectionBehavior](https://developer.apple.com/documentation/appkit/nswindow/collectionbehavior-swift.struct),
forum notes that `.mainMenu + 1` alone did not cover full-screen apps without
`fullScreenAuxiliary` ([thread](https://developer.apple.com/forums/thread/716741)).

**Taken:** notch from `safeAreaInsets.top` + auxiliary areas (via `objc_msgSend`, Apple
silicon only — NSRect returns in registers there; Intel Macs have no notch); level
`mainMenu + 3` and `canJoinAllSpaces | stationary | ignoresCycle | fullScreenAuxiliary`,
no shadow, non-focusable (tao `focusable(false)` = `canBecomeKey` false); pill-under-menu-bar
fallback without a notch.

**Changed:** the window is resized to the visible shapes instead of a large transparent
panel — a WKWebView swallows clicks on its transparent pixels, so a big window would block
the menu bar. **Learned on the device:** a WKWebView in an inactive app gets no
`mouseenter`, so hover comes from the host sampling the global cursor (as boring.notch does
with `NSEvent.mouseLocation`), with the page's own hover / click as the fallback.

## macOS 26 Live Activities in the menu bar (visual reference)

macOS Tahoe 26 shows the nearby iPhone's Live Activities in the Mac menu bar: the compact
presentation looks like the Dynamic Island, a click expands it to the full activity, and the
Lock Screen view opens the app through iPhone Mirroring
([Apple newsroom](https://www.apple.com/newsroom/2025/06/macos-tahoe-26-makes-the-mac-more-capable-productive-and-intelligent-than-ever/),
[Apple support](https://support.apple.com/en-us/120684),
[9to5Mac](https://9to5mac.com/2025/06/13/live-activities-are-coming-to-ipad-and-mac-heres-how-theyll-work/)).
Reviewers note several activities crowd the menu bar — one reason for minimal circles here.
There is no public API for a Mac app to post there.

## Windows

- [DynamicWin](https://github.com/FlorianButz/DynamicWin) — C# / WPF + SkiaSharp top-centre
  island (media, timers, widgets); Ctrl+Win hides it.
- [WinIsland](https://alternativeto.net/software/winisland/about/), Windhawk island mods,
  [FluentFlyout](https://github.com/unchihugo/FluentFlyout) /
  [ModernFlyouts](https://github.com/ModernFlyouts-Community/ModernFlyouts) — topmost tool
  windows (`WS_EX_TOOLWINDOW`, `WS_EX_NOACTIVATE`), no taskbar button, per-monitor DPI.

**Taken:** tao already maps the pieces: `always_on_top` (topmost), `skip_taskbar`,
`focusable(false)` = `WS_EX_NOACTIVATE`; position in logical px from the primary monitor's
work area (DPI via the monitor scale factor).

## Linux

- GNOME: [NexNotch](https://github.com/NexVar/NexNotch) (pill replacing the clock, hover
  hub), [gnome-dynamic-island](https://github.com/HIMURAw/gnome-dynamic-island) — shell
  extensions; Mutter has no wlr-layer-shell, so an app cannot do that from outside.
- wlroots / Hyprland: [ActivSpot](https://github.com/Devvvmn/ActivSpot) (Quickshell panels),
  eww / AGS bars — layer-shell surfaces.

**Taken:** X11 `_NET_WM_WINDOW_TYPE_DOCK` (above, all desktops, unfocused); Wayland with
feature `layer-shell`: `Layer::Overlay`, anchored to the top edge only so the compositor keeps
it centred while it resizes. GNOME Wayland without layer-shell: ordinary always-on-top window.

## Not done

- Following the display with the mouse / per-display islands (boring.notch, MewNotch):
  the island sits on the primary display.
- Native expand animation (`setFrame:display:animate:`): the window resizes at once, the
  pill animates with CSS inside it.
- Windows / Linux runtime were not run (only built for macOS); Wayland GNOME has no overlay.
