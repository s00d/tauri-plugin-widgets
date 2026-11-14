//! Model-based test of the widget state store (`src/store.rs`): random chains of `set` /
//! `setWidgetState` patches, layer clears and renders, checked after every step against a
//! plain `BTreeMap` model of the three layers (config defaults → `state:{widgetId}` →
//! `state:{widgetId}#{instance}`). `PROPTEST_CASES` overrides the case count.

use std::collections::{BTreeMap, HashMap};

use proptest::prelude::*;
use proptest::test_runner::Config;
use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};
use serde_json::{json, Map, Value};
use tauri_plugin_widgets::resolve::{resolve, ResolveContext};
use tauri_plugin_widgets::store::{apply_state_patch, merged_state, state_key};

const WIDGET: &str = "w";
const INSTANCE: &str = "i";
const KEYS: [&str; 3] = ["a", "b", "c"];

type Layer = BTreeMap<String, Value>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Scope {
    Widget,
    Instance,
}

impl Scope {
    fn instance(self) -> Option<&'static str> {
        (self == Scope::Instance).then_some(INSTANCE)
    }
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
enum Transition {
    SetPatch {
        scope: Scope,
        key: String,
        op: Op,
    },
    ClearScope(Scope),
    /// Render `bind: {value: "state.<key>"}` with the merged state.
    ReadMerged(String),
}

#[derive(Clone, Debug)]
struct Model {
    defaults: Layer,
    widget: Layer,
    instance: Layer,
}

impl Model {
    /// What a placement of `scope` sees: defaults, then the widget layer, then (for an
    /// instance) its own layer.
    fn merged(&self, scope: Scope) -> Layer {
        let mut out = self.defaults.clone();
        out.extend(self.widget.clone());
        if scope == Scope::Instance {
            out.extend(self.instance.clone());
        }
        out
    }

    fn layer(&mut self, scope: Scope) -> &mut Layer {
        match scope {
            Scope::Widget => &mut self.widget,
            Scope::Instance => &mut self.instance,
        }
    }
}

/// Spec of `$inc` / `$dec`: integers stay integers while they fit `i64`, otherwise (and for
/// floats) the float sum; anything else starts from 0.
fn step(cur: Option<&Value>, d: i64) -> Value {
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

fn value() -> impl Strategy<Value = Value> {
    prop_oneof![
        (-3i64..3).prop_map(Value::from),
        any::<bool>().prop_map(Value::from),
        "[a-z]{0,2}".prop_map(Value::from),
        Just(json!(0.5)),
        Just(json!(i64::MAX)),
        Just(json!(i64::MIN)),
        Just(json!(u64::MAX)),
        Just(json!([1])),
    ]
}

fn key() -> impl Strategy<Value = String> {
    prop::sample::select(&KEYS[..]).prop_map(String::from)
}

fn scope() -> impl Strategy<Value = Scope> {
    prop_oneof![Just(Scope::Widget), Just(Scope::Instance)]
}

struct StateModel;

impl ReferenceStateMachine for StateModel {
    type State = Model;
    type Transition = Transition;

    fn init_state() -> BoxedStrategy<Model> {
        prop::collection::btree_map(key(), value(), 0..=3)
            .prop_map(|defaults| Model {
                defaults,
                widget: Layer::new(),
                instance: Layer::new(),
            })
            .boxed()
    }

    fn transitions(_: &Model) -> BoxedStrategy<Transition> {
        let op = prop_oneof![
            Just(Op::Toggle),
            Just(Op::Inc),
            Just(Op::Dec),
            Just(Op::Null),
            value().prop_map(Op::Set),
        ];
        prop_oneof![
            6 => (scope(), key(), op).prop_map(|(scope, key, op)| Transition::SetPatch { scope, key, op }),
            1 => scope().prop_map(Transition::ClearScope),
            2 => key().prop_map(Transition::ReadMerged),
        ]
        .boxed()
    }

    fn apply(mut m: Model, t: &Transition) -> Model {
        match t {
            Transition::SetPatch { scope, key, op } => {
                let cur = m.merged(*scope).get(key).cloned();
                let next = match op {
                    Op::Toggle => Some(Value::Bool(cur != Some(Value::Bool(true)))),
                    Op::Inc => Some(step(cur.as_ref(), 1)),
                    Op::Dec => Some(step(cur.as_ref(), -1)),
                    Op::Null => None,
                    Op::Set(v) => Some(v.clone()),
                };
                let layer = m.layer(*scope);
                match next {
                    Some(v) => layer.insert(key.clone(), v),
                    None => layer.remove(key),
                };
            }
            Transition::ClearScope(scope) => m.layer(*scope).clear(),
            Transition::ReadMerged(_) => {}
        }
        m
    }
}

/// The host side: config defaults plus the shared store keyed like the real one.
struct Store {
    defaults: Value,
    kv: HashMap<String, Value>,
}

impl Store {
    fn merged(&self, scope: Scope) -> Map<String, Value> {
        let widget = self.kv.get(&state_key(WIDGET, None));
        let instance = match scope {
            Scope::Instance => self.kv.get(&state_key(WIDGET, Some(INSTANCE))),
            Scope::Widget => None,
        };
        merged_state(&[Some(&self.defaults), widget, instance])
    }
}

struct StateStoreTest;

impl StateMachineTest for StateStoreTest {
    type SystemUnderTest = Store;
    type Reference = StateModel;

    fn init_test(m: &Model) -> Store {
        Store {
            defaults: json!(m.defaults),
            kv: HashMap::new(),
        }
    }

    fn apply(mut s: Store, m: &Model, t: Transition) -> Store {
        match t {
            Transition::SetPatch { scope, key, op } => {
                let patch_value = match op {
                    Op::Toggle => json!("$toggle"),
                    Op::Inc => json!("$inc"),
                    Op::Dec => json!("$dec"),
                    Op::Null => Value::Null,
                    Op::Set(v) => v,
                };
                let current = s.merged(scope);
                let k = state_key(WIDGET, scope.instance());
                let mut layer =
                    s.kv.get(&k)
                        .and_then(Value::as_object)
                        .cloned()
                        .unwrap_or_default();
                let patch = Map::from_iter([(key, patch_value)]);
                apply_state_patch(&mut layer, &current, &patch);
                s.kv.insert(k, Value::Object(layer));
            }
            Transition::ClearScope(scope) => {
                s.kv.remove(&state_key(WIDGET, scope.instance()));
            }
            Transition::ReadMerged(key) => {
                // The instance placement renders what the model merged; a missing value
                // keeps the literal.
                let state = Value::Object(s.merged(Scope::Instance));
                let cfg = json!({"small": {"type": "progress", "value": "literal",
                    "bind": {"value": format!("state.{key}")}}});
                let ctx = ResolveContext {
                    state: Some(&state),
                    ..Default::default()
                };
                let want = m
                    .merged(Scope::Instance)
                    .get(&key)
                    .cloned()
                    .unwrap_or(json!("literal"));
                assert_eq!(resolve(&cfg, &ctx)["small"]["value"], want);
            }
        }
        s
    }

    fn check_invariants(s: &Store, m: &Model) {
        for scope in [Scope::Widget, Scope::Instance] {
            let got: Layer = s.merged(scope).into_iter().collect();
            assert_eq!(got, m.merged(scope), "{scope:?} merged state");
        }
        // Only the two scoped keys are ever written.
        for k in s.kv.keys() {
            assert!(k == "state:w" || k == "state:w#i", "unexpected key {k}");
        }
    }
}

prop_state_machine! {
    #![proptest_config(Config {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|c| c.parse().ok()).unwrap_or(64),
        failure_persistence: None,
        ..Config::default()
    })]

    #[test]
    fn state_layers_follow_the_model(sequential 1..30 => StateStoreTest);
}
