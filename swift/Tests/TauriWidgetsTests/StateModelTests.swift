import Foundation
import PropertyBased
import Testing
@testable import TauriWidgets

/// Model-based chain over the App Group state store: generated command sequences run
/// against `TauriWidgetDataStore` (the extension's `set` path) and against a plain
/// dictionary model; every scope's merged read is compared after every step.
///
/// Serialized: the store paths come from process environment variables.
@Suite(.serialized)
struct StateModelTests {
    /// Model value: what a JSON scalar in the store reads back as.
    enum V: Equatable, Sendable, CustomStringConvertible {
        case num(Double), bool(Bool), str(String)

        var description: String {
            switch self {
            case .num(let n): return "\(n)"
            case .bool(let b): return "\(b)"
            case .str(let s): return "\"\(s)\""
            }
        }

        var json: Any {
            switch self {
            case .num(let n): return n
            case .bool(let b): return b
            case .str(let s): return s
            }
        }

        init?(_ any: Any?) {
            if let s = any as? String { self = .str(s); return }
            guard let n = any as? NSNumber else { return nil }
            self = TauriResolve.isBool(n) ? .bool(n.boolValue) : .num(n.doubleValue)
        }
    }

    /// Scopes: the widget layer `w` and two placements. A placement reads
    /// `config.state` < `state:w` < `state:w#i`; the widget scope reads the first two.
    static let scopes: [String?] = [nil, "a", "b"]
    static let keys = ["on", "n", "f", "s"]
    /// `config.state` defaults: a bool, an int, a fraction (for `$inc` on non-integers).
    static let defaults: [String: V] = ["on": .bool(false), "n": .num(1), "f": .num(1.5)]

    enum Op: Equatable, Sendable, CustomStringConvertible {
        case toggle, inc, dec, remove, set(V)

        var json: Any {
            switch self {
            case .toggle: return "$toggle"
            case .inc: return "$inc"
            case .dec: return "$dec"
            case .remove: return NSNull()
            case .set(let v): return v.json
            }
        }

        var description: String {
            switch self {
            case .toggle: return "$toggle"
            case .inc: return "$inc"
            case .dec: return "$dec"
            case .remove: return "null"
            case .set(let v): return v.description
            }
        }
    }

    enum Command: Sendable, CustomStringConvertible {
        /// One patch on one scope; several keys at once (later ops on the same key win).
        case patch(scope: Int, ops: [(key: String, op: Op)])
        /// Host `setWidgetState(patch: null)`: the scope's stored layer is emptied.
        case clear(scope: Int)
        /// Raw read of the scope's own stored layer.
        case read(scope: Int)

        var description: String {
            switch self {
            case .patch(let s, let ops): return "patch(\(s), \(ops.map { "\($0.key):\($0.op)" }.joined(separator: ",")))"
            case .clear(let s): return "clear(\(s))"
            case .read(let s): return "read(\(s))"
            }
        }
    }

    static func op(_ kind: Int, _ value: Int) -> Op {
        switch kind {
        case 0: return .toggle
        case 1: return .inc
        case 2: return .dec
        case 3: return .remove
        case 4: return .set(.num(Double(value)))
        case 5: return .set(.bool(value % 2 == 0))
        default: return .set(.str("v\(value)"))
        }
    }

    static let opGen = zip(Gen.int(in: 0..<keys.count), Gen.int(in: 0...6), Gen.int(in: -3...3))
        .map { k, kind, v in (key: keys[k], op: op(kind, v)) }

    /// Patches dominate; clears and reads are rarer.
    static let commandGen = zip(Gen.int(in: 0...9), Gen.int(in: 0..<scopes.count), opGen.array(of: 1...3))
        .map { kind, scope, ops -> Command in
            switch kind {
            case 0: return .clear(scope: scope)
            case 1: return .read(scope: scope)
            default: return .patch(scope: scope, ops: ops)
            }
        }

    /// Plain-dictionary model of the store's state layers.
    struct Model {
        var layers: [String: [String: V]] = [:]

        static func key(_ scope: Int) -> String {
            TauriWidgetStoreKeys.scopedId("w", StateModelTests.scopes[scope])
        }

        func merged(_ scope: Int) -> [String: V] {
            var out = StateModelTests.defaults
            var names = ["w"]
            if scope != 0 { names.append(Model.key(scope)) }
            for n in names { out.merge(layers[n] ?? [:]) { _, new in new } }
            return out
        }

        mutating func patch(_ scope: Int, _ ops: [(key: String, op: Op)]) {
            var last: [String: Op] = [:]
            for o in ops { last[o.key] = o.op }
            let current = merged(scope)
            var own = layers[Model.key(scope)] ?? [:]
            for (k, op) in last {
                switch op {
                case .toggle:
                    own[k] = .bool(!(current[k] == .bool(true)))
                case .inc, .dec:
                    var n = 0.0
                    if case .num(let c)? = current[k] { n = c }
                    own[k] = .num(n + (op == .inc ? 1 : -1))
                case .remove:
                    own[k] = nil
                case .set(let v):
                    own[k] = v
                }
            }
            layers[Model.key(scope)] = own
        }
    }

    static func decode(_ any: Any?) -> [String: V]? {
        guard let dict = (any ?? [String: Any]()) as? [String: Any] else { return nil }
        var out: [String: V] = [:]
        for (k, v) in dict {
            guard let mv = V(v) else { return nil }
            out[k] = mv
        }
        return out
    }

    /// Given generated patch / clear / read sequences, when run on the store and a dictionary model, then every scope's merged read agrees after every step.
    @Test func storeMatchesModel() async {
        await propertyCheck(count: 60, input: Self.commandGen.array(of: 1...25)) { commands in
            let box = StoreSandbox("state-model")
            defer { box.restore() }
            let group = box.group
            let cfg = #"{"state":{"on":false,"n":1,"f":1.5},"small":{"type":"text","content":"{{state.n}}"}}"#
            TauriWidgetDataStore.writeValue(cfg, forKey: TauriWidgetStoreKeys.configKey("w"), appGroup: group)
            var model = Model()

            for (step, cmd) in commands.enumerated() {
                // The (shrunk) whole sequence goes into every failure message.
                let trace = "step \(step): \(cmd) in \(commands)"
                switch cmd {
                case .patch(let scope, let ops):
                    var patch: [String: Any] = [:]
                    for o in ops { patch[o.key] = o.op.json }
                    let returned = TauriWidgetDataStore.applyStatePatch(
                        patch, appGroup: group, widgetId: "w", instance: Self.scopes[scope])
                    model.patch(scope, ops)
                    // The returned merged state is what the tap's `$state` event carries.
                    #expect(Self.decode(returned) == model.merged(scope), "\(trace) returned")
                case .clear(let scope):
                    TauriWidgetDataStore.writeValue("", forKey: TauriWidgetStoreKeys.statePrefix + Model.key(scope),
                                                    appGroup: group)
                    model.layers[Model.key(scope)] = nil
                case .read(let scope):
                    let raw = TauriWidgetDataStore.readValue(
                        forKey: TauriWidgetStoreKeys.statePrefix + Model.key(scope), appGroup: group)
                    #expect(Self.decode(TauriWidgetSnapshot.json(raw)) == (model.layers[Model.key(scope)] ?? [:]),
                            "\(trace) own layer")
                }
                // After every step: each scope's merged read matches the model.
                for scope in Self.scopes.indices {
                    let snap = TauriWidgetDataStore.loadSnapshot(appGroup: group, widgetId: "w",
                                                                 instance: Self.scopes[scope])
                    let got = Self.decode(snap.state)
                    let want = model.merged(scope)
                    #expect(got == want, "\(trace) scope \(scope): store \(String(describing: got)) model \(want)")
                    if got != want { return }
                }
            }
        }
    }
}
