//! Render-time resolution of a widget config: scheduled entries, `{{…}}` placeholders.
//!
//! Every renderer (WidgetKit, Glance, desktop webview, Adaptive Cards) runs the same steps
//! on the stored JSON right before drawing, with what only the device knows (time, locale,
//! placement, fetched data). This module is the reference; the Swift, Kotlin and TypeScript
//! ports are checked against the same fixtures in `tests/fixtures/resolve/`.
//!
//! 1. **Entries** — the last entry with `at <= now` replaces the base `small` / `medium` /
//!    `large` it carries.
//! 2. **Placeholders** — in every string of the layouts:
//!    `{{t.key}}` (localized `strings`), `{{data.a.b}}` / `{{data.list.0}}` (data),
//!    `{{locale}}`, `{{instance}}`. A missing `t.key` renders the key, missing data renders
//!    an empty string. `{{{{` is not an escape; placeholders do not nest.
//!    `bind: {"value": "data.progress"}` on any element sets that field to the data value
//!    with its JSON type (for numbers, booleans, arrays); a missing value keeps the literal.
//! 3. **Contract v2** (array elements and family roots):
//!    - `when: {path, eq|ne|gt|gte|lt|lte|exists|in}` or `{all|any: [...]}` / `{not: ...}`;
//!      false removes the element. A bare `path` tests truthiness. `when` is stripped.
//!    - `{"type":"repeat","each":"data.list","as":"item","limit":5,"template":{...}}` splices
//!      one copy of `template` per item (at most 50), with `{{item.x}}` and `{{index}}`.
//!    - `{"type":"use","template":"row","args":{...}}` expands `templates.row`; an exact
//!      `"{{expr}}"` arg keeps its JSON type, other keys on `use` override the template root.
//!      Nesting stops at depth 8. Loop variables and `arg` shadow the built-in roots.
//!    - `requires: {resolve: n}` with `n` above [`CONTRACT_VERSION`] renders `fallback`.
//! 3a. **Contract v3**: root `state` beside `data` (`{{state.k}}`, `when`, `bind`,
//!    `repeat.each`), read from [`ResolveContext::state`] (the config's `state` defaults
//!    merged with the stored state). Resolve only reads it; `set` on button / toggle / link
//!    is kept for the renderer, which writes the state on tap.
//! 4. The result keeps only `version` / `small` / `medium` / `large`, so the existing
//!    decoders read it unchanged.

use serde_json::{Map, Value};

/// What a renderer knows at draw time.
#[derive(Debug, Clone, Default)]
pub struct ResolveContext<'a> {
    /// Current time, Unix epoch milliseconds.
    pub now_ms: u64,
    /// Device locale (BCP-47 `pt-BR` or ICU `pt_BR`).
    pub locale: &'a str,
    /// Placement id: Android `appWidgetId`, the Apple Edit Widget choice, a window label.
    pub instance: Option<&'a str>,
    /// Data layers already merged with [`merge_data`].
    pub data: Option<&'a Value>,
    /// Local state already merged with [`merge_data`]: config `state`, then the stored
    /// `state:{widgetId}`, then `state:{widgetId}#{instance}`.
    pub state: Option<&'a Value>,
}

/// Layout families resolved and kept in the output (`accessory`: Apple Lock Screen).
const FAMILIES: [&str; 4] = ["small", "medium", "large", "accessory"];

/// Base layouts with the active entry applied.
pub fn active_layouts(config: &Value, now_ms: u64) -> Map<String, Value> {
    let mut out = Map::new();
    for f in FAMILIES {
        if let Some(v) = config.get(f).filter(|v| !v.is_null()) {
            out.insert(f.into(), v.clone());
        }
    }
    let active = entries(config)
        .filter(|(at, _)| *at <= now_ms)
        .max_by_key(|(at, _)| *at);
    if let Some((_, entry)) = active {
        for f in FAMILIES {
            if let Some(v) = entry.get(f).filter(|v| !v.is_null()) {
                out.insert(f.into(), v.clone());
            }
        }
    }
    out
}

/// The next time the active entry changes (first `at` after `now`), for scheduling a
/// redraw.
pub fn next_change_ms(config: &Value, now_ms: u64) -> Option<u64> {
    entries(config)
        .map(|(at, _)| at)
        .filter(|at| *at > now_ms)
        .min()
}

/// Entry start times after `now` in ascending order, at most `limit` (WidgetKit timeline).
pub fn upcoming_changes(config: &Value, now_ms: u64, limit: usize) -> Vec<u64> {
    let mut v: Vec<u64> = entries(config)
        .map(|(at, _)| at)
        .filter(|at| *at > now_ms)
        .collect();
    v.sort_unstable();
    v.dedup();
    v.truncate(limit);
    v
}

fn entries(config: &Value) -> impl Iterator<Item = (u64, &Value)> {
    config
        .get("entries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|e| Some((e.get("at")?.as_u64()?, e)))
}

/// Shallow merge of data layers, later layers win per top-level key. Non-object layers
/// replace everything before them.
pub fn merge_data<'a>(layers: impl IntoIterator<Item = Option<&'a Value>>) -> Option<Value> {
    let mut out: Option<Value> = None;
    for layer in layers.into_iter().flatten().filter(|l| !l.is_null()) {
        match (out.as_mut(), layer) {
            (Some(Value::Object(acc)), Value::Object(m)) => {
                for (k, v) in m {
                    acc.insert(k.clone(), v.clone());
                }
            }
            _ => out = Some(layer.clone()),
        }
    }
    out
}

/// Lower case, `_` → `-`, Unicode / private-use extensions (`-u-…`, `-x-…`) dropped.
fn normalize_locale(tag: &str) -> String {
    let t = tag.trim().replace('_', "-").to_ascii_lowercase();
    let cut = ["-u-", "-x-"]
        .iter()
        .filter_map(|m| t.find(m))
        .min()
        .unwrap_or(t.len());
    t[..cut].to_string()
}

/// `zh-hant-tw` → `zh-tw`: the tag without a 4-letter script subtag.
fn language_region(tag: &str) -> String {
    let parts: Vec<&str> = tag.split('-').collect();
    parts
        .iter()
        .enumerate()
        .filter(|(i, p)| !(*i == 1 && p.len() == 4 && p.chars().all(|c| c.is_ascii_alphabetic())))
        .map(|(_, p)| *p)
        .collect::<Vec<_>>()
        .join("-")
}

fn language(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// Name of the `strings` table for `locale`: exact tag, then same language, then
/// `defaultLocale`, then `"en"`, then the lexicographically first table. Matching ignores
/// case and `_` vs `-`.
pub fn pick_locale_table(config: &Value, locale: &str) -> Option<String> {
    let tables = config.get("strings")?.as_object()?;
    if tables.is_empty() {
        return None;
    }
    let want = normalize_locale(locale);
    let mut keys: Vec<&String> = tables.keys().collect();
    keys.sort();
    let find = |pred: &dyn Fn(&str) -> bool| {
        keys.iter()
            .find(|k| pred(&normalize_locale(k)))
            .map(|k| (*k).clone())
    };
    if !want.is_empty() {
        if let Some(k) = find(&|k| k == want) {
            return Some(k);
        }
        let lr = language_region(&want);
        if let Some(k) = find(&|k| language_region(k) == lr) {
            return Some(k);
        }
        let lang = language(&want);
        if let Some(k) = find(&|k| k == lang) {
            return Some(k);
        }
        if let Some(k) = find(&|k| language(k) == lang) {
            return Some(k);
        }
    }
    if let Some(def) = config.get("defaultLocale").and_then(Value::as_str) {
        let def = normalize_locale(def);
        if let Some(k) = find(&|k| k == def) {
            return Some(k);
        }
    }
    find(&|k| k == "en").or_else(|| keys.first().map(|k| (*k).clone()))
}

fn lookup_path<'v>(root: &'v Value, path: &str) -> Option<&'v Value> {
    if path.is_empty() {
        return Some(root);
    }
    path.split('.').try_fold(root, |cur, seg| match cur {
        Value::Object(m) => m.get(seg),
        Value::Array(a) if !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_digit()) => {
            seg.parse::<usize>().ok().and_then(|i| a.get(i))
        }
        _ => None,
    })
}

fn value_text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => match n.as_f64() {
            // `72.0` from JavaScript arrives as a float; draw it like JS does.
            // Whole floats (`72.0` from JavaScript) print as integers, like JS, up to 1e21.
            Some(f) if n.is_f64() && f.fract() == 0.0 && f.abs() < 1e21 => {
                format!("{:.0}", f + 0.0)
            }
            // Like JS: exponent outside 1e-6 ≤ |x| < 1e21 (`4e-323`, `1.5e+21`).
            Some(f) if n.is_f64() && f != 0.0 && (f.abs() < 1e-6 || f.abs() >= 1e21) => {
                let e = format!("{f:e}");
                match e.split_once('e') {
                    Some((m, x)) if !x.starts_with('-') => format!("{m}e+{x}"),
                    _ => e,
                }
            }
            // Shortest round-trip decimal, no exponent (`0.0001`, `12345678.5`).
            Some(f) if n.is_f64() => format!("{f}"),
            _ => n.to_string(),
        },
        other => other.to_string(),
    }
}

/// Renderer contract version: a config with `requires.resolve` above it shows `fallback`.
pub const CONTRACT_VERSION: u64 = 3;
/// Hard cap of items one `repeat` produces.
const MAX_REPEAT: usize = 50;
/// Nested `use` depth (also stops template cycles).
const MAX_TEMPLATE_DEPTH: usize = 8;

#[derive(Clone)]
struct Scope<'a> {
    strings: Option<&'a Map<String, Value>>,
    fallback_strings: Option<&'a Map<String, Value>>,
    ctx: &'a ResolveContext<'a>,
    templates: Option<&'a Map<String, Value>>,
    /// Loop / template variables, innermost last (`task`, `arg`, …).
    locals: Vec<(String, Value)>,
    /// Index of the innermost `repeat`.
    index: Option<usize>,
    depth: usize,
}

/// Path under a typed root: `data` → `""`, `data.a.b` → `"a.b"`, `datax` → `None`.
fn data_root_path<'e>(expr: &'e str, root: &str) -> Option<&'e str> {
    match expr.strip_prefix(root)? {
        "" => Some(""),
        rest => rest.strip_prefix('.'),
    }
}

/// `data…` or `state…`: roots whose values keep their JSON type.
fn is_data_root(expr: &str) -> bool {
    ["data", "state"]
        .iter()
        .any(|r| data_root_path(expr, r).is_some())
}

fn split_head(expr: &str) -> (&str, Option<&str>) {
    match expr.split_once('.') {
        Some((h, rest)) => (h, Some(rest)),
        None => (expr, None),
    }
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::Number(n)) => n.as_f64() != Some(0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(_) => true,
    }
}

/// JSON equality where numbers compare by value (`1 == 1.0`).
fn json_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| json_eq(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| json_eq(v, w)))
        }
        _ => a == b,
    }
}

impl<'a> Scope<'a> {
    fn child(&self, name: &str, value: Value, index: Option<usize>) -> Scope<'a> {
        let mut s = self.clone();
        s.locals.push((name.to_string(), value));
        if index.is_some() {
            s.index = index;
        }
        s
    }

    fn local(&self, name: &str) -> Option<&Value> {
        self.locals
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    fn string(&self, key: &str) -> Option<&Value> {
        self.strings
            .and_then(|m| m.get(key))
            .or_else(|| self.fallback_strings.and_then(|m| m.get(key)))
    }

    /// Typed value of an expression; locals shadow the built-in roots.
    fn lookup_expr(&self, expr: &str) -> Option<Value> {
        let expr = expr.trim();
        let (head, rest) = split_head(expr);
        if let Some(l) = self.local(head) {
            return lookup_path(l, rest.unwrap_or_default()).cloned();
        }
        match expr {
            "index" => return self.index.map(Value::from),
            "locale" => return Some(Value::String(self.ctx.locale.to_string())),
            "instance" => {
                return Some(Value::String(
                    self.ctx.instance.unwrap_or_default().to_string(),
                ))
            }
            _ => {}
        }
        if let Some(key) = expr.strip_prefix("t.") {
            return self.string(key).cloned();
        }
        for (root, value) in [("data", self.ctx.data), ("state", self.ctx.state)] {
            if let Some(path) = data_root_path(expr, root) {
                return value.and_then(|d| lookup_path(d, path)).cloned();
            }
        }
        None
    }

    fn known_root(&self, expr: &str) -> bool {
        let (head, _) = split_head(expr);
        self.local(head).is_some()
            || matches!(expr, "locale" | "instance")
            || is_data_root(expr)
            || (expr == "index" && self.index.is_some())
    }

    fn placeholder(&self, expr: &str) -> String {
        let expr = expr.trim();
        let (head, _) = split_head(expr);
        if self.local(head).is_none() {
            if let Some(key) = expr.strip_prefix("t.") {
                return self
                    .string(key)
                    .map(value_text)
                    .unwrap_or_else(|| key.to_string());
            }
        }
        if !self.known_root(expr) {
            // Unknown root: keep the text so a typo stays visible.
            return format!("{{{{{expr}}}}}");
        }
        self.lookup_expr(expr)
            .map(|v| value_text(&v))
            .unwrap_or_default()
    }

    fn interpolate(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        while let Some(start) = rest.find("{{") {
            let Some(len) = rest[start + 2..].find("}}") else {
                break;
            };
            out.push_str(&rest[..start]);
            out.push_str(&self.placeholder(&rest[start + 2..start + 2 + len]));
            rest = &rest[start + 2 + len + 2..];
        }
        out.push_str(rest);
        out
    }

    /// Structured condition (see the module docs). Bad shapes are false.
    fn eval_when(&self, cond: &Value) -> bool {
        let Some(c) = cond.as_object() else {
            return false;
        };
        if let Some(all) = c.get("all").and_then(Value::as_array) {
            return all.iter().all(|x| self.eval_when(x));
        }
        if let Some(any) = c.get("any").and_then(Value::as_array) {
            return any.iter().any(|x| self.eval_when(x));
        }
        if let Some(not) = c.get("not") {
            return !self.eval_when(not);
        }
        let Some(path) = c.get("path").and_then(Value::as_str) else {
            return false;
        };
        let v = self.lookup_expr(path);
        let has = v.as_ref().is_some_and(|v| !v.is_null());
        let ops = ["eq", "ne", "gt", "gte", "lt", "lte", "exists", "in"];
        if !ops.iter().any(|op| c.contains_key(*op)) {
            return truthy(v.as_ref());
        }
        ops.iter()
            .filter_map(|op| c.get(*op).map(|x| (*op, x)))
            .all(|(op, x)| match op {
                "exists" => has == (x == &Value::Bool(true)),
                "eq" => has && json_eq(v.as_ref().unwrap(), x),
                "ne" => has && !json_eq(v.as_ref().unwrap(), x),
                "in" => {
                    has && x
                        .as_array()
                        .is_some_and(|a| a.iter().any(|y| json_eq(v.as_ref().unwrap(), y)))
                }
                _ => match (v.as_ref().and_then(Value::as_f64), x.as_f64()) {
                    (Some(a), Some(b)) => match op {
                        "gt" => a > b,
                        "gte" => a >= b,
                        "lt" => a < b,
                        _ => a <= b,
                    },
                    _ => false,
                },
            })
    }

    /// One array element → zero or more (`repeat` splices, `when` drops, `use` expands).
    fn expand(&self, el: &Value) -> Vec<Value> {
        let Some(m) = el.as_object() else {
            return vec![self.walk(el)];
        };
        if let Some(cond) = m.get("when") {
            if !self.eval_when(cond) {
                return Vec::new();
            }
        }
        match m.get("type").and_then(Value::as_str) {
            Some("repeat") => {
                let items = m
                    .get("each")
                    .and_then(Value::as_str)
                    .and_then(|e| self.lookup_expr(e));
                let (Some(Value::Array(items)), Some(template)) = (items, m.get("template")) else {
                    return Vec::new();
                };
                if !template.is_object() {
                    return Vec::new();
                }
                let name = m
                    .get("as")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("item");
                let limit = m
                    .get("limit")
                    .and_then(Value::as_u64)
                    .map_or(MAX_REPEAT, |l| (l as usize).min(MAX_REPEAT));
                items
                    .into_iter()
                    .take(limit)
                    .enumerate()
                    .flat_map(|(i, item)| self.child(name, item, Some(i)).expand(template))
                    .collect()
            }
            Some("use") => {
                let tpl = m
                    .get("template")
                    .and_then(Value::as_str)
                    .and_then(|n| self.templates?.get(n))
                    .and_then(Value::as_object);
                let Some(tpl) = tpl.filter(|_| self.depth < MAX_TEMPLATE_DEPTH) else {
                    return Vec::new();
                };
                let mut args = Map::new();
                if let Some(a) = m.get("args").and_then(Value::as_object) {
                    for (k, v) in a {
                        args.insert(k.clone(), self.arg_value(v));
                    }
                }
                // Keys on the `use` element override the template root (instance wins).
                let mut root = tpl.clone();
                for (k, v) in m {
                    if !matches!(k.as_str(), "type" | "template" | "args" | "when") {
                        root.insert(k.clone(), v.clone());
                    }
                }
                let mut inner = self.child("arg", Value::Object(args), None);
                inner.depth += 1;
                inner.expand(&Value::Object(root))
            }
            _ => vec![self.walk(el)],
        }
    }

    /// An argument that is exactly one `{{expr}}` keeps the expression's JSON type.
    fn arg_value(&self, v: &Value) -> Value {
        let Some(s) = v.as_str() else {
            return v.clone();
        };
        if let Some(inner) = s.strip_prefix("{{").and_then(|r| r.strip_suffix("}}")) {
            if !inner.contains("{{") && !inner.contains("}}") {
                return self
                    .lookup_expr(inner)
                    .unwrap_or_else(|| Value::String(String::new()));
            }
        }
        Value::String(self.interpolate(s))
    }

    fn walk(&self, v: &Value) -> Value {
        match v {
            Value::String(s) => Value::String(self.interpolate(s)),
            Value::Array(a) => Value::Array(a.iter().flat_map(|x| self.expand(x)).collect()),
            Value::Object(m) => {
                let mut out = Map::new();
                for (k, x) in m {
                    if k != "bind" && k != "when" {
                        out.insert(k.clone(), self.walk(x));
                    }
                }
                if let Some(Value::Object(b)) = m.get("bind") {
                    for (field, expr) in b {
                        if let Some(val) = expr.as_str().and_then(|e| self.bound(e)) {
                            out.insert(field.clone(), val);
                        }
                    }
                }
                Value::Object(out)
            }
            other => other.clone(),
        }
    }

    /// Value for `bind: {field: expr}`: data / loop / arg values keep their JSON type
    /// (numbers stay numbers); other expressions render as text. A missing value keeps
    /// the literal.
    fn bound(&self, expr: &str) -> Option<Value> {
        let expr = expr.trim();
        let (head, _) = split_head(expr);
        let typed = self.local(head).is_some()
            || is_data_root(expr)
            || (expr == "index" && self.index.is_some());
        if typed {
            return self.lookup_expr(expr).filter(|v| !v.is_null());
        }
        Some(Value::String(self.placeholder(expr)))
    }

    /// A family root: `use` / `repeat` / `when` apply there too (a dropped root is omitted).
    fn walk_root(&self, v: &Value) -> Option<Value> {
        if v.get("type").and_then(Value::as_str) == Some("repeat") {
            return Some(serde_json::json!({"type": "vstack", "children": self.expand(v)}));
        }
        self.expand(v).into_iter().next()
    }
}

fn strings_table<'a>(config: &'a Value, name: Option<&str>) -> Option<&'a Map<String, Value>> {
    config.get("strings")?.get(name?)?.as_object()
}

fn scope_for<'a>(config: &'a Value, ctx: &'a ResolveContext<'a>) -> Scope<'a> {
    let table = pick_locale_table(config, ctx.locale);
    let def = pick_locale_table(config, "");
    Scope {
        strings: strings_table(config, table.as_deref()),
        fallback_strings: strings_table(config, def.as_deref()),
        ctx,
        templates: config.get("templates").and_then(Value::as_object),
        locals: Vec::new(),
        index: None,
        depth: 0,
    }
}

/// Replace placeholders in one string (used for [`crate::models::DataSource`] fields).
pub fn interpolate(config: &Value, ctx: &ResolveContext<'_>, s: &str) -> String {
    scope_for(config, ctx).interpolate(s)
}

/// The config this renderer can show: `fallback` (or a built-in notice) when
/// `requires.resolve` is above [`CONTRACT_VERSION`].
fn effective_config(config: &Value) -> std::borrow::Cow<'_, Value> {
    let need = config
        .get("requires")
        .and_then(|r| r.get("resolve"))
        .and_then(Value::as_u64);
    if !need.is_some_and(|n| n > CONTRACT_VERSION) {
        return std::borrow::Cow::Borrowed(config);
    }
    let mut out = match config.get("fallback") {
        Some(Value::Object(fb)) => fb.clone(),
        _ => serde_json::json!({"small": {"type": "text", "content": "Update the app to show this widget"}})
            .as_object()
            .cloned()
            .unwrap_or_default(),
    };
    for k in ["version", "strings", "defaultLocale"] {
        if let Some(v) = config.get(k) {
            out.insert(k.into(), v.clone());
        }
    }
    std::borrow::Cow::Owned(Value::Object(out))
}

/// Full resolution: active entry, then templates / repeat / when / placeholders / bind.
/// Output has only `version` and the size families.
pub fn resolve(config: &Value, ctx: &ResolveContext<'_>) -> Value {
    let config = effective_config(config);
    let config = config.as_ref();
    let scope = scope_for(config, ctx);
    let layouts = active_layouts(config, ctx.now_ms);
    let mut out = Map::new();
    out.insert(
        "version".into(),
        config
            .get("version")
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or(Value::from(1)),
    );
    for (family, v) in layouts {
        if let Some(r) = scope.walk_root(&v) {
            out.insert(family, r);
        }
    }
    Value::Object(out)
}

/// Resolve a free-form object (a Live Activity template, see
/// [`crate::models::resolve_activity`]): element roots (objects with a `type`) like a family
/// root, everything else like any value in a layout. `strings` / `templates` /
/// `defaultLocale` in it work as in a config and are not part of the output.
pub fn resolve_fields(fields: &Value, ctx: &ResolveContext<'_>) -> Value {
    let scope = scope_for(fields, ctx);
    let Some(m) = fields.as_object() else {
        return scope.walk(fields);
    };
    let mut out = Map::new();
    for (k, v) in m {
        if matches!(k.as_str(), "strings" | "templates" | "defaultLocale") {
            continue;
        }
        let r = if v.get("type").is_some() {
            scope.walk_root(v)
        } else {
            Some(scope.walk(v))
        };
        if let Some(r) = r {
            out.insert(k.clone(), r);
        }
    }
    Value::Object(out)
}

/// `https://` anywhere; plain `http://` only to `localhost` / `127.0.0.1` (development).
/// The host is compared exactly, so `http://localhost.evil.com` and
/// `http://localhost@evil.com` are refused.
pub fn source_url_allowed(url: &str) -> bool {
    if url.starts_with("https://") {
        return true;
    }
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return false;
    }
    let (host, port) = authority.split_once(':').unwrap_or((authority, ""));
    let host_ok = host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1";
    host_ok && port.chars().all(|c| c.is_ascii_digit())
}

/// Next time a host-rendered surface needs work: the next entry change, or when `source`
/// is due (`last_fetch_ms + refreshMinutes`; now when never fetched or overdue).
pub fn next_wake_ms(config: &Value, now_ms: u64, last_fetch_ms: Option<u64>) -> Option<u64> {
    // Only placeholders of the source fields are filled; they never read the time.
    let ctx = ResolveContext::default();
    let source = source_request(config, &ctx).map(|s| {
        last_fetch_ms
            .map(|t| t.saturating_add(u64::from(s.refresh_minutes) * 60_000))
            .unwrap_or(now_ms)
            .max(now_ms)
    });
    [next_change_ms(config, now_ms), source]
        .into_iter()
        .flatten()
        .min()
}

/// A [`crate::models::DataSource`] with placeholders filled, ready to send.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRequest {
    /// Request URL.
    pub url: String,
    /// `GET` or `POST`.
    pub method: String,
    /// Header name → value.
    pub headers: Vec<(String, String)>,
    /// Body for `POST`.
    pub body: Option<String>,
    /// Minutes between fetches (15..=1440).
    pub refresh_minutes: u32,
}

/// Default minutes between source fetches.
pub const DEFAULT_REFRESH_MINUTES: u32 = 30;
/// Largest response body a renderer accepts.
pub const MAX_SOURCE_BYTES: usize = 256 * 1024;

/// The fetch a renderer should make for this config, or `None` without a valid `source`.
/// Only `https://` is allowed (`http://localhost` too, for development).
pub fn source_request(config: &Value, ctx: &ResolveContext<'_>) -> Option<SourceRequest> {
    let src = config.get("source")?;
    let url = interpolate(config, ctx, src.get("url")?.as_str()?);
    if !source_url_allowed(&url) {
        return None;
    }
    let method = match src.get("method").and_then(Value::as_str) {
        Some(m) if m.eq_ignore_ascii_case("post") => "POST",
        _ => "GET",
    }
    .to_string();
    let headers = src
        .get("headers")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.clone(), interpolate(config, ctx, v.as_str()?))))
                .collect()
        })
        .unwrap_or_default();
    let body = if method == "POST" {
        src.get("body")
            .and_then(Value::as_str)
            .map(|b| interpolate(config, ctx, b))
    } else {
        None
    };
    let refresh_minutes = src
        .get("refreshMinutes")
        .and_then(Value::as_u64)
        .map(|m| m.clamp(15, 1440) as u32)
        .unwrap_or(DEFAULT_REFRESH_MINUTES);
    Some(SourceRequest {
        url,
        method,
        headers,
        body,
        refresh_minutes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    fn ctx<'a>(now: u64, locale: &'a str, data: Option<&'a Value>) -> ResolveContext<'a> {
        ResolveContext {
            now_ms: now,
            locale,
            instance: None,
            data,
            state: None,
        }
    }

    /// `small.content` of `cfg` resolved with `data`.
    fn small_text(cfg: &Value, data: &Value) -> Value {
        resolve(cfg, &ctx(0, "en", Some(data)))["small"]["content"].clone()
    }

    /// Fixture format v2: whether this port runs a case. `platforms` lists the ports that
    /// run it (all when absent); `skip: {port: reason}` skips one with a reason.
    /// `Err` when the port neither runs it nor has a reason.
    pub(crate) fn fixture_runs(case: &Value, port: &str) -> Result<bool, String> {
        let reason = case
            .get("skip")
            .and_then(|s| s.get(port))
            .and_then(Value::as_str)
            .filter(|r| !r.trim().is_empty());
        if reason.is_some() {
            return Ok(false);
        }
        match case.get("platforms").and_then(Value::as_array) {
            None => Ok(true),
            Some(p) if p.iter().any(|x| x == port) => Ok(true),
            Some(_) => Err(format!("not in `platforms` and no `skip.{port}` reason")),
        }
    }

    /// Shared contract: the Swift, Kotlin and TypeScript ports run the same files. First the
    /// rules that pick which fixtures this port runs, then every fixture.
    #[test]
    fn fixtures_match() {
        assert_eq!(fixture_runs(&json!({}), "rust"), Ok(true));
        assert_eq!(
            fixture_runs(&json!({"platforms": ["rust"]}), "rust"),
            Ok(true)
        );
        assert_eq!(
            fixture_runs(
                &json!({"platforms": ["ts"], "skip": {"rust": "n/a"}}),
                "rust"
            ),
            Ok(false)
        );
        assert!(fixture_runs(&json!({"platforms": ["ts"]}), "rust").is_err());
        assert!(
            fixture_runs(&json!({"platforms": ["ts"], "skip": {"rust": " "}}), "rust").is_err()
        );
        assert_eq!(
            fixture_runs(&json!({"skip": {"swift": "x"}}), "rust"),
            Ok(true)
        );

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/resolve");
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .collect();
        files.sort();
        let (mut ran, mut skipped, mut unaccounted) = (Vec::new(), Vec::new(), Vec::new());
        for path in &files {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let case: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap())
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            match fixture_runs(&case, "rust") {
                Ok(true) => {}
                Ok(false) => {
                    skipped.push(name);
                    continue;
                }
                Err(e) => {
                    unaccounted.push(format!("{name}: {e}"));
                    continue;
                }
            }
            let c = &case["ctx"];
            let config = &case["config"];
            let data = merge_data([config.get("data"), c.get("data")]);
            let state = merge_data([config.get("state"), c.get("state")]);
            let rc = ResolveContext {
                now_ms: c["nowMs"].as_u64().unwrap(),
                locale: c["locale"].as_str().unwrap_or(""),
                instance: c.get("instance").and_then(Value::as_str),
                data: data.as_ref(),
                state: state.as_ref(),
            };
            assert_eq!(resolve(config, &rc), case["expected"], "{}", path.display());
            let next = next_change_ms(config, rc.now_ms);
            assert_eq!(
                next.map(Value::from).unwrap_or(Value::Null),
                case.get("nextChangeMs").cloned().unwrap_or(Value::Null),
                "{} nextChangeMs",
                path.display()
            );
            if let Some(want) = case.get("source") {
                let got = source_request(config, &rc).map(|s| {
                    json!({
                        "url": s.url, "method": s.method, "body": s.body,
                        "refreshMinutes": s.refresh_minutes,
                        "headers": s.headers.into_iter()
                            .map(|(k, v)| (k, Value::String(v))).collect::<Map<_, _>>(),
                    })
                });
                assert_eq!(
                    got.unwrap_or(Value::Null),
                    *want,
                    "{} source",
                    path.display()
                );
            }
            ran.push(name);
        }
        // Executed-case manifest (`cargo test -- --nocapture` shows it).
        println!("resolve fixtures ran: {}", ran.join(", "));
        println!("resolve fixtures skipped: {}", skipped.join(", "));
        assert!(
            unaccounted.is_empty(),
            "fixtures neither run nor skipped with a reason: {unaccounted:?}"
        );
        assert_eq!(ran.len() + skipped.len(), files.len());
        assert!(
            ran.len() >= 5,
            "expected resolve fixtures, ran {}",
            ran.len()
        );
    }

    /// A widget's day: base layouts, scheduled entries switching in (in any order, exactly at
    /// `at`), the redraw schedule, and the source fetch joining the wake-up times.
    #[test]
    fn timeline_scenario() {
        let cfg = json!({
            "version": 2,
            "small": {"type": "text", "content": "s"},
            "medium": {"type": "text", "content": "m"},
            "entries": [
                {"at": 300, "small": {"type": "text", "content": "c"}},
                {"at": 100, "small": {"type": "text", "content": "a"}},
                {"at": 200, "small": {"type": "text", "content": "b"},
                            "medium": {"type": "text", "content": "m2"}}
            ]
        });
        let texts = |now: u64| {
            let l = active_layouts(&cfg, now);
            assert!(!l.contains_key("large"));
            (
                l["small"]["content"].clone(),
                l["medium"]["content"].clone(),
            )
        };
        // Before the first entry: base layouts, next change at the first `at`.
        assert_eq!(texts(99), (json!("s"), json!("m")));
        assert_eq!(next_change_ms(&cfg, 99), Some(100));
        assert_eq!(upcoming_changes(&cfg, 0, 2), vec![100, 200]);
        // Exactly at `at` the entry is active and no longer upcoming; an entry without
        // `medium` keeps the base one.
        assert_eq!(texts(100), (json!("a"), json!("m")));
        assert_eq!(upcoming_changes(&cfg, 100, 10), vec![200, 300]);
        // The latest past entry wins, whatever the array order.
        assert_eq!(texts(250), (json!("b"), json!("m2")));
        assert_eq!(upcoming_changes(&cfg, 150, 10), vec![200, 300]);
        // After the last entry nothing is scheduled; it replaces only what it carries.
        assert_eq!(texts(300), (json!("c"), json!("m")));
        assert_eq!(next_change_ms(&cfg, 300), None);
        // The output keeps `version` and the families only.
        assert_eq!(
            resolve(&cfg, &ctx(250, "en", None)),
            json!({"version": 2, "small": {"type": "text", "content": "b"},
                   "medium": {"type": "text", "content": "m2"}})
        );
        assert_eq!(
            resolve(&json!({}), &ctx(0, "en", None)),
            json!({"version": 1})
        );

        // Wake-ups: entries plus a source due `refreshMinutes` after the last fetch.
        let cfg = json!({"entries": [{"at": 5_000}], "source": {"url": "https://a", "refreshMinutes": 15}});
        assert_eq!(
            next_wake_ms(&cfg, 1_000, None),
            Some(1_000),
            "never fetched: now"
        );
        assert_eq!(
            next_wake_ms(&cfg, 1_000, Some(0)),
            Some(5_000),
            "entry first"
        );
        assert_eq!(
            next_wake_ms(&cfg, 6_000, Some(0)),
            Some(900_000),
            "then the source"
        );
        assert_eq!(
            next_wake_ms(&cfg, 2_000_000, Some(0)),
            Some(2_000_000),
            "overdue: now"
        );
        assert_eq!(next_wake_ms(&json!({}), 0, None), None);
        assert_eq!(
            next_wake_ms(&json!({"entries": [{"at": 9}]}), 0, Some(0)),
            Some(9)
        );
    }

    /// Strings: the table chosen for a locale (exact, language, default, `en`, first), then
    /// a missing key falling back to the default table and finally to the key itself.
    #[test]
    fn localization_scenario() {
        let cfg = json!({
            "defaultLocale": "de",
            "strings": {"en": {}, "de": {}, "pt-BR": {}, "ru": {}}
        });
        assert_eq!(pick_locale_table(&cfg, "pt_BR").as_deref(), Some("pt-BR"));
        assert_eq!(pick_locale_table(&cfg, "pt-PT").as_deref(), Some("pt-BR"));
        assert_eq!(pick_locale_table(&cfg, "ru-RU").as_deref(), Some("ru"));
        assert_eq!(pick_locale_table(&cfg, "fr").as_deref(), Some("de"));
        assert_eq!(pick_locale_table(&cfg, "").as_deref(), Some("de"));
        let no_default = json!({"strings": {"fr": {}, "en": {}}});
        assert_eq!(pick_locale_table(&no_default, "ja").as_deref(), Some("en"));
        let first = json!({"strings": {"fr": {}, "de": {}}});
        assert_eq!(pick_locale_table(&first, "ja").as_deref(), Some("de"));
        assert_eq!(pick_locale_table(&json!({}), "en"), None);
        assert_eq!(pick_locale_table(&json!({"strings": {}}), "en"), None);

        let cfg = json!({
            "defaultLocale": "en",
            "strings": {"en": {"a": "A", "b": "B"}, "ru": {"a": "А"}},
            "small": {"type": "text", "content": "{{t.a}}{{t.b}}{{t.c}}"}
        });
        assert_eq!(
            resolve(&cfg, &ctx(0, "ru", None))["small"]["content"],
            "АBc"
        );
        assert_eq!(
            resolve(&cfg, &ctx(0, "en", None))["small"]["content"],
            "ABc"
        );
    }

    /// How values print and bind: data and state roots, JS-like numbers, exact big
    /// integers, unknown roots and `index` outside a repeat left visible, `bind` keeping
    /// the JSON type.
    #[test]
    fn placeholders_and_bind_scenario() {
        let data = json!({"t": 72.0, "f": 1.5, "b": true, "n": null, "list": ["x", "y"],
            "o": {"k": 1}, "u": u64::MAX, "i": 9_007_199_254_740_993_i64});
        let text = |content: &str| json!({"small": {"type": "text", "content": content}});
        assert_eq!(
            small_text(&text("{{data.t}}|{{data.f}}|{{data.b}}|{{data.n}}|{{data.list.1}}|{{data.o}}|{{data.missing}}|{{data.list.9}}|{{ data.t }}"), &data),
            r#"72|1.5|true||y|{"k":1}|||72"#
        );
        assert_eq!(
            small_text(&text("{{data.u}} {{data.i}}"), &data),
            "18446744073709551615 9007199254740993"
        );
        assert_eq!(
            small_text(&text("{{oops}} {{index}} {{data.x"), &data),
            "{{oops}} {{index}} {{data.x"
        );

        // State reads like data; `set` stays for the renderer.
        let state = json!({"on": true, "n": 2, "list": ["a"]});
        let c = ResolveContext {
            state: Some(&state),
            ..ctx(0, "en", Some(&data))
        };
        let cfg = json!({"small": {"type": "vstack", "children": [
            {"type": "text", "content": "{{state.n}}|{{state.x}}|{{statex}}"},
            {"type": "toggle", "isOn": false, "bind": {"isOn": "state.on"}, "set": {"on": "$toggle"}},
            {"type": "text", "content": "hidden", "when": {"path": "state.on", "eq": false}},
            {"type": "text", "content": "lit", "bind": {"content": "index"}},
            {"type": "progress", "value": 0.1, "bind": {"value": "data.missing"}},
            {"type": "repeat", "each": "state.list", "template":
                {"type": "progress", "value": 0.5, "label": "{{oops}}", "bind": {"value": "index"}}}
        ]}});
        let out = resolve(&cfg, &c);
        let ch = out["small"]["children"].as_array().unwrap();
        assert_eq!(ch.len(), 5, "the `when` child is dropped");
        assert_eq!(ch[0]["content"], "2||{{statex}}");
        assert_eq!(ch[1]["isOn"], true);
        assert_eq!(ch[1]["set"]["on"], "$toggle");
        // Outside a repeat `index` is not a root: bound as visible text.
        assert_eq!(ch[2]["content"], "{{index}}");
        assert_eq!(ch[3]["value"], 0.1, "missing bound value keeps the literal");
        // Inside a repeat `index` binds as a number; unknown roots still stay visible.
        assert_eq!(ch[4]["value"], json!(0));
        assert_eq!(ch[4]["label"], "{{oops}}");
    }

    /// `when` on array children: every operator on both sides of its boundary, missing
    /// values, truthiness, and deep equality where numbers compare by value.
    #[test]
    fn conditions_scenario() {
        let data = json!({"n": 3, "s": "ok", "e": "", "arr": [1, 2], "obj": {"a": 1}});
        let t = |name: &str, when: Value| json!({"type": "text", "content": name, "when": when});
        let cfg = json!({"small": {"type": "vstack", "children": [
            t("gte-above", json!({"path": "data.n", "gte": 2})),
            t("lt-equal-no", json!({"path": "data.n", "lt": 3})),
            t("lt-above", json!({"path": "data.n", "lt": 4})),
            t("lte-equal", json!({"path": "data.n", "lte": 3})),
            t("lte-below-no", json!({"path": "data.n", "lte": 2})),
            t("ne-equal-no", json!({"path": "data.s", "ne": "ok"})),
            t("ne-missing-no", json!({"path": "data.x", "ne": "ok"})),
            t("in-no", json!({"path": "data.s", "in": ["x"]})),
            t("in-missing-no", json!({"path": "data.x", "in": ["ok"]})),
            t("str-truthy", json!({"path": "data.s"})),
            t("str-empty-no", json!({"path": "data.e"})),
            t("arr-num-eq", json!({"path": "data.arr", "eq": [1.0, 2.0]})),
            t("arr-prefix-no", json!({"path": "data.arr", "eq": [1]})),
            t("arr-other-no", json!({"path": "data.arr", "eq": [1, 3]})),
            t("obj-num-eq", json!({"path": "data.obj", "eq": {"a": 1.0}})),
            t("obj-superset-no", json!({"path": "data.obj", "eq": {"a": 1, "b": 2}})),
            t("obj-other-no", json!({"path": "data.obj", "eq": {"b": 1}})),
        ]}});
        let got: Vec<Value> = resolve(&cfg, &ctx(0, "en", Some(&data)))["small"]["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["content"].clone())
            .collect();
        assert_eq!(
            got,
            [
                "gte-above",
                "lt-above",
                "lte-equal",
                "str-truthy",
                "arr-num-eq",
                "obj-num-eq"
            ]
        );
    }

    /// Reuse: `use` with typed and interpolated args, self-recursion cut at the depth cap,
    /// and `resolve_fields` on a Live Activity template.
    #[test]
    fn templates_scenario() {
        let data = json!({"a": 5, "eta": 5, "late": false});
        let cfg = json!({
            "templates": {
                "row": {"type": "text", "content": "{{arg.v}}"},
                "r": {"type": "vstack", "children": [{"type": "use", "template": "r"}]}
            },
            "small": {"type": "use", "template": "row", "args": {"v": "{{data.a}}x}}"}},
            "medium": {"type": "use", "template": "r"}
        });
        let out = resolve(&cfg, &ctx(0, "en", Some(&data)));
        // Not one exact `{{expr}}`: the arg is interpolated as text.
        assert_eq!(out["small"]["content"], "5x}}");
        let (mut depth, mut cur) = (0, &out["medium"]);
        while cur["type"] == "vstack" {
            depth += 1;
            cur = &cur["children"][0];
        }
        assert_eq!(depth, MAX_TEMPLATE_DEPTH);

        let tpl = json!({
            "title": "ETA {{data.eta}}",
            "lockScreen": {"type": "text", "content": "{{data.eta}} min"},
            "minimal": {"type": "text", "content": "x", "when": {"path": "data.late"}},
            "strings": {"en": {"a": "b"}}
        });
        assert_eq!(
            resolve_fields(&tpl, &ctx(0, "en", Some(&data))),
            json!({"title": "ETA 5", "lockScreen": {"type": "text", "content": "5 min"}})
        );
    }

    /// Fetch inputs: data layers merged, then the `source` request built from them — only
    /// `https://` or loopback `http://`, `POST` only when asked, refresh clamped.
    #[test]
    fn data_source_scenario() {
        let a = json!({"x": 1, "y": 1});
        let b = json!({"y": 2});
        assert_eq!(
            merge_data([Some(&a), None, Some(&b)]),
            Some(json!({"x": 1, "y": 2}))
        );
        let scalar = json!(5);
        assert_eq!(merge_data([Some(&a), Some(&scalar)]), Some(json!(5)));
        assert_eq!(merge_data([Some(&scalar), Some(&b)]), Some(json!({"y": 2})));
        assert_eq!(merge_data([None, None]), None);

        let c = ctx(0, "ru", None);
        let get = |src: Value| source_request(&json!({ "source": src }), &c);
        assert!(get(json!({"url": "http://example.com"})).is_none());
        assert!(get(json!({"url": "ftp://x"})).is_none());
        assert!(get(json!({})).is_none());
        for bad in [
            "http://localhost.evil.com/a",
            "http://localhost@evil.com/a",
            "http://127.0.0.1.nip.io/a",
            "http://localhost:80x/a",
            "http://evil.com/?localhost",
        ] {
            assert!(get(json!({ "url": bad })).is_none(), "{bad}");
        }
        for ok in [
            "http://LOCALHOST/a",
            "http://localhost",
            "http://127.0.0.1:8080?x=1",
            "http://127.0.0.1/a",
        ] {
            assert!(get(json!({ "url": ok })).is_some(), "{ok}");
        }
        let r = get(json!({"url": "http://localhost:1/a"})).unwrap();
        assert_eq!(r.method, "GET");
        assert_eq!(r.refresh_minutes, DEFAULT_REFRESH_MINUTES);
        let r = get(json!({"url": "https://a/{{locale}}", "method": "post", "body": "{{locale}}", "refreshMinutes": 1})).unwrap();
        assert_eq!(
            (
                r.url.as_str(),
                r.method.as_str(),
                r.body.as_deref(),
                r.refresh_minutes
            ),
            ("https://a/ru", "POST", Some("ru"), 15)
        );
        let r = get(json!({"url": "https://a", "body": "x", "refreshMinutes": 99999})).unwrap();
        assert_eq!((r.body, r.refresh_minutes), (None, 1440));
        for m in ["GET", "put"] {
            let r = get(json!({"url": "https://a", "method": m, "body": "x"})).unwrap();
            assert_eq!((r.method.as_str(), r.body), ("GET", None), "{m}");
        }
        // Renderers refuse responses above 256 KiB.
        assert_eq!(MAX_SOURCE_BYTES, 262_144);
    }
}
