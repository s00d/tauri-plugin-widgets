//! Live activities: a short-lived status surface for an ongoing task.
//!
//! | Platform | Surface |
//! |---|---|
//! | iOS 16.2+ | Live Activity (Lock Screen, Dynamic Island) — `layout` drawn by `TauriLiveActivity` |
//! | Android 8+ | ongoing notification; Android 16 promotes it to a Live Update chip |
//! | macOS / Windows / Linux | Dynamic-Island pill at the top centre (`activitySurface`, default `island`) |
//!
//! One request shape everywhere. Without `layout`, the plugin builds one from `title`,
//! `body`, `progress` and `endsAt` ([`ActivityContent::effective_layout`]), so a plain
//! request looks alike on every OS.

use serde::{Deserialize, Serialize};

#[cfg(feature = "schema")]
use schemars::JsonSchema;

use super::{
    ElementStyle, HStackElement, ImageElement, ProgressElement, SpacerElement, TextElement,
    TimerElement, VStackElement, WidgetElement,
};

/// ActivityKit drops content states over 4 KiB; the same budget applies on every platform
/// so a request that works on desktop does not fail on an iPhone.
pub const MAX_ACTIVITY_BYTES: usize = 4096;

/// Start or update request for a live activity.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ActivityContent {
    /// Your key for this activity (letters, digits, `.`, `_`, `-`). Starting an id that is
    /// already running updates it.
    pub id: String,
    /// Main line (notification title on Android). Required unless a `template` gives it.
    #[serde(default)]
    pub title: String,
    /// Second line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Progress `0..=1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    /// Android 16 Live Updates `ProgressStyle` track: relative lengths and colors (one
    /// `tint` segment when omitted). Other platforms draw a plain bar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_segments: Option<Vec<ProgressSegment>>,
    /// Android 16 Live Updates `ProgressStyle` markers along the track.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_points: Option<Vec<ProgressPoint>>,
    /// Unix ms the countdown runs to (live timer; Android status-bar chip countdown).
    #[serde(rename = "endsAt", default, skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<u64>,
    /// SF Symbol name (mapped to Material / emoji off Apple).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Accent color (hex) for the icon, progress and Android notification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<String>,
    /// Full layout (Lock Screen, expanded Dynamic Island, desktop window).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<WidgetElement>,
    /// Dynamic Island compact, leading side.
    #[serde(
        rename = "compactLeading",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compact_leading: Option<WidgetElement>,
    /// Dynamic Island compact, trailing side.
    #[serde(
        rename = "compactTrailing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compact_trailing: Option<WidgetElement>,
    /// Dynamic Island minimal (several activities at once).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimal: Option<WidgetElement>,
    /// iOS Lock Screen layout when it should differ from `layout` (the expanded island).
    #[serde(
        rename = "lockScreen",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lock_screen: Option<WidgetElement>,
    /// Expanded Dynamic Island, leading region.
    #[serde(
        rename = "islandLeading",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub island_leading: Option<WidgetElement>,
    /// Expanded Dynamic Island, trailing region.
    #[serde(
        rename = "islandTrailing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub island_trailing: Option<WidgetElement>,
    /// Expanded Dynamic Island, center region (under the camera).
    #[serde(
        rename = "islandCenter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub island_center: Option<WidgetElement>,
    /// Dynamic Island keyline color (hex).
    #[serde(
        rename = "keylineTint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub keyline_tint: Option<String>,
    /// Lock Screen background color (hex) when the layout has none.
    #[serde(
        rename = "backgroundTint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub background_tint: Option<String>,
    /// Android: small text after the app name (notification sub text).
    #[serde(rename = "subText", default, skip_serializing_if = "Option::is_none")]
    pub sub_text: Option<String>,
    /// Android 16: short text for the promoted status-bar chip (≤ 7 characters).
    #[serde(
        rename = "shortCriticalText",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub short_critical_text: Option<String>,
    /// Unix ms the activity started: a stopwatch counts up from it (when `endsAt` is unset).
    #[serde(rename = "startedAt", default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    /// Android lock-screen visibility: `public` (default), `private`, `secret`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// Android notification category (`progress` default, `navigation`, `transport`,
    /// `workout`, `status`, `alarm`, `service`, `event`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Buttons (Android notification actions, at most 3). Taps arrive as `widget-action`
    /// with `widgetId = "activity.<id>"`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ActivityAction>,
    /// Deep link opened on tap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Unix ms after which the OS marks the content outdated (iOS `staleDate`).
    #[serde(rename = "staleAt", default, skip_serializing_if = "Option::is_none")]
    pub stale_at: Option<u64>,
    /// iOS: request an APNs push token (`activity-token` event, `pushToken` in the result).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub push: bool,
    /// Name of a layout stored with `registerActivityTemplate`: the device fills it with
    /// `data` (`{{data.eta}}`, `when`, `repeat`, …), so updates carry only data. Fields
    /// set on this content override the template's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// iOS: which of several activities the system shows first (higher wins).
    #[serde(
        rename = "relevanceScore",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub relevance_score: Option<f64>,
    /// iOS: alert shown with this update (lights the screen, plays `sound`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert: Option<ActivityAlert>,
    /// Unix ms the content was produced: an older update than the shown one is dropped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    /// Values for `{{data.…}}` in the `template`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<serde_json::Map<String, serde_json::Value>>")
    )]
    pub data: Option<serde_json::Value>,
}

/// Alert of a live activity update (iOS `AlertConfiguration`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ActivityAlert {
    /// Alert title (Apple Watch / notification banner).
    pub title: String,
    /// Alert body.
    pub body: String,
    /// Sound file name in the app bundle; system default when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
}

/// A button of a live activity.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ActivityAction {
    /// Action verb delivered in `widget-action`.
    pub id: String,
    /// Button text.
    pub label: String,
    /// Icon (SF Symbol; Material / emoji elsewhere).
    #[serde(
        rename = "systemName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub system_name: Option<String>,
    /// Open this deep link instead of queuing a `widget-action`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// What `start_activity` returns.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityHandle {
    /// The request id.
    pub id: String,
    /// `ios` | `android` | `desktop`.
    pub surface: String,
    /// iOS APNs token (hex), when `push` was requested and already known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_token: Option<String>,
}

/// Widget id under which a desktop activity window stores its config.
pub fn activity_widget_id(id: &str) -> String {
    format!("activity.{id}")
}

impl ActivityContent {
    /// Reject what no platform can show: empty id or title, bad charset, too many actions,
    /// progress out of range, payload over [`MAX_ACTIVITY_BYTES`].
    pub fn validate(&self) -> crate::Result<()> {
        crate::group::validate_widget_id(&self.id)?;
        if let Some(t) = self.template.as_deref() {
            crate::group::validate_widget_id(t)?;
        }
        // A templated request may take its title from the template.
        if self.template.is_none() && self.title.trim().is_empty() {
            return Err(crate::Error::new("activity title must not be empty"));
        }
        if self.actions.len() > 3 {
            return Err(crate::Error::new("an activity has at most 3 actions"));
        }
        if let Some(p) = self.progress {
            if !(0.0..=1.0).contains(&p) {
                return Err(crate::Error::new("activity progress must be within 0..=1"));
            }
        }
        if let Some(v) = self.visibility.as_deref() {
            if !matches!(v, "public" | "private" | "secret") {
                return Err(crate::Error::new(
                    "activity visibility must be public, private or secret",
                ));
            }
        }
        if let Some(c) = self.category.as_deref() {
            const CATS: [&str; 8] = [
                "progress",
                "navigation",
                "transport",
                "workout",
                "status",
                "alarm",
                "service",
                "event",
            ];
            if !CATS.contains(&c) {
                return Err(crate::Error::new(format!(
                    "activity category must be one of {}",
                    CATS.join(", ")
                )));
            }
        }
        let bytes = serde_json::to_vec(self)?.len();
        if bytes > MAX_ACTIVITY_BYTES {
            return Err(crate::Error::new(format!(
                "activity payload is {bytes} bytes; the limit is {MAX_ACTIVITY_BYTES} (ActivityKit)"
            )));
        }
        Ok(())
    }

    /// The request with `layout` filled by [`Self::effective_layout`]: what every surface
    /// receives, so a native renderer never needs its own default.
    pub fn filled(mut self) -> Self {
        self.layout = Some(self.effective_layout());
        self
    }

    /// `layout`, or one built from the plain fields: icon + title, body, timer, progress.
    pub fn effective_layout(&self) -> WidgetElement {
        if let Some(l) = &self.layout {
            return l.clone();
        }
        let tint = self.tint.clone().map(super::ColorValue::Solid);
        let mut head: Vec<WidgetElement> = Vec::new();
        if let Some(icon) = &self.icon {
            head.push(WidgetElement::Image(ImageElement {
                system_name: Some(icon.clone()),
                size: Some(18.0),
                color: tint.clone(),
                ..Default::default()
            }));
        }
        head.push(WidgetElement::Text(TextElement {
            content: self.title.clone(),
            font_size: Some(15.0),
            font_weight: Some(super::FontWeight::Semibold),
            line_limit: Some(1),
            ..Default::default()
        }));
        // Countdown to `endsAt`, else a stopwatch from `startedAt`.
        let timer = match (self.ends_at, self.started_at) {
            (Some(end), _) => Some((end, super::TimerCounting::Down)),
            (None, Some(start)) => Some((start, super::TimerCounting::Up)),
            _ => None,
        };
        if let Some((at, counting)) = timer {
            head.push(WidgetElement::Spacer(SpacerElement::default()));
            head.push(WidgetElement::Timer(TimerElement {
                target_date: iso8601_utc(at),
                counting: Some(counting),
                font_size: Some(15.0),
                ..Default::default()
            }));
        }
        let mut rows = vec![WidgetElement::HStack(HStackElement {
            children: head,
            spacing: Some(8.0),
            ..Default::default()
        })];
        if let Some(body) = &self.body {
            rows.push(WidgetElement::Text(TextElement {
                content: body.clone(),
                font_size: Some(13.0),
                line_limit: Some(2),
                ..Default::default()
            }));
        }
        if let Some(p) = self.progress {
            rows.push(WidgetElement::Progress(ProgressElement {
                value: p,
                tint,
                ..Default::default()
            }));
        }
        if !self.actions.is_empty() {
            rows.push(WidgetElement::HStack(HStackElement {
                children: self
                    .actions
                    .iter()
                    .map(|a| {
                        WidgetElement::Button(super::ButtonElement {
                            label: a.label.clone(),
                            action: a.url.is_none().then(|| a.id.clone()),
                            url: a.url.clone(),
                            system_name: a.system_name.clone(),
                            ..Default::default()
                        })
                    })
                    .collect(),
                spacing: Some(8.0),
                ..Default::default()
            }));
        }
        WidgetElement::VStack(VStackElement {
            children: rows,
            spacing: Some(6.0),
            alignment: Some(super::HorizontalAlignment::Leading),
            justify: None,
            style: ElementStyle {
                padding: Some(super::PaddingValue::Uniform(12.0)),
                ..Default::default()
            },
        })
    }
}

/// A templated request with the stored template applied: the template (read by `lookup`
/// from `activity-template:{name}`) is resolved with `data` like a widget config
/// (`{{data.x}}`, `when`, `repeat`, …), then fields set on `content` override it. Returns
/// `content` unchanged without `template`; errors when the template is missing.
pub fn resolve_activity(
    content: ActivityContent,
    lookup: impl Fn(&str) -> Option<String>,
) -> crate::Result<ActivityContent> {
    let Some(name) = content.template.clone() else {
        return Ok(content);
    };
    let raw = lookup(&crate::store::activity_template_key(&name))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            crate::Error::new(format!(
                "activity template \"{name}\" is not registered (registerActivityTemplate)"
            ))
        })?;
    let template: serde_json::Value = serde_json::from_str(&raw)?;
    // No `entries` here, so the time is never read.
    let ctx = crate::resolve::ResolveContext {
        data: content.data.as_ref(),
        ..Default::default()
    };
    let mut merged = match crate::resolve::resolve_fields(&template, &ctx) {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };
    let serde_json::Value::Object(own) = serde_json::to_value(&content)? else {
        return Ok(content);
    };
    for (k, v) in own {
        let empty_title = k == "title" && v.as_str().is_some_and(|t| t.trim().is_empty());
        if !(empty_title || k == "template" || k == "data") {
            merged.insert(k, v);
        }
    }
    merged
        .entry("title")
        .or_insert_with(|| serde_json::Value::String(String::new()));
    Ok(serde_json::from_value(serde_json::Value::Object(merged))?)
}

/// `YYYY-MM-DDTHH:MM:SSZ` for Unix ms (the timer element takes ISO-8601).
pub fn iso8601_utc(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn base() -> ActivityContent {
        ActivityContent {
            id: "order.42".into(),
            title: "Order".into(),
            ..Default::default()
        }
    }

    fn action(id: &str, label: &str) -> ActivityAction {
        ActivityAction {
            id: id.into(),
            label: label.into(),
            ..Default::default()
        }
    }

    fn layout(c: &ActivityContent) -> Value {
        serde_json::to_value(c.effective_layout()).unwrap()
    }

    /// The layout built from plain fields, growing field by field: title only → icon,
    /// countdown, body, progress, actions (typography and spacing included) → stopwatch and
    /// link buttons → a custom `layout` that wins → `filled` storing it once.
    #[test]
    fn default_layout_scenario() {
        let plain = layout(&base());
        assert_eq!(plain["type"], "vstack");
        assert_eq!(
            (&plain["spacing"], &plain["alignment"], &plain["padding"]),
            (&json!(6.0), &json!("leading"), &json!(12.0))
        );
        assert_eq!(plain["children"].as_array().unwrap().len(), 1);
        let title = &plain["children"][0]["children"];
        assert_eq!(title.as_array().unwrap().len(), 1);
        assert_eq!(
            title[0],
            json!({"type": "text", "content": "Order", "fontSize": 15.0, "fontWeight": "semibold", "lineLimit": 1})
        );

        let full = ActivityContent {
            icon: Some("bolt".into()),
            body: Some("Driver en route".into()),
            progress: Some(0.5),
            ends_at: Some(0),
            tint: Some("#ff0000".into()),
            actions: vec![action("stop", "Stop")],
            ..base()
        };
        let v = layout(&full);
        let head = &v["children"][0];
        assert_eq!(head["spacing"], 8.0);
        let types: Vec<&str> = head["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["type"].as_str().unwrap())
            .collect();
        assert_eq!(types, ["image", "text", "spacer", "timer"]);
        let icon = &head["children"][0];
        assert_eq!(
            (&icon["systemName"], &icon["size"], &icon["color"]),
            (&json!("bolt"), &json!(18.0), &json!("#ff0000"))
        );
        assert_eq!(head["children"][1], title[0]);
        let timer = &head["children"][3];
        assert_eq!(
            (&timer["targetDate"], &timer["counting"], &timer["fontSize"]),
            (&json!("1970-01-01T00:00:00Z"), &json!("down"), &json!(15.0))
        );
        let body = &v["children"][1];
        assert_eq!(
            (&body["content"], &body["fontSize"], &body["lineLimit"]),
            (&json!("Driver en route"), &json!(13.0), &json!(2))
        );
        let bar = &v["children"][2];
        assert_eq!(
            (&bar["type"], &bar["value"], &bar["tint"]),
            (&json!("progress"), &json!(0.5), &json!("#ff0000"))
        );
        let buttons = &v["children"][3];
        assert_eq!(buttons["spacing"], 8.0);
        let b = &buttons["children"][0];
        assert_eq!(
            (&b["type"], &b["action"], &b["label"]),
            (&json!("button"), &json!("stop"), &json!("Stop"))
        );

        // Without `endsAt` a `startedAt` counts up; a link button has a url, no action.
        let v = layout(&ActivityContent {
            started_at: Some(0),
            actions: vec![ActivityAction {
                url: Some("app://x".into()),
                system_name: Some("arrow.up".into()),
                ..action("open", "Open")
            }],
            ..base()
        });
        assert_eq!(v["children"][0]["children"][2]["counting"], "up");
        let b = &v["children"][1]["children"][0];
        assert_eq!(
            (b["url"].as_str(), b.get("action"), b["systemName"].as_str()),
            (Some("app://x"), None, Some("arrow.up"))
        );

        // A custom layout is used as is.
        let custom = ActivityContent {
            layout: Some(WidgetElement::Spacer(SpacerElement::default())),
            ..full.clone()
        };
        assert_eq!(layout(&custom)["type"], "spacer");

        // `filled` stores the built layout; filling again changes nothing.
        let built = layout(&full);
        let f = full.filled();
        assert_eq!(
            serde_json::to_value(f.layout.as_ref().unwrap()).unwrap(),
            built
        );
        let again = f.clone().filled();
        assert_eq!(
            serde_json::to_value(&again).unwrap(),
            serde_json::to_value(&f).unwrap()
        );

        // The timer takes ISO-8601 UTC.
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(951_782_400_000), "2000-02-29T00:00:00Z");
        assert_eq!(iso8601_utc(1_763_051_445_999), "2025-11-13T16:30:45Z");
        assert_eq!(iso8601_utc(4_107_542_400_000), "2100-03-01T00:00:00Z");
    }

    /// A templated request: register, resolve with data, own fields override (an empty own
    /// title falls back, any other own value wins, even an empty body), missing template.
    #[test]
    fn template_scenario() {
        let tpl = json!({
            "title": "Driver {{data.name}}",
            "body": "template body",
            "lockScreen": {"type": "text", "content": "{{data.eta}} min"},
            "minimal": {"type": "text", "content": "!", "when": {"path": "data.late"}}
        })
        .to_string();
        let lookup = |k: &str| (k == "activity-template:ride").then(|| tpl.clone());
        let c = ActivityContent {
            id: "r1".into(),
            template: Some("ride".into()),
            data: Some(json!({"name": "Ann", "eta": 4, "late": false})),
            body: Some("own body".into()),
            ..Default::default()
        };
        c.validate().unwrap();
        let r = resolve_activity(c.clone(), lookup).unwrap();
        assert_eq!(r.title, "Driver Ann");
        assert_eq!(r.body.as_deref(), Some("own body"));
        assert!(r.minimal.is_none() && r.template.is_none() && r.data.is_none());
        let lock = serde_json::to_value(r.lock_screen.unwrap()).unwrap();
        assert_eq!(lock["content"], "4 min");

        let own = resolve_activity(
            ActivityContent {
                title: "Own".into(),
                body: Some(String::new()),
                ..c.clone()
            },
            lookup,
        )
        .unwrap();
        assert_eq!((own.title.as_str(), own.body.as_deref()), ("Own", Some("")));

        assert!(resolve_activity(
            ActivityContent {
                template: Some("nope".into()),
                ..c
            },
            lookup
        )
        .is_err());
        let plain = base();
        assert_eq!(
            resolve_activity(plain.clone(), lookup).unwrap().title,
            plain.title
        );
    }
}

/// One part of an Android Live Updates progress track ([`ActivityContent::progress_segments`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProgressSegment {
    /// Relative length (any positive unit; lengths are normalized).
    pub length: f64,
    /// Hex color; `tint` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// A marker on an Android Live Updates progress track ([`ActivityContent::progress_points`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProgressPoint {
    /// Position `0..=1` along the track.
    pub at: f64,
    /// Hex color; `tint` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}
