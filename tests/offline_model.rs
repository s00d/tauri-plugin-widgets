//! Model test of the plugin through its external entry: random sequences of the IPC
//! commands a JavaScript app sends (valid and invalid arguments), run against the real
//! plugin on Tauri's MockRuntime and against a small reference model (maps of items,
//! configs, data, instances, state layers, templates, running activities, receipts).
//!
//! Each step compares the KIND of the response (the shape of the `Ok` value, or that it is
//! an error; never error text) and the events it emits; after every step the observable
//! state (`get_widget_state`, `get_widget_config`, stored data / instances,
//! `list_activities`, the activity island, diagnostics, active widgets) must equal the
//! model's. A failure prints the shrunk action sequence: that list is the bug scenario.
//! `PROPTEST_CASES` overrides the case count.
#![cfg(not(any(target_os = "android", target_os = "ios")))]

mod common;
use common::*;

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use proptest::test_runner::{Config, TestError, TestRunner};
use serde_json::Map;

// ---- actions ------------------------------------------------------------------------

/// A widget id: one of two valid ids of the case, or one with path characters.
#[derive(Clone, Copy, Debug)]
enum Id {
    W(u8),
    Bad,
}

/// A placement: none, one of two valid ids, blank (treated as none) or invalid.
#[derive(Clone, Copy, Debug)]
enum Inst {
    None,
    P(u8),
    Blank,
    Bad,
}

/// `group` argument: omitted, blank (default), another valid group, invalid.
#[derive(Clone, Copy, Debug)]
enum Grp {
    Default,
    Blank,
    Other,
    Bad,
}

#[derive(Clone, Debug)]
enum Cfg {
    /// `small: text(content)` plus optional `state` defaults.
    Valid {
        content: u8,
        state: Option<BTreeMap<String, Value>>,
    },
    /// Unknown element type.
    UnknownElement,
    /// Not an object.
    NotObject,
}

#[derive(Clone, Debug)]
enum Data {
    Null,
    Obj(i64),
    /// Over the 256 KiB limit.
    Oversize,
}

#[derive(Clone, Debug)]
enum Instances {
    Ok(u8),
    Empty,
    TooMany,
    BlankId,
    BadId,
    LongTitle,
}

#[derive(Clone, Debug)]
enum Op {
    Toggle,
    Inc,
    Dec,
    Null,
    Set(Value),
}

#[derive(Clone, Debug)]
enum Patch {
    Null,
    Ops(Vec<(String, Op)>),
    NotObject,
}

#[derive(Clone, Debug)]
enum Tpl {
    Null,
    /// `{title: "Tpl {{data.name}}", body}` or, without title, `{icon}`.
    Obj {
        title: bool,
    },
    NotObject,
}

#[derive(Clone, Copy, Debug)]
enum Choice {
    Valid,
    Bad,
}

#[derive(Clone, Debug)]
struct Content {
    /// Activity id; `None` = invalid (`bad id`).
    act: Option<u8>,
    /// `None` = blank title.
    title: Option<u8>,
    /// `Some(None)` = invalid template name.
    template: Option<Option<u8>>,
    progress: Option<f64>,
    actions: usize,
    visibility: Option<Choice>,
    category: Option<Choice>,
    big_body: bool,
    timestamp: Option<u64>,
    name: u8,
}

#[derive(Clone, Copy, Debug)]
enum Query {
    WidgetPushToken(Grp),
    PushToStartToken,
    CanPromote,
    OpenPromotionSettings,
    RequestWidget,
    ReloadAll,
    ReloadKind,
    RegisterWidgets,
}

#[derive(Clone, Debug)]
enum Action {
    SetItems {
        key: u8,
        value: u8,
        group: Grp,
    },
    GetItems {
        key: u8,
        group: Grp,
    },
    SetConfig {
        id: Id,
        inst: Inst,
        config: Cfg,
        skip_reload: bool,
    },
    GetConfig {
        id: Id,
        inst: Inst,
    },
    SetData {
        id: Id,
        inst: Inst,
        data: Data,
    },
    SetInstances {
        id: Id,
        list: Instances,
    },
    SetState {
        id: Id,
        inst: Inst,
        patch: Patch,
    },
    GetState {
        id: Id,
        inst: Inst,
    },
    RegisterTemplate {
        name: Option<u8>,
        layout: Tpl,
    },
    Start(Content),
    Update(Content),
    End {
        act: Option<u8>,
        final_content: Option<Content>,
        policy: Option<Choice>,
        dismiss_after_ms: Option<u64>,
    },
    ListActivities,
    Island {
        expanded: Option<bool>,
        focus: Option<u8>,
    },
    Query(Query),
    Poll(Grp),
    /// `widget_action`: a tap the built-in renderer forwards (emits `widget-action`).
    Tap {
        action: u8,
        payload: Option<u8>,
        widget: Option<Id>,
        group: Grp,
    },
    Receipt {
        widget: u8,
        inst: u8,
        bad_group: bool,
    },
    /// A command with required arguments left out.
    MissingArgs(&'static str),
}

/// One step: the action, optionally with an unknown extra argument (must be ignored).
#[derive(Clone, Debug)]
struct Step {
    action: Action,
    extra: bool,
}

const STATE_KEYS: [&str; 3] = ["a", "b", "c"];
const NEEDS_ARGS: [&str; 10] = [
    "set_items",
    "get_items",
    "set_widget_config",
    "get_widget_config",
    "set_widget_instances",
    "start_activity",
    "update_activity",
    "end_activity",
    "register_activity_template",
    "reload_timelines",
];

fn id() -> impl Strategy<Value = Id> {
    prop_oneof![6 => (0u8..2).prop_map(Id::W), 1 => Just(Id::Bad)]
}

fn inst() -> impl Strategy<Value = Inst> {
    prop_oneof![
        4 => Just(Inst::None),
        4 => (0u8..2).prop_map(Inst::P),
        1 => Just(Inst::Blank),
        1 => Just(Inst::Bad),
    ]
}

fn grp() -> impl Strategy<Value = Grp> {
    prop_oneof![
        6 => Just(Grp::Default),
        1 => Just(Grp::Blank),
        1 => Just(Grp::Other),
        1 => Just(Grp::Bad),
    ]
}

fn value() -> impl Strategy<Value = Value> {
    prop_oneof![
        (-3i64..3).prop_map(Value::from),
        any::<bool>().prop_map(Value::from),
        "[a-z]{0,2}".prop_map(Value::from),
        Just(json!(0.5)),
        Just(json!(i64::MAX)),
        Just(json!(u64::MAX)),
        Just(json!([1])),
    ]
}

fn state_key() -> impl Strategy<Value = String> {
    prop::sample::select(&STATE_KEYS[..]).prop_map(String::from)
}

fn cfg() -> impl Strategy<Value = Cfg> {
    prop_oneof![
        8 => (0u8..3, prop::option::of(prop::collection::btree_map(state_key(), value(), 0..3)))
            .prop_map(|(content, state)| Cfg::Valid { content, state }),
        1 => Just(Cfg::UnknownElement),
        1 => Just(Cfg::NotObject),
    ]
}

fn choice() -> impl Strategy<Value = Option<Choice>> {
    prop_oneof![6 => Just(None), 2 => Just(Some(Choice::Valid)), 1 => Just(Some(Choice::Bad))]
}

fn content() -> impl Strategy<Value = Content> {
    (
        prop_oneof![8 => (0u8..3).prop_map(Some), 1 => Just(None)],
        prop_oneof![6 => (0u8..3).prop_map(Some), 1 => Just(None)],
        prop_oneof![6 => Just(None), 3 => (0u8..2).prop_map(|t| Some(Some(t))), 1 => Just(Some(None))],
        prop_oneof![4 => Just(None), 3 => (0u8..=4).prop_map(|p| Some(p as f64 / 4.0)), 1 => Just(Some(1.25)), 1 => Just(Some(-0.25))],
        prop_oneof![8 => 0usize..=3, 1 => Just(4usize)],
        choice(),
        choice(),
        prop::bool::weighted(0.1),
        prop::option::of(0u64..4),
        0u8..2,
    )
        .prop_map(
            |(act, title, template, progress, actions, visibility, category, big_body, timestamp, name)| Content {
                act,
                title,
                template,
                progress,
                actions,
                visibility,
                category,
                big_body,
                timestamp,
                name,
            },
        )
}

fn action() -> impl Strategy<Value = Action> {
    let op = prop_oneof![
        Just(Op::Toggle),
        Just(Op::Inc),
        Just(Op::Dec),
        Just(Op::Null),
        value().prop_map(Op::Set),
    ];
    let patch = prop_oneof![
        1 => Just(Patch::Null),
        8 => prop::collection::vec((state_key(), op), 1..3).prop_map(Patch::Ops),
        1 => Just(Patch::NotObject),
    ];
    let data = prop_oneof![
        2 => Just(Data::Null),
        6 => (-2i64..3).prop_map(Data::Obj),
        1 => Just(Data::Oversize),
    ];
    let list = prop_oneof![
        6 => (0u8..3).prop_map(Instances::Ok),
        1 => Just(Instances::Empty),
        1 => Just(Instances::TooMany),
        1 => Just(Instances::BlankId),
        1 => Just(Instances::BadId),
        1 => Just(Instances::LongTitle),
    ];
    let tpl = prop_oneof![
        1 => Just(Tpl::Null),
        4 => any::<bool>().prop_map(|title| Tpl::Obj { title }),
        1 => Just(Tpl::NotObject),
    ];
    let query = prop_oneof![
        grp().prop_map(Query::WidgetPushToken),
        Just(Query::PushToStartToken),
        Just(Query::CanPromote),
        Just(Query::OpenPromotionSettings),
        Just(Query::RequestWidget),
        Just(Query::ReloadAll),
        Just(Query::ReloadKind),
        Just(Query::RegisterWidgets),
    ];
    let act = prop_oneof![8 => (0u8..3).prop_map(Some), 1 => Just(None)];
    prop_oneof![
        3 => (0u8..2, 0u8..3, grp()).prop_map(|(key, value, group)| Action::SetItems { key, value, group }),
        2 => (0u8..2, grp()).prop_map(|(key, group)| Action::GetItems { key, group }),
        4 => (id(), inst(), cfg(), any::<bool>()).prop_map(|(id, inst, config, skip_reload)| Action::SetConfig { id, inst, config, skip_reload }),
        1 => (id(), inst()).prop_map(|(id, inst)| Action::GetConfig { id, inst }),
        3 => (id(), inst(), data).prop_map(|(id, inst, data)| Action::SetData { id, inst, data }),
        2 => (id(), list).prop_map(|(id, list)| Action::SetInstances { id, list }),
        6 => (id(), inst(), patch).prop_map(|(id, inst, patch)| Action::SetState { id, inst, patch }),
        1 => (id(), inst()).prop_map(|(id, inst)| Action::GetState { id, inst }),
        2 => (prop_oneof![6 => (0u8..2).prop_map(Some), 1 => Just(None)], tpl)
            .prop_map(|(name, layout)| Action::RegisterTemplate { name, layout }),
        4 => content().prop_map(Action::Start),
        3 => content().prop_map(Action::Update),
        3 => (act, prop::option::of(content()), choice(), prop_oneof![4 => Just(None), 1 => Just(Some(0u64)), 1 => Just(Some(40u64))])
            .prop_map(|(act, final_content, policy, dismiss_after_ms)| Action::End { act, final_content, policy, dismiss_after_ms }),
        1 => Just(Action::ListActivities),
        2 => (prop::option::of(any::<bool>()), prop::option::of(0u8..3)).prop_map(|(expanded, focus)| Action::Island { expanded, focus }),
        2 => query.prop_map(Action::Query),
        1 => grp().prop_map(Action::Poll),
        1 => (0u8..2, prop::option::of(0u8..2), prop::option::of(id()), grp())
            .prop_map(|(action, payload, widget, group)| Action::Tap { action, payload, widget, group }),
        2 => (0u8..2, 0u8..3, prop::bool::weighted(0.15)).prop_map(|(widget, inst, bad_group)| Action::Receipt { widget, inst, bad_group }),
        1 => prop::sample::select(&NEEDS_ARGS[..]).prop_map(Action::MissingArgs),
    ]
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    prop::collection::vec(
        (action(), prop::bool::weighted(0.1)).prop_map(|(action, extra)| Step { action, extra }),
        1..30,
    )
}

// ---- argument encoding (what JS sends) -------------------------------------------------

/// Names of one case: everything is prefixed so cases sharing the app (and its store
/// files) never see each other's keys.
struct Names {
    ns: String,
}

impl Names {
    fn widget(&self, n: u8) -> String {
        format!("{}-w{n}", self.ns)
    }
    fn id(&self, id: Id) -> String {
        match id {
            Id::W(n) => self.widget(n),
            Id::Bad => "bad/id".into(),
        }
    }
    fn act(&self, a: Option<u8>) -> String {
        a.map_or_else(|| "bad id".into(), |n| format!("{}-a{n}", self.ns))
    }
    fn tpl(&self, t: Option<u8>) -> String {
        t.map_or_else(|| "bad tpl".into(), |n| format!("{}-t{n}", self.ns))
    }
    fn key(&self, k: u8) -> String {
        format!("{}-k{k}", self.ns)
    }
    fn receipt_inst(&self, n: u8) -> String {
        format!("{}-r{n}", self.ns)
    }
}

fn inst_arg(args: &mut Value, inst: Inst) {
    match inst {
        Inst::None => {}
        Inst::P(n) => args["instance"] = json!(format!("p{n}")),
        Inst::Blank => args["instance"] = json!("  "),
        Inst::Bad => args["instance"] = json!("a b"),
    }
}

fn group_arg(args: &mut Value, g: Grp) {
    match g {
        Grp::Default => {}
        Grp::Blank => args["group"] = json!(" "),
        Grp::Other => args["group"] = json!("group.other.test"),
        Grp::Bad => args["group"] = json!("bad/group"),
    }
}

fn config_json(c: &Cfg) -> Value {
    match c {
        Cfg::Valid { content, state } => {
            let mut v =
                json!({"version": 1, "small": {"type": "text", "content": format!("c{content}")}});
            if let Some(s) = state {
                v["state"] = json!(s);
            }
            v
        }
        Cfg::UnknownElement => json!({"version": 1, "small": {"type": "nope"}}),
        Cfg::NotObject => json!("config"),
    }
}

fn data_json(d: &Data) -> Value {
    match d {
        Data::Null => Value::Null,
        Data::Obj(n) => json!({ "n": n }),
        Data::Oversize => json!({ "big": "x".repeat(256 * 1024) }),
    }
}

fn instances_json(l: &Instances) -> Value {
    let item = |id: String, title: String| json!({ "id": id, "title": title });
    match l {
        Instances::Ok(n) => (0..*n)
            .map(|i| item(format!("i{i}"), format!("T{i}")))
            .collect(),
        Instances::Empty => json!([]),
        Instances::TooMany => (0..201)
            .map(|i| item(format!("i{i}"), "t".into()))
            .collect(),
        Instances::BlankId => json!([item(" ".into(), "x".into())]),
        Instances::BadId => json!([item("a/b".into(), "x".into())]),
        Instances::LongTitle => json!([item("i".into(), "x".repeat(201))]),
    }
}

fn op_json(op: &Op) -> Value {
    match op {
        Op::Toggle => json!("$toggle"),
        Op::Inc => json!("$inc"),
        Op::Dec => json!("$dec"),
        Op::Null => Value::Null,
        Op::Set(v) => v.clone(),
    }
}

/// The patch object as sent (duplicate keys: the last one wins, like a JS object).
fn patch_map(ops: &[(String, Op)]) -> Map<String, Value> {
    ops.iter().map(|(k, op)| (k.clone(), op_json(op))).collect()
}

fn template_json(t: &Tpl) -> Value {
    match t {
        Tpl::Null => Value::Null,
        Tpl::Obj { title: true } => json!({"title": "Tpl {{data.name}}", "body": "tb"}),
        Tpl::Obj { title: false } => json!({"icon": "bolt"}),
        Tpl::NotObject => json!([1]),
    }
}

fn choice_str(c: Choice, valid: &str) -> &str {
    match c {
        Choice::Valid => valid,
        Choice::Bad => "nope",
    }
}

fn content_json(n: &Names, c: &Content) -> Value {
    let mut v = json!({
        "id": n.act(c.act),
        "title": c.title.map_or_else(|| "  ".to_string(), |t| format!("T{t}")),
    });
    if let Some(t) = c.template {
        v["template"] = json!(n.tpl(t));
        v["data"] = json!({ "name": format!("n{}", c.name) });
    }
    if let Some(p) = c.progress {
        v["progress"] = json!(p);
    }
    if c.actions > 0 {
        v["actions"] = (0..c.actions)
            .map(|i| json!({"id": format!("x{i}"), "label": "L"}))
            .collect();
    }
    if let Some(vis) = c.visibility {
        v["visibility"] = json!(choice_str(vis, "private"));
    }
    if let Some(cat) = c.category {
        v["category"] = json!(choice_str(cat, "workout"));
    }
    if c.big_body {
        v["body"] = json!("x".repeat(5000));
    }
    if let Some(ts) = c.timestamp {
        v["timestamp"] = json!(ts);
    }
    v
}

// ---- reference model --------------------------------------------------------------------

type Placement = (String, Option<String>);

#[derive(Clone, Debug, PartialEq)]
struct ConfigV {
    content: String,
    state: Option<Map<String, Value>>,
}

#[derive(Default)]
struct Model {
    /// (other group, key) → value.
    items: BTreeMap<(bool, String), String>,
    configs: BTreeMap<Placement, ConfigV>,
    /// `None` = cleared (stored as "").
    data: BTreeMap<Placement, Option<Value>>,
    instances: BTreeMap<String, Value>,
    state: BTreeMap<Placement, Map<String, Value>>,
    /// Registered template name → whether it has a title. Removed when unregistered.
    templates: BTreeMap<String, bool>,
    running: BTreeSet<String>,
    /// Island entries, oldest first (the last one is the pill).
    island: Vec<String>,
    last_ts: BTreeMap<String, u64>,
    /// Activity id → title of the last applied content (its stored layout keeps it).
    titles: BTreeMap<String, String>,
    /// Receipt instance → widget id of its latest receipt.
    receipts: BTreeMap<String, String>,
}

/// `Err(())` when the group is rejected; else whether it is another group's store.
fn group_ok(g: Grp) -> Result<bool, ()> {
    match g {
        Grp::Default | Grp::Blank => Ok(false),
        Grp::Other if APPLE => Err(()),
        Grp::Other => Ok(true),
        Grp::Bad => Err(()),
    }
}

/// Valid placement, or `Err` for an invalid widget id / instance.
fn placement(n: &Names, id: Id, inst: Inst) -> Result<Placement, ()> {
    let Id::W(w) = id else { return Err(()) };
    let inst = match inst {
        Inst::None | Inst::Blank => None,
        Inst::P(p) => Some(format!("p{p}")),
        Inst::Bad => return Err(()),
    };
    Ok((n.widget(w), inst))
}

fn scoped((w, i): &Placement) -> String {
    match i {
        Some(i) => format!("{w}#{i}"),
        None => w.clone(),
    }
}

/// `$inc` / `$dec`: integers stay integers while they fit `i64`, otherwise the float sum;
/// anything else starts from 0.
fn step_num(cur: Option<&Value>, d: i64) -> Value {
    match cur.and_then(Value::as_number) {
        Some(n) => match n.as_i64() {
            Some(i) => i
                .checked_add(d)
                .map_or_else(|| Value::from(i as f64 + d as f64), Value::from),
            None => Value::from(n.as_f64().unwrap() + d as f64),
        },
        None => Value::from(d),
    }
}

impl Model {
    fn defaults(&self, p: &Placement) -> Option<Map<String, Value>> {
        let own = p.1.as_ref().and_then(|_| self.configs.get(p));
        own.or_else(|| self.configs.get(&(p.0.clone(), None)))
            .and_then(|c| c.state.clone())
    }

    fn merged(&self, p: &Placement) -> Map<String, Value> {
        let mut out = self.defaults(p).unwrap_or_default();
        let widget = (p.0.clone(), None);
        let mut layers = vec![self.state.get(&widget)];
        if p.1.is_some() {
            layers.push(self.state.get(p));
        }
        for l in layers.into_iter().flatten() {
            out.extend(l.clone());
        }
        out
    }

    /// The content a surface receives, or `Err` when the request is rejected:
    /// validation first (a templated request as sent), then the stored template.
    fn prepare(&self, n: &Names, c: &Content) -> Result<(String, String), ()> {
        let id = c.act.map(|_| n.act(c.act)).ok_or(())?;
        let ok = !matches!(c.template, Some(None))
            && (c.template.is_some() || c.title.is_some())
            && c.actions <= 3
            && c.progress.map_or(true, |p| (0.0..=1.0).contains(&p))
            && !matches!(c.visibility, Some(Choice::Bad))
            && !matches!(c.category, Some(Choice::Bad))
            && !c.big_body;
        if !ok {
            return Err(());
        }
        let own = c.title.map(|t| format!("T{t}"));
        let title = match c.template {
            Some(t) => {
                let has_title = *self.templates.get(&n.tpl(t)).ok_or(())?;
                own.unwrap_or_else(|| {
                    if has_title {
                        format!("Tpl n{}", c.name)
                    } else {
                        String::new()
                    }
                })
            }
            None => own.unwrap_or_default(),
        };
        Ok((id, title))
    }

    /// Apply the step to the model and return the expected (projected) response, or
    /// `None` when the step is not observable deterministically (it is then not run).
    fn apply(&mut self, n: &Names, a: &Action) -> Option<Result<Value, ()>> {
        use Action::*;
        Some(match a {
            SetItems { key, value, group } => group_ok(*group).map(|other| {
                self.items.insert((other, n.key(*key)), format!("v{value}"));
                json!(true)
            }),
            GetItems { key, group } => {
                group_ok(*group).map(|other| json!(self.items.get(&(other, n.key(*key)))))
            }
            SetConfig {
                id,
                inst,
                config,
                skip_reload,
            } => {
                let Ok(p) = placement(n, *id, *inst) else {
                    return Some(Err(()));
                };
                let Cfg::Valid { content, state } = config else {
                    return Some(Err(()));
                };
                let v = ConfigV {
                    content: format!("c{content}"),
                    state: state
                        .as_ref()
                        .map(|s| json!(s).as_object().unwrap().clone()),
                };
                let written = self.configs.get(&p) != Some(&v);
                self.configs.insert(p, v);
                let reload = match (written, skip_reload) {
                    (true, false) => "ok",
                    _ => "skipped",
                };
                Ok(json!({
                    "written": written,
                    "reload": reload,
                    "skip": (!written).then_some("unchanged"),
                }))
            }
            GetConfig { id, inst } => {
                placement(n, *id, *inst).map(|p| match self.configs.get(&p) {
                    Some(c) => json!({"content": c.content, "state": c.state}),
                    None => Value::Null,
                })
            }
            SetData { id, inst, data } => placement(n, *id, *inst).and_then(|p| {
                let v = match data {
                    Data::Null => None,
                    Data::Obj(_) => Some(data_json(data)),
                    Data::Oversize => return Err(()),
                };
                self.data.insert(p, v);
                Ok(json!(true))
            }),
            SetInstances { id, list } => placement(n, *id, Inst::None).and_then(|(w, _)| {
                if !matches!(list, Instances::Ok(_) | Instances::Empty) {
                    return Err(());
                }
                self.instances.insert(w, instances_json(list));
                Ok(json!(true))
            }),
            SetState { id, inst, patch } => placement(n, *id, *inst).and_then(|p| {
                match patch {
                    Patch::NotObject => return Err(()),
                    Patch::Null => {
                        self.state.remove(&p);
                    }
                    Patch::Ops(ops) => {
                        let cur = self.merged(&p);
                        let layer = self.state.entry(p.clone()).or_default();
                        for (k, v) in patch_map(ops) {
                            let now = cur.get(&k);
                            let next = match v.as_str() {
                                _ if v.is_null() => {
                                    layer.remove(&k);
                                    continue;
                                }
                                Some("$toggle") => Value::Bool(now != Some(&Value::Bool(true))),
                                Some("$inc") => step_num(now, 1),
                                Some("$dec") => step_num(now, -1),
                                _ => v,
                            };
                            layer.insert(k, next);
                        }
                    }
                }
                Ok(Value::Object(self.merged(&p)))
            }),
            GetState { id, inst } => {
                placement(n, *id, *inst).map(|p| Value::Object(self.merged(&p)))
            }
            RegisterTemplate { name, layout } => {
                if name.is_none() {
                    return Some(Err(()));
                }
                let key = n.tpl(*name);
                match layout {
                    Tpl::Null => {
                        self.templates.remove(&key);
                    }
                    Tpl::Obj { title } => {
                        self.templates.insert(key, *title);
                    }
                    Tpl::NotObject => return Some(Err(())),
                }
                Ok(json!(true))
            }
            Start(c) => self.prepare(n, c).map(|(id, title)| {
                if !self.running.contains(&id) {
                    self.last_ts.remove(&id);
                    self.island.push(id.clone());
                    self.running.insert(id.clone());
                }
                if let Some(ts) = c.timestamp {
                    self.last_ts.insert(id.clone(), ts);
                }
                self.titles.insert(id.clone(), title);
                json!({"id": id, "surface": "desktop"})
            }),
            Update(c) => self
                .prepare(n, c)
                .and_then(|(id, title)| self.update(&id, title, c.timestamp)),
            End {
                act,
                final_content,
                policy,
                dismiss_after_ms,
            } => {
                // A delayed dismissal finishes on a timer (story: tests/activities.rs).
                if dismiss_after_ms.is_some_and(|ms| ms > 0) {
                    return None;
                }
                let id = act.map(|_| n.act(*act)).ok_or(());
                let final_ok = match final_content {
                    Some(c) if c.act != *act => Err(()),
                    Some(c) => self.prepare(n, c).map(|t| Some((t.1, c.timestamp))),
                    None => Ok(None),
                };
                match (id, policy, final_ok) {
                    (Ok(id), None | Some(Choice::Valid), Ok(fin)) => {
                        if let Some((title, ts)) = fin {
                            let _ = self.update(&id, title, ts);
                        }
                        let was = self.running.remove(&id);
                        if was {
                            self.last_ts.remove(&id);
                            self.island.retain(|x| *x != id);
                        }
                        Ok(json!(was))
                    }
                    _ => Err(()),
                }
            }
            ListActivities => Ok(json!(self.running)),
            Island { expanded, focus } => {
                // Expanding starts the hover poller's collapse timer (story:
                // tests/activities.rs); the model keeps the island collapsed.
                if *expanded == Some(true) && !self.island.is_empty() {
                    return None;
                }
                if self.island.is_empty() {
                    Ok(Value::Null)
                } else {
                    if let Some(f) = focus {
                        let id = n.act(Some(*f));
                        if let Some(i) = self.island.iter().position(|x| *x == id) {
                            let e = self.island.remove(i);
                            self.island.push(e);
                        }
                    }
                    Ok(self.island_view())
                }
            }
            Query(q) => match q {
                self::Query::WidgetPushToken(g) => group_ok(*g).map(|_| Value::Null),
                self::Query::PushToStartToken => Ok(Value::Null),
                self::Query::CanPromote => Ok(json!(true)),
                self::Query::OpenPromotionSettings => Ok(json!(false)),
                self::Query::RequestWidget => Err(()),
                self::Query::ReloadAll | self::Query::ReloadKind | self::Query::RegisterWidgets => {
                    Ok(json!(true))
                }
            },
            Poll(g) => group_ok(*g).map(|_| json!([])),
            Tap { .. } => Ok(json!(true)),
            Receipt {
                widget,
                inst,
                bad_group,
            } => {
                if *bad_group {
                    Err(())
                } else {
                    self.receipts
                        .insert(n.receipt_inst(*inst), n.widget(*widget));
                    Ok(json!(true))
                }
            }
            MissingArgs(_) => Err(()),
        })
    }

    fn update(&mut self, id: &str, title: String, ts: Option<u64>) -> Result<Value, ()> {
        if !self.running.contains(id) {
            return Err(());
        }
        if let Some(ts) = ts {
            if self.last_ts.get(id).is_some_and(|last| ts < *last) {
                return Ok(json!(false));
            }
            self.last_ts.insert(id.into(), ts);
        }
        self.titles.insert(id.into(), title);
        Ok(json!(true))
    }

    /// What the island shows: the pill first, then up to three minimal activities.
    fn island_view(&self) -> Value {
        let ids: Vec<&String> = self.island.iter().rev().take(4).collect();
        json!({"expanded": false, "ids": ids})
    }
}

// ---- the system ---------------------------------------------------------------------------

fn args_of(n: &Names, a: &Action) -> (&'static str, Value) {
    use Action::*;
    let mut args = json!({});
    let cmd = match a {
        SetItems { key, value, group } => {
            args = json!({"key": n.key(*key), "value": format!("v{value}")});
            group_arg(&mut args, *group);
            "set_items"
        }
        GetItems { key, group } => {
            args = json!({"key": n.key(*key)});
            group_arg(&mut args, *group);
            "get_items"
        }
        SetConfig {
            id,
            inst,
            config,
            skip_reload,
        } => {
            args = json!({"widgetId": n.id(*id), "config": config_json(config), "skipReload": skip_reload});
            inst_arg(&mut args, *inst);
            "set_widget_config"
        }
        GetConfig { id, inst } => {
            args = json!({"widgetId": n.id(*id)});
            inst_arg(&mut args, *inst);
            "get_widget_config"
        }
        SetData { id, inst, data } => {
            args = json!({"widgetId": n.id(*id), "data": data_json(data), "skipReload": true});
            inst_arg(&mut args, *inst);
            "set_widget_data"
        }
        SetInstances { id, list } => {
            args = json!({"widgetId": n.id(*id), "instances": instances_json(list)});
            "set_widget_instances"
        }
        SetState { id, inst, patch } => {
            let patch = match patch {
                Patch::Null => Value::Null,
                Patch::Ops(ops) => Value::Object(patch_map(ops)),
                Patch::NotObject => json!([1]),
            };
            args = json!({"widgetId": n.id(*id), "patch": patch, "skipReload": true});
            inst_arg(&mut args, *inst);
            "set_widget_state"
        }
        GetState { id, inst } => {
            args = json!({"widgetId": n.id(*id)});
            inst_arg(&mut args, *inst);
            "get_widget_state"
        }
        RegisterTemplate { name, layout } => {
            args = json!({"name": n.tpl(*name), "layout": template_json(layout)});
            "register_activity_template"
        }
        Start(c) => {
            args = json!({"content": content_json(n, c)});
            "start_activity"
        }
        Update(c) => {
            args = json!({"content": content_json(n, c)});
            "update_activity"
        }
        End {
            act,
            final_content,
            policy,
            dismiss_after_ms,
        } => {
            args = json!({"id": n.act(*act)});
            if let Some(c) = final_content {
                args["content"] = content_json(n, c);
            }
            if let Some(p) = policy {
                args["dismissalPolicy"] = json!(choice_str(*p, "immediate"));
            }
            if let Some(ms) = dismiss_after_ms {
                args["dismissAfterMs"] = json!(ms);
            }
            "end_activity"
        }
        ListActivities => "list_activities",
        Island { expanded, focus } => {
            args = json!({"expanded": expanded, "focus": focus.map(|f| n.act(Some(f)))});
            "activity_island"
        }
        Query(q) => match q {
            self::Query::WidgetPushToken(g) => {
                group_arg(&mut args, *g);
                "get_widget_push_token"
            }
            self::Query::PushToStartToken => "get_activity_push_to_start_token",
            self::Query::CanPromote => "can_promote_activities",
            self::Query::OpenPromotionSettings => "open_activity_promotion_settings",
            self::Query::RequestWidget => "request_widget",
            self::Query::ReloadAll => "reload_all_timelines",
            self::Query::ReloadKind => {
                args = json!({"ofKind": n.widget(0)});
                "reload_timelines"
            }
            self::Query::RegisterWidgets => {
                args = json!({"widgets": [n.widget(0)]});
                "set_register_widget"
            }
        },
        Poll(g) => {
            group_arg(&mut args, *g);
            "poll_pending_actions"
        }
        Tap {
            action,
            payload,
            widget,
            group,
        } => {
            args = json!({"action": format!("tap{action}"), "payload": payload.map(|p| format!("p{p}"))});
            if let Some(w) = widget {
                args["widgetId"] = json!(n.id(*w));
            }
            group_arg(&mut args, *group);
            "widget_action"
        }
        Receipt {
            widget,
            inst,
            bad_group,
        } => {
            args = json!({"receipt": {
                "widgetId": n.widget(*widget),
                "group": if *bad_group { "bad/group" } else { DEFAULT_GROUP },
                "instance": n.receipt_inst(*inst),
                "nonce": 0,
                "source": "prefs",
                "ts": 0,
            }});
            "report_receipt"
        }
        MissingArgs(cmd) => cmd,
    };
    (cmd, args)
}

/// The part of a response the model predicts.
fn project(a: &Action, v: Value) -> Value {
    match a {
        Action::SetConfig { .. } => json!({
            "written": v["written"],
            "reload": v["reload"]["outcome"],
            "skip": v.get("skip").map(|s| s["reason"].clone()),
        }),
        Action::GetConfig { .. } if !v.is_null() => {
            json!({"content": v["small"]["content"], "state": v.get("state")})
        }
        Action::Start(_) => json!({"id": v["id"], "surface": v["surface"]}),
        Action::Island { .. } if !v.is_null() => json!({
            "expanded": v["expanded"],
            "ids": v["activities"].as_array().unwrap().iter().map(|a| a["id"].clone()).collect::<Vec<_>>(),
        }),
        _ => v,
    }
}

/// The event a successful step must emit: (name, payload check).
fn expected_event(
    n: &Names,
    a: &Action,
    resp: &Value,
    changed_item: bool,
) -> Option<(&'static str, Value)> {
    let group_of = |g: Grp| match g {
        Grp::Other if !APPLE => "group.other.test",
        Grp::Other | Grp::Bad => "",
        _ => DEFAULT_GROUP,
    };
    let inst_of = |i: Inst| match i {
        Inst::P(p) => json!(format!("p{p}")),
        _ => Value::Null,
    };
    match a {
        Action::SetItems { group, .. } if changed_item => {
            Some(("widget-update", json!(group_of(*group))))
        }
        Action::SetConfig {
            id, inst, config, ..
        } => {
            let p = placement(n, *id, *inst).ok()?;
            let Cfg::Valid { content, .. } = config else {
                return None;
            };
            Some((
                "widget-config-push",
                json!({"widgetId": scoped(&p), "content": format!("c{content}")}),
            ))
        }
        Action::SetData { id, inst, data, .. } => Some((
            "widget-data-push",
            json!({"widgetId": n.id(*id), "instance": inst_of(*inst), "data": data_json(data)}),
        )),
        Action::SetState { id, inst, .. } => Some((
            "widget-state-changed",
            json!({"widgetId": n.id(*id), "instance": inst_of(*inst), "state": resp}),
        )),
        Action::Tap {
            action,
            payload,
            widget,
            group,
        } => Some((
            "widget-action",
            json!({
                "action": format!("tap{action}"),
                "payload": payload.map(|p| format!("p{p}")),
                "widgetId": widget.map(|w| n.id(w)).unwrap_or_default(),
                "group": group_of(*group),
            }),
        )),
        _ => None,
    }
}

/// The fields of an event payload the expectation names.
fn event_view(name: &str, p: &Value) -> Value {
    match name {
        "widget-config-push" => {
            json!({"widgetId": p["widgetId"], "content": p["config"]["small"]["content"]})
        }
        "widget-data-push" | "widget-state-changed" => {
            let key = if name == "widget-data-push" {
                "data"
            } else {
                "state"
            };
            json!({"widgetId": p["widgetId"], "instance": p["instance"], key: p[key]})
        }
        "widget-action" => {
            json!({"action": p["action"], "payload": p["payload"], "widgetId": p["widgetId"], "group": p["group"]})
        }
        _ => p.clone(),
    }
}

fn get(app: &App, cmd: &str, args: Value) -> Value {
    app.call(cmd, args)
        .unwrap_or_else(|e| panic!("{cmd} failed: {e}"))
}

/// The JSON stored under `key` (`None` when missing, `Some(Null)` when empty).
fn stored(app: &App, key: &str) -> Option<Value> {
    match get(app, "get_items", json!({ "key": key })) {
        Value::Null => None,
        Value::String(s) if s.is_empty() => Some(Value::Null),
        Value::String(s) => Some(serde_json::from_str(&s).unwrap()),
        v => panic!("get_items {key}: {v}"),
    }
}

/// Observable state equals the model's.
fn check_invariants(app: &App, n: &Names, m: &Model) -> Result<(), String> {
    macro_rules! eq {
        ($got:expr, $want:expr, $($what:tt)*) => {{
            let (got, want) = ($got, $want);
            if got != want {
                return Err(format!("{}: got {got:?}, model {want:?}", format!($($what)*)));
            }
        }};
    }
    let placements: Vec<Placement> = (0..2)
        .flat_map(|w| {
            [None, Some("p0".to_string()), Some("p1".to_string())].map(|i| (n.widget(w), i))
        })
        .collect();
    for p in &placements {
        let mut args = json!({"widgetId": p.0});
        if let Some(i) = &p.1 {
            args["instance"] = json!(i);
        }
        eq!(
            get(app, "get_widget_state", args.clone()),
            Value::Object(m.merged(p)),
            "state {p:?}"
        );
        let cfg = project(
            &Action::GetConfig {
                id: Id::Bad,
                inst: Inst::None,
            },
            get(app, "get_widget_config", args),
        );
        let want = m.configs.get(p).map_or(
            Value::Null,
            |c| json!({"content": c.content, "state": c.state}),
        );
        eq!(cfg, want, "config {p:?}");
        let key = format!("data:{}", scoped(p));
        eq!(
            stored(app, &key),
            m.data.get(p).map(|d| d.clone().unwrap_or(Value::Null)),
            "stored {key}"
        );
    }
    for w in 0..2 {
        let w = n.widget(w);
        eq!(
            stored(app, &format!("instances:{w}")),
            m.instances.get(&w).cloned(),
            "instances of {w}"
        );
    }
    for t in 0..2 {
        let key = format!("activity-template:{}", n.tpl(Some(t)));
        let registered = stored(app, &key).is_some_and(|v| !v.is_null());
        eq!(
            registered,
            m.templates.contains_key(&n.tpl(Some(t))),
            "{key} registered"
        );
    }
    let running: Vec<String> =
        serde_json::from_value(get(app, "list_activities", json!({}))).unwrap();
    let ours: BTreeSet<String> = running
        .into_iter()
        .filter(|id| id.starts_with(&n.ns))
        .collect();
    eq!(json!(ours), json!(m.running), "list_activities");
    let island = get(app, "activity_island", json!({}));
    let want = if m.island.is_empty() {
        Value::Null
    } else {
        m.island_view()
    };
    eq!(
        project(
            &Action::Island {
                expanded: None,
                focus: None
            },
            island
        ),
        want,
        "island"
    );
    for (id, title) in &m.titles {
        let cfg = get(
            app,
            "get_widget_config",
            json!({"widgetId": format!("activity.{id}")}),
        );
        let shows = title.is_empty()
            || cfg
                .to_string()
                .contains(&format!("\"content\":\"{title}\""));
        eq!(shows, true, "activity {id} layout shows {title:?}: {cfg}");
    }
    let diag: BTreeMap<String, String> = get(app, "get_widget_diagnostics", json!({}))
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["instance"].as_str().unwrap().starts_with(&n.ns))
        .map(|r| {
            (
                r["instance"].as_str().unwrap().into(),
                r["widgetId"].as_str().unwrap().into(),
            )
        })
        .collect();
    eq!(json!(diag), json!(m.receipts), "diagnostics");
    let active: BTreeSet<(String, String)> = get(app, "get_active_widgets", json!({}))
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["widgetId"].as_str().unwrap().starts_with(&n.ns))
        .map(|a| {
            (
                a["widgetId"].as_str().unwrap().into(),
                a["instance"].as_str().unwrap_or("").into(),
            )
        })
        .collect();
    let want: BTreeSet<(String, String)> = m
        .receipts
        .iter()
        .map(|(i, w)| (w.clone(), i.clone()))
        .collect();
    eq!(json!(active), json!(want), "active widgets");
    Ok(())
}

fn run_case(app: &App, ns: String, steps: &[Step]) -> Result<(), String> {
    let n = Names { ns };
    let mut m = Model::default();
    app.drain_events();
    for (i, s) in steps.iter().enumerate() {
        let before = match &s.action {
            Action::SetItems { key, group, .. } => group_ok(*group)
                .ok()
                .map(|other| m.items.get(&(other, n.key(*key))).cloned()),
            _ => None,
        };
        let Some(want) = m.apply(&n, &s.action) else {
            continue;
        };
        let (cmd, mut args) = args_of(&n, &s.action);
        if s.extra {
            args["extraField"] = json!({"ignored": true});
        }
        let got = app.call(cmd, args);
        let events = app.drain_events();
        let got = got.map(|v| project(&s.action, v)).map_err(|_| ());
        if got != want {
            return Err(format!("step {i} {cmd}: got {got:?}, model {want:?}"));
        }
        if let Ok(resp) = &got {
            let changed = matches!(&s.action, Action::SetItems { value, .. }
                if before.as_ref().is_some_and(|b| b.as_deref() != Some(format!("v{value}").as_str())));
            if let Some((name, want)) = expected_event(&n, &s.action, resp, changed) {
                if !events
                    .iter()
                    .any(|e| e.name == name && event_view(name, &e.payload) == want)
                {
                    return Err(format!("step {i} {cmd}: no {name} {want} in {events:?}"));
                }
            }
            if matches!(s.action, Action::SetItems { .. }) && !changed {
                if let Some(e) = events.iter().find(|e| e.name == "widget-update") {
                    return Err(format!("step {i} {cmd}: unchanged value emitted {e:?}"));
                }
            }
        }
        check_invariants(app, &n, &m).map_err(|e| format!("after step {i} {cmd}: {e}"))?;
    }
    Ok(())
}

#[test]
fn plugin_follows_the_model_over_ipc() {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(16);
    let mut runner = TestRunner::new(Config {
        cases,
        failure_persistence: None,
        ..Config::default()
    });
    let case = Cell::new(0u32);
    let result = runner.run(&steps(), |steps| {
        case.set(case.get() + 1);
        // Each case (and shrink run) on a fresh app over an empty store.
        let app = App::clean();
        let result = run_case(&app, format!("c{}", case.get()), &steps);
        // Ending the activities lets the island's hover poller thread finish.
        let running: Vec<String> =
            serde_json::from_value(get(&app, "list_activities", json!({}))).unwrap();
        for id in running {
            get(&app, "end_activity", json!({"id": id}));
        }
        result.map_err(proptest::test_runner::TestCaseError::fail)
    });
    match result {
        Ok(()) => {}
        Err(TestError::Fail(why, steps)) => {
            panic!("{why}\nminimal scenario:\n{steps:#?}")
        }
        Err(e) => panic!("{e}"),
    }
}
