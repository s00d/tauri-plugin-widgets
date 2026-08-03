//! SF Symbol → cross-platform icon resolve (Material ligature + emoji fallback).
//!
//! Not SF Symbols outside Apple — similar glyphs only. Unknown names never fall
//! back to a random first letter (that looked broken); use a neutral bullet.

use std::sync::OnceLock;

/// Resolved icon for non-Apple hosts.
#[derive(Debug, Clone, Copy)]
pub struct IconRef {
    /// Material Symbols ligature / icon name (e.g. `favorite`).
    pub material: &'static str,
    /// Emoji / unicode glyph fallback when a font is unavailable.
    pub emoji: &'static str,
}

/// Resolve an SF Symbol-ish name to a Material + emoji pair.
pub fn resolve_sf_symbol(name: &str) -> IconRef {
    let n = normalize(name);
    if let Some(r) = table().get(n.as_str()).copied() {
        return r;
    }
    // Heuristic: cloud.sun.fill → cloud_sun / favorite-style material names.
    let material = heuristic_material(&n);
    IconRef {
        material: intern_material(material),
        emoji: "•",
    }
}

/// Emoji/glyph used by Adaptive Cards TextBlock and Desktop when no PNG bake.
pub fn sf_symbol_emoji(name: &str) -> &'static str {
    resolve_sf_symbol(name).emoji
}

/// Material Symbols ligature name.
pub fn sf_symbol_material(name: &str) -> &'static str {
    resolve_sf_symbol(name).material
}

fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn heuristic_material(n: &str) -> String {
    let base = n
        .trim_end_matches(".fill")
        .trim_end_matches(".circle")
        .trim_end_matches(".square")
        .replace(['.', '-'], "_");
    if base.is_empty() {
        "circle".into()
    } else {
        base
    }
}

fn intern_material(s: String) -> &'static str {
    // Leak rare heuristic strings so IconRef stays 'static (widgets are long-lived).
    Box::leak(s.into_boxed_str())
}

static LOOKUP: OnceLock<std::collections::HashMap<&'static str, IconRef>> = OnceLock::new();

fn table() -> &'static std::collections::HashMap<&'static str, IconRef> {
    LOOKUP.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        for (sf, material, emoji) in ENTRIES {
            m.insert(*sf, IconRef { material, emoji });
        }
        m
    })
}

/// Curated SF → Material Symbols + emoji. Expand as presets need.
const ENTRIES: &[(&str, &str, &str)] = &[
    ("star", "star", "⭐"),
    ("star.fill", "star", "⭐"),
    ("heart", "favorite", "❤"),
    ("heart.fill", "favorite", "❤"),
    ("person", "person", "👤"),
    ("person.fill", "person", "👤"),
    ("person.2.fill", "group", "👥"),
    ("person.badge.plus", "person_add", "👤"),
    ("gear", "settings", "⚙"),
    ("gearshape", "settings", "⚙"),
    ("gearshape.fill", "settings", "⚙"),
    ("checkmark", "check", "✓"),
    ("checkmark.circle", "check_circle", "✓"),
    ("checkmark.circle.fill", "check_circle", "✓"),
    ("xmark", "close", "✕"),
    ("xmark.circle", "cancel", "✕"),
    ("plus", "add", "+"),
    ("plus.circle", "add_circle", "+"),
    ("minus", "remove", "−"),
    ("minus.circle", "do_not_disturb_on", "−"),
    ("bell", "notifications", "🔔"),
    ("bell.fill", "notifications", "🔔"),
    ("house", "home", "⌂"),
    ("house.fill", "home", "⌂"),
    ("magnifyingglass", "search", "🔍"),
    ("calendar", "calendar_today", "📅"),
    ("calendar.badge.clock", "event", "📅"),
    ("calendar.circle", "calendar_today", "📅"),
    ("clock", "schedule", "⏱"),
    ("clock.fill", "schedule", "⏱"),
    ("globe", "public", "🌐"),
    ("cloud.fill", "cloud", "☁"),
    ("cloud.sun.fill", "partly_cloudy_day", "⛅"),
    ("cloud.rain.fill", "rainy", "☔"),
    ("cloud.bolt.fill", "thunderstorm", "⚡"),
    ("sun.max", "wb_sunny", "☀"),
    ("sun.max.fill", "wb_sunny", "☀"),
    ("moon.fill", "dark_mode", "☾"),
    ("moon.stars.fill", "clear_night", "☾"),
    ("bolt", "bolt", "⚡"),
    ("bolt.fill", "bolt", "⚡"),
    ("location.fill", "location_on", "⌂"),
    ("flame.fill", "local_fire_department", "🔥"),
    ("drop.fill", "water_drop", "●"),
    ("leaf.fill", "eco", "✿"),
    ("eye.fill", "visibility", "◉"),
    ("bubble.left.fill", "chat_bubble", "◑"),
    ("music.note", "music_note", "♪"),
    ("music.note.list", "queue_music", "♪"),
    ("hourglass", "hourglass_empty", "⏳"),
    ("hourglass.bottomhalf.filled", "hourglass_bottom", "⏳"),
    ("figure.run", "directions_run", "➤"),
    ("paintpalette.fill", "palette", "⚘"),
    ("quote.opening", "format_quote", "“"),
    ("bitcoinsign.circle.fill", "currency_bitcoin", "₿"),
    ("battery.100", "battery_full", "▮"),
    ("iphone", "phone_iphone", "▯"),
    ("ipad", "tablet_mac", "▭"),
    ("applewatch", "watch", "⌚"),
    ("airpodspro", "headphones", "○"),
    ("desktopcomputer", "desktop_windows", "▣"),
    ("internaldrive", "hard_drive", "■"),
    ("network", "lan", "≡"),
    ("envelope.fill", "mail", "✉"),
    ("envelope", "mail", "✉"),
    ("phone.fill", "call", "☎"),
    ("phone", "call", "☎"),
    ("message.fill", "message", "💬"),
    ("trash", "delete", "🗑"),
    ("trash.fill", "delete", "🗑"),
    ("folder", "folder", "📁"),
    ("folder.fill", "folder", "📁"),
    ("doc", "description", "📄"),
    ("doc.fill", "description", "📄"),
    ("photo", "photo", "🖼"),
    ("photo.fill", "photo", "🖼"),
    ("camera", "photo_camera", "📷"),
    ("camera.fill", "photo_camera", "📷"),
    ("map", "map", "🗺"),
    ("map.fill", "map", "🗺"),
    ("cart", "shopping_cart", "🛒"),
    ("cart.fill", "shopping_cart", "🛒"),
    ("creditcard", "credit_card", "💳"),
    ("creditcard.fill", "credit_card", "💳"),
    ("wifi", "wifi", "📶"),
    ("antenna.radiowaves.left.and.right", "cell_tower", "📡"),
    ("lock.fill", "lock", "🔒"),
    ("lock", "lock", "🔒"),
    ("lock.open.fill", "lock_open", "🔓"),
    ("key.fill", "key", "🔑"),
    ("paperplane.fill", "send", "➤"),
    ("arrow.right", "arrow_forward", "→"),
    ("arrow.left", "arrow_back", "←"),
    ("arrow.up", "arrow_upward", "↑"),
    ("arrow.down", "arrow_downward", "↓"),
    ("chevron.right", "chevron_right", "›"),
    ("chevron.left", "chevron_left", "‹"),
    ("chevron.up", "expand_less", "ˆ"),
    ("chevron.down", "expand_more", "ˇ"),
    ("info.circle", "info", "ℹ"),
    ("info.circle.fill", "info", "ℹ"),
    ("exclamationmark.triangle", "warning", "⚠"),
    ("exclamationmark.triangle.fill", "warning", "⚠"),
    ("play.fill", "play_arrow", "▶"),
    ("pause.fill", "pause", "⏸"),
    ("stop.fill", "stop", "⏹"),
    ("forward.fill", "fast_forward", "⏩"),
    ("backward.fill", "fast_rewind", "⏪"),
];

/// Tiny SVG that draws the emoji/glyph centered (works with resvg for Windows Board).
pub fn icon_svg(name: &str, size: f64, color: &str) -> String {
    let glyph = sf_symbol_emoji(name);
    let s = size.max(12.0);
    let fs = s * 0.72;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{s}" height="{s}" viewBox="0 0 {s} {s}">
  <text x="50%" y="54%" dominant-baseline="middle" text-anchor="middle"
        font-size="{fs}" fill="{color}">{glyph}</text>
</svg>"##
    )
}

/// Rasterize an SF-ish name to a PNG `data:` URI (`rasterize` feature).
#[cfg(feature = "rasterize")]
pub fn icon_png_data_uri(name: &str, size: f64, color: &str) -> Result<String, String> {
    crate::rasterize::svg_to_data_uri(&icon_svg(name, size, color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heart_maps_to_favorite() {
        let r = resolve_sf_symbol("heart.fill");
        assert_eq!(r.material, "favorite");
        assert_ne!(r.emoji, "h");
    }

    #[test]
    fn unknown_not_first_letter() {
        let r = resolve_sf_symbol("totally.unknown.glyph");
        assert_eq!(r.emoji, "•");
        assert!(!r.material.is_empty());
    }
}
