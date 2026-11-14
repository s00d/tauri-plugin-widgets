import Foundation

/// Port of `src/resolve.rs`: scheduled entries and `{{...}}` placeholders, run on the stored
/// JSON right before decoding. Checked against `tests/fixtures/resolve/*.json` like the
/// Rust, Kotlin and TypeScript ports.
public enum TauriResolve {
    public struct Context {
        public var nowMs: UInt64
        public var locale: String
        public var instance: String?
        /// Data layers merged with `mergeData`.
        public var data: Any?
        /// Placement state (`config.state` < `state:{id}` < `state:{id}#{instance}`), merged.
        public var state: Any?

        public init(nowMs: UInt64, locale: String, instance: String? = nil, data: Any? = nil, state: Any? = nil) {
            self.nowMs = nowMs; self.locale = locale; self.instance = instance; self.data = data; self.state = state
        }

        /// Now, the device locale (BCP-47).
        public static func current(instance: String? = nil, data: Any? = nil, state: Any? = nil) -> Context {
            Context(
                nowMs: UInt64(Date().timeIntervalSince1970 * 1000),
                locale: Locale.current.identifier(.bcp47),
                instance: instance,
                data: data,
                state: state
            )
        }
    }

    public static let families = ["small", "medium", "large", "accessory"]

    /// Non-negative integer `at` values only (NSNumber bools are not numbers here).
    static func entries(_ config: [String: Any]) -> [(UInt64, [String: Any])] {
        guard let list = config["entries"] as? [Any] else { return [] }
        return list.compactMap { item in
            guard let e = item as? [String: Any], let at = uint(e["at"]) else { return nil }
            return (at, e)
        }
    }

    static func uint(_ v: Any?) -> UInt64? {
        guard let n = v as? NSNumber, !isBool(n) else { return nil }
        let d = n.doubleValue
        guard d >= 0, d.rounded() == d, d < 1.8e19 else { return nil }
        return n.uint64Value
    }

    static func isBool(_ n: NSNumber) -> Bool {
        CFGetTypeID(n) == CFBooleanGetTypeID()
    }

    static func isFloat(_ n: NSNumber) -> Bool {
        let t = String(cString: n.objCType)
        return t == "d" || t == "f"
    }

    static func present(_ v: Any?) -> Bool {
        guard let v else { return false }
        return !(v is NSNull)
    }

    public static func activeLayouts(_ config: [String: Any], nowMs: UInt64, families: [String] = families) -> [String: Any] {
        var out: [String: Any] = [:]
        for f in families where present(config[f]) { out[f] = config[f] }
        var active: [String: Any]?
        var activeAt: UInt64 = 0
        for (at, e) in entries(config) where at <= nowMs && (active == nil || at >= activeAt) {
            active = e
            activeAt = at
        }
        if let active {
            for f in families where present(active[f]) { out[f] = active[f] }
        }
        return out
    }

    public static func nextChangeMs(_ config: [String: Any], nowMs: UInt64) -> UInt64? {
        entries(config).map(\.0).filter { $0 > nowMs }.min()
    }

    /// Entry start times after now, ascending, unique, at most `limit`.
    public static func upcomingChanges(_ config: [String: Any], nowMs: UInt64, limit: Int) -> [UInt64] {
        Array(Set(entries(config).map(\.0).filter { $0 > nowMs }).sorted().prefix(limit))
    }

    public static func mergeData(_ layers: [Any?]) -> Any? {
        var out: Any?
        for layer in layers {
            guard let layer, present(layer) else { continue }
            if var acc = out as? [String: Any], let m = layer as? [String: Any] {
                for (k, v) in m { acc[k] = v }
                out = acc
            } else {
                out = layer
            }
        }
        return out
    }

    /// Lower case, `_` -> `-`, Unicode / private-use extensions (`-u-...`, `-x-...`) dropped.
    static func normalize(_ tag: String) -> String {
        let t = tag.trimmingCharacters(in: .whitespacesAndNewlines).replacingOccurrences(of: "_", with: "-").lowercased()
        let cut = ["-u-", "-x-"].compactMap { t.range(of: $0)?.lowerBound }.min()
        return cut.map { String(t[..<$0]) } ?? t
    }

    /// `zh-hant-tw` -> `zh-tw`: the tag without a 4-letter script subtag.
    static func languageRegion(_ tag: String) -> String {
        tag.split(separator: "-", omittingEmptySubsequences: false).enumerated()
            .filter { !($0.offset == 1 && $0.element.count == 4 && $0.element.allSatisfy { $0.isASCII && $0.isLetter }) }
            .map { String($0.element) }
            .joined(separator: "-")
    }

    static func language(_ tag: String) -> String {
        String(tag.split(separator: "-", omittingEmptySubsequences: false).first ?? "")
    }

    public static func pickLocaleTable(_ config: [String: Any], locale: String) -> String? {
        guard let tables = config["strings"] as? [String: Any], !tables.isEmpty else { return nil }
        let keys = tables.keys.sorted()
        func find(_ pred: (String) -> Bool) -> String? { keys.first { pred(normalize($0)) } }
        let want = normalize(locale)
        if !want.isEmpty {
            let lang = language(want)
            let lr = languageRegion(want)
            if let k = find({ $0 == want }) ?? find({ languageRegion($0) == lr }) ?? find({ $0 == lang }) ?? find({ language($0) == lang }) {
                return k
            }
        }
        if let def = config["defaultLocale"] as? String, let k = find({ $0 == normalize(def) }) {
            return k
        }
        return find({ $0 == "en" }) ?? keys.first
    }

    static func lookup(_ root: Any, _ path: String) -> Any? {
        if path.isEmpty { return root }
        var cur: Any? = root
        for seg in path.split(separator: ".", omittingEmptySubsequences: false) {
            if let a = cur as? [Any] {
                guard seg.allSatisfy(\.isASCII), seg.allSatisfy(\.isNumber), let i = Int(seg), i < a.count else {
                    return nil
                }
                cur = a[i]
            } else if let m = cur as? [String: Any] {
                cur = m[String(seg)]
            } else {
                return nil
            }
        }
        return cur
    }

    static func text(_ v: Any?) -> String {
        guard let v, present(v) else { return "" }
        if let s = v as? String { return s }
        if let n = v as? NSNumber {
            if isBool(n) { return n.boolValue ? "true" : "false" }
            // Integers stay integers; whole floats print as integers (JS) up to 1e21; other
            // floats as the shortest round-trip decimal without exponent.
            if !isFloat(n) { return n.stringValue }
            let d = n.doubleValue
            if d.rounded() == d, abs(d) < 1e21 { return String(format: "%.0f", d + 0) }
            // Like JS: exponent outside 1e-6 <= |x| < 1e21 (Swift already prints `4e-323`).
            if d != 0, abs(d) < 1e-6 || abs(d) >= 1e21 {
                // Swift writes `1e-07`; JavaScript writes `1e-7`.
                return String(d).replacingOccurrences(of: "e-0", with: "e-").replacingOccurrences(of: "e+0", with: "e+")
            }
            return Decimal(string: String(d)).map { "\($0)" } ?? String(d)
        }
        if let data = try? JSONSerialization.data(withJSONObject: v, options: [.sortedKeys, .fragmentsAllowed, .withoutEscapingSlashes]),
           let s = String(data: data, encoding: .utf8) {
            return s
        }
        return ""
    }

    /// Renderer contract version: a config with `requires.resolve` above it shows `fallback`.
    public static let contractVersion = 3
    static let maxRepeat = 50
    static let maxTemplateDepth = 8

    struct Scope {
        let strings: [String: Any]?
        let fallback: [String: Any]?
        let ctx: Context
        var templates: [String: Any]? = nil
        /// Loop / template variables, innermost last.
        var locals: [(String, Any)] = []
        var index: Int? = nil
        var depth = 0

        func child(_ name: String, _ value: Any, index: Int? = nil) -> Scope {
            var s = self
            s.locals.append((name, value))
            if let index { s.index = index }
            return s
        }

        func local(_ name: String) -> (found: Bool, value: Any?) {
            for (n, v) in locals.reversed() where n == name { return (true, v) }
            return (false, nil)
        }

        static func split(_ expr: String) -> (String, String?) {
            guard let dot = expr.firstIndex(of: ".") else { return (expr, nil) }
            return (String(expr[..<dot]), String(expr[expr.index(after: dot)...]))
        }

        /// Typed value of an expression; locals shadow the built-in roots.
        func lookupExpr(_ raw: String) -> Any? {
            let expr = raw.trimmingCharacters(in: .whitespacesAndNewlines)
            let (head, rest) = Scope.split(expr)
            let l = local(head)
            if l.found { return l.value.flatMap { lookup($0, rest ?? "") } }
            switch expr {
            case "index": return index
            case "locale": return ctx.locale
            case "instance": return ctx.instance ?? ""
            default: break
            }
            if expr.hasPrefix("t.") {
                let key = String(expr.dropFirst(2))
                return strings?[key] ?? fallback?[key]
            }
            for (root, value) in [("data", ctx.data), ("state", ctx.state)] where Scope.isRoot(expr, root) {
                guard let d = value, present(d) else { return nil }
                return lookup(d, String(expr.dropFirst(root.count + 1)))
            }
            return nil
        }

        /// `data` / `data.x` (not `database`).
        static func isRoot(_ expr: String, _ root: String) -> Bool {
            expr == root || expr.hasPrefix(root + ".")
        }

        func knownRoot(_ expr: String) -> Bool {
            let (head, _) = Scope.split(expr)
            return local(head).found || expr == "locale" || expr == "instance" || Scope.isRoot(expr, "data")
                || Scope.isRoot(expr, "state") || (expr == "index" && index != nil)
        }

        func placeholder(_ raw: String) -> String {
            let expr = raw.trimmingCharacters(in: .whitespacesAndNewlines)
            let (head, _) = Scope.split(expr)
            if !local(head).found, expr.hasPrefix("t.") {
                let key = String(expr.dropFirst(2))
                if let v = strings?[key] ?? fallback?[key] { return TauriResolve.text(v) }
                return key
            }
            if !knownRoot(expr) { return "{{" + expr + "}}" }
            return TauriResolve.text(lookupExpr(expr))
        }

        func interpolate(_ s: String) -> String {
            var out = ""
            var rest = Substring(s)
            while let start = rest.range(of: "{{") {
                guard let end = rest[start.upperBound...].range(of: "}}") else { break }
                out += rest[..<start.lowerBound]
                out += placeholder(String(rest[start.upperBound..<end.lowerBound]))
                rest = rest[end.upperBound...]
            }
            return out + rest
        }

        static func number(_ v: Any?) -> Double? {
            guard let n = v as? NSNumber, !isBool(n) else { return nil }
            return n.doubleValue
        }

        static func truthy(_ v: Any?) -> Bool {
            guard let v, present(v) else { return false }
            if let n = v as? NSNumber {
                if isBool(n) { return n.boolValue }
                return n.doubleValue != 0
            }
            if let s = v as? String { return !s.isEmpty }
            if let a = v as? [Any] { return !a.isEmpty }
            return true
        }

        /// JSON equality; numbers by value, booleans never equal numbers.
        static func jsonEq(_ a: Any?, _ b: Any?) -> Bool {
            if let x = a as? NSNumber, let y = b as? NSNumber {
                if isBool(x) != isBool(y) { return false }
                if isBool(x) { return x.boolValue == y.boolValue }
                return x.doubleValue == y.doubleValue
            }
            if let x = a as? String, let y = b as? String { return x == y }
            if let x = a as? [Any], let y = b as? [Any] {
                return x.count == y.count && zip(x, y).allSatisfy { jsonEq($0, $1) }
            }
            if let x = a as? [String: Any], let y = b as? [String: Any] {
                return x.count == y.count && x.allSatisfy { k, v in y[k].map { jsonEq(v, $0) } ?? false }
            }
            return !present(a) && !present(b)
        }

        /// Structured condition (see `src/resolve.rs`). Bad shapes are false.
        func evalWhen(_ cond: Any?) -> Bool {
            guard let c = cond as? [String: Any] else { return false }
            if let all = c["all"] as? [Any] { return all.allSatisfy { evalWhen($0) } }
            if let any = c["any"] as? [Any] { return any.contains { evalWhen($0) } }
            if let not = c["not"] { return !evalWhen(not) }
            guard let path = c["path"] as? String else { return false }
            let v = lookupExpr(path)
            let has = v.map { present($0) } ?? false
            let ops = ["eq", "ne", "gt", "gte", "lt", "lte", "exists", "in"].filter { c[$0] != nil }
            if ops.isEmpty { return Scope.truthy(v) }
            return ops.allSatisfy { op in
                let x = c[op]
                switch op {
                case "exists": return has == ((x as? NSNumber).map { isBool($0) && $0.boolValue } ?? false)
                case "eq": return has && Scope.jsonEq(v, x)
                case "ne": return has && !Scope.jsonEq(v, x)
                case "in": return has && ((x as? [Any])?.contains { Scope.jsonEq(v, $0) } ?? false)
                default:
                    guard let a = Scope.number(v), let b = Scope.number(x) else { return false }
                    switch op {
                    case "gt": return a > b
                    case "gte": return a >= b
                    case "lt": return a < b
                    default: return a <= b
                    }
                }
            }
        }

        /// One array element -> zero or more (`repeat` splices, `when` drops, `use` expands).
        func expand(_ el: Any) -> [Any] {
            guard let m = el as? [String: Any] else { return [walk(el)] }
            if let cond = m["when"], !evalWhen(cond) { return [] }
            switch m["type"] as? String {
            case "repeat":
                guard let each = m["each"] as? String, let items = lookupExpr(each) as? [Any],
                      let template = m["template"] as? [String: Any] else { return [] }
                let name = (m["as"] as? String).flatMap { $0.isEmpty ? nil : $0 } ?? "item"
                let limit = uint(m["limit"]).map { Int(min($0, UInt64(TauriResolve.maxRepeat))) } ?? TauriResolve.maxRepeat
                return items.prefix(limit).enumerated().flatMap { i, item in child(name, item, index: i).expand(template) }
            case "use":
                guard let name = m["template"] as? String, let tpl = templates?[name] as? [String: Any],
                      depth < TauriResolve.maxTemplateDepth else { return [] }
                var args: [String: Any] = [:]
                for (k, v) in (m["args"] as? [String: Any]) ?? [:] { args[k] = argValue(v) }
                // Keys on the `use` element override the template root (instance wins).
                var root = tpl
                for (k, v) in m where !["type", "template", "args", "when"].contains(k) { root[k] = v }
                var inner = child("arg", args)
                inner.depth += 1
                return inner.expand(root)
            default:
                return [walk(el)]
            }
        }

        /// An argument that is exactly one `{{expr}}` keeps the expression's JSON type.
        func argValue(_ v: Any) -> Any {
            guard let s = v as? String else { return v }
            if s.hasPrefix("{{"), s.hasSuffix("}}"), s.count >= 4 {
                let inner = String(s.dropFirst(2).dropLast(2))
                if !inner.contains("{{"), !inner.contains("}}") { return lookupExpr(inner) ?? "" }
            }
            return interpolate(s)
        }

        func walk(_ v: Any) -> Any {
            if let s = v as? String { return s.contains("{{") ? interpolate(s) : s }
            if let a = v as? [Any] { return a.flatMap { expand($0) } }
            if let m = v as? [String: Any] {
                var out: [String: Any] = [:]
                for (k, x) in m where k != "bind" && k != "when" { out[k] = walk(x) }
                for (field, expr) in (m["bind"] as? [String: Any]) ?? [:] {
                    if let e = expr as? String, let val = bound(e) { out[field] = val }
                }
                return out
            }
            return v
        }

        /// `bind: {field: expr}`: data / state / loop / arg values keep their JSON type; other
        /// expressions render as text; a missing value keeps the literal.
        func bound(_ raw: String) -> Any? {
            let expr = raw.trimmingCharacters(in: .whitespacesAndNewlines)
            let (head, _) = Scope.split(expr)
            if local(head).found || Scope.isRoot(expr, "data") || Scope.isRoot(expr, "state") || (expr == "index" && index != nil) {
                guard let v = lookupExpr(expr), present(v) else { return nil }
                return v
            }
            return placeholder(expr)
        }

        /// A family root: `use` / `repeat` / `when` apply there too (a dropped root is omitted).
        func walkRoot(_ v: Any) -> Any? {
            if let m = v as? [String: Any], m["type"] as? String == "repeat" {
                return ["type": "vstack", "children": expand(v)] as [String: Any]
            }
            return expand(v).first
        }
    }

    static func scope(_ config: [String: Any], _ ctx: Context) -> Scope {
        let tables = config["strings"] as? [String: Any]
        let t = pickLocaleTable(config, locale: ctx.locale)
        let d = pickLocaleTable(config, locale: "")
        var sc = Scope(
            strings: t.flatMap { tables?[$0] as? [String: Any] },
            fallback: d.flatMap { tables?[$0] as? [String: Any] },
            ctx: ctx
        )
        sc.templates = config["templates"] as? [String: Any]
        return sc
    }

    public static func interpolate(_ config: [String: Any], _ ctx: Context, _ s: String) -> String {
        scope(config, ctx).interpolate(s)
    }

    /// `requires.resolve` above `contractVersion`: `fallback` (or a built-in notice).
    static func effectiveConfig(_ config: [String: Any]) -> [String: Any] {
        guard let need = uint((config["requires"] as? [String: Any])?["resolve"]),
              need > UInt64(contractVersion) else { return config }
        var out = (config["fallback"] as? [String: Any])
            ?? ["small": ["type": "text", "content": "Update the app to show this widget"] as [String: Any]]
        for k in ["version", "strings", "defaultLocale"] { if let v = config[k] { out[k] = v } }
        return out
    }

    /// Active entry, then templates / repeat / when / placeholders / bind; output has
    /// `version` and the size families only.
    public static func resolve(_ config: [String: Any], _ ctx: Context) -> [String: Any] {
        let cfg = effectiveConfig(config)
        let sc = scope(cfg, ctx)
        var out: [String: Any] = ["version": present(cfg["version"]) ? cfg["version"]! : 1]
        for (f, v) in activeLayouts(cfg, nowMs: ctx.nowMs) {
            if let r = sc.walkRoot(v) { out[f] = r }
        }
        return out
    }

    /// Apple-only key outside the shared resolve output: the `transition` hint.
    static func appleExtras(_ config: [String: Any], _ ctx: Context, into out: inout [String: Any]) {
        if let t = effectiveConfig(config)["transition"] as? String { out["transition"] = t }
    }

    /// Resolve raw config JSON and decode it.
    public static func decode(_ raw: String, _ ctx: Context) -> WidgetUIConfig? {
        guard let data = raw.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return nil }
        var resolved = resolve(obj, ctx)
        appleExtras(obj, ctx, into: &resolved)
        guard let out = try? JSONSerialization.data(withJSONObject: resolved) else { return nil }
        return try? JSONDecoder().decode(WidgetUIConfig.self, from: out)
    }

    /// Live Activity template (contract v3 section 3): every string of the stored layout fields is
    /// resolved with `data`; fields present on the content itself win over the template's.
    public static func applyActivityTemplate(_ content: [String: Any], template: [String: Any]) -> [String: Any] {
        let sc = scope([:], Context(nowMs: UInt64(Date().timeIntervalSince1970 * 1000),
                                    locale: Locale.current.identifier(.bcp47), data: content["data"]))
        var out: [String: Any] = [:]
        for (k, v) in template {
            // Element slots go through `walkRoot` (a root `when` / `repeat` / `use` applies).
            let r: Any? = (v as? [String: Any])?["type"] is String ? sc.walkRoot(v) : sc.walk(v)
            if let r { out[k] = r }
        }
        for (k, v) in content where k != "template" && k != "data" && present(v) {
            // A templated request may take its title from the template (as in Rust).
            if k == "title", (v as? String)?.trimmingCharacters(in: .whitespaces).isEmpty == true { continue }
            out[k] = v
        }
        return out
    }

    public struct SourceRequest: Equatable {
        public var url: String
        public var method: String
        public var headers: [String: String]
        public var body: String?
        public var refreshMinutes: Int
    }

    public static let defaultRefreshMinutes = 30
    public static let maxSourceBytes = 256 * 1024

    /// `https://` anywhere; `http://` only to exactly `localhost` / `127.0.0.1`.
    public static func sourceURLAllowed(_ url: String) -> Bool {
        if url.hasPrefix("https://") { return true }
        guard url.hasPrefix("http://") else { return false }
        let authority = url.dropFirst(7).prefix { $0 != "/" && $0 != "?" && $0 != "#" }
        if authority.contains("@") { return false }
        let parts = authority.split(separator: ":", maxSplits: 1, omittingEmptySubsequences: false)
        let host = parts.first.map(String.init) ?? ""
        let port = parts.count > 1 ? String(parts[1]) : ""
        return (host.lowercased() == "localhost" || host == "127.0.0.1") && port.allSatisfy { $0.isASCII && $0.isNumber }
    }

    public static func sourceRequest(_ config: [String: Any], _ ctx: Context) -> SourceRequest? {
        guard let src = config["source"] as? [String: Any], let rawUrl = src["url"] as? String else { return nil }
        let url = interpolate(config, ctx, rawUrl)
        guard sourceURLAllowed(url) else { return nil }
        let method = (src["method"] as? String)?.lowercased() == "post" ? "POST" : "GET"
        var headers: [String: String] = [:]
        for (k, v) in (src["headers"] as? [String: Any]) ?? [:] {
            if let s = v as? String { headers[k] = interpolate(config, ctx, s) }
        }
        let body = method == "POST" ? (src["body"] as? String).map { interpolate(config, ctx, $0) } : nil
        let minutes = uint(src["refreshMinutes"]).map { Int(min(1440, max(15, $0))) } ?? defaultRefreshMinutes
        return SourceRequest(url: url, method: method, headers: headers, body: body, refreshMinutes: minutes)
    }
}
