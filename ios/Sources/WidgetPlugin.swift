import Tauri
import UIKit
import WebKit

#if canImport(ActivityKit)
import ActivityKit
#endif

#if canImport(WidgetKit)
import WidgetKit
#endif

struct SetItemsArgs: Decodable {
    let key: String
    let value: String
    let group: String
}

struct GetItemsArgs: Decodable {
    let key: String
    let group: String
}

struct SetRegisterWidgetArgs: Decodable {
    let widgets: [String]
}

struct ReloadTimelinesArgs: Decodable {
    let ofKind: String
}

struct SetWidgetConfigArgs: Decodable {
    let config: String
    let group: String
    let widgetId: String
}

struct GetWidgetConfigArgs: Decodable {
    let group: String
    let widgetId: String
}

struct GroupArgs: Decodable {
    let group: String
}

struct ActivityArgs: Decodable {
    let content: JSObjectBox
    let group: String
}

struct EndActivityArgs: Decodable {
    let id: String
    let dismissAfterMs: Double?
    /// Final content shown until the activity is dismissed.
    let content: JSObjectBox?
    /// `default` (system, up to 4 h) | `immediate`; `dismissAfterMs` wins when set.
    let dismissalPolicy: String?
}

/// `setWidgetState` / `getWidgetState`: `patch` (null clears the stored layer) or a full `state`.
struct StateArgs: Decodable {
    let group: String
    let widgetId: String
    let instance: String?
    let patch: JSONValue?
    let state: JSONValue?
}

/// Keeps an arbitrary JSON object as compact JSON text.
/// Drops out-of-order activity updates: an update whose `timestamp` is older than the last
/// applied one for that id is rejected; no timestamp always applies; `forget` (end) resets.
struct ActivityTimestampGuard {
    private var last: [String: Double] = [:]

    mutating func accept(id: String, timestamp: Double?) -> Bool {
        guard let ts = timestamp else { return true }
        if let prev = last[id], ts < prev { return false }
        last[id] = ts
        return true
    }

    mutating func forget(_ id: String) { last[id] = nil }
}

struct JSObjectBox: Decodable {
    let json: String
    let id: String
    let staleAt: Double?
    let push: Bool
    /// Which activity the system shows first when several run (ActivityKit `relevanceScore`).
    let relevanceScore: Double?
    /// Unix ms the content was produced: an older update than the last one applied is dropped.
    let timestamp: Double?
    /// `alert: {title, body, sound?}` lights the screen / plays a sound on update.
    let alert: (title: String, body: String, sound: String?)?

    init(from decoder: Decoder) throws {
        let value = try decoder.singleValueContainer().decode(JSONValue.self)
        let data = try JSONEncoder().encode(value)
        json = String(decoding: data, as: UTF8.self)
        let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] ?? [:]
        id = obj["id"] as? String ?? ""
        staleAt = (obj["staleAt"] as? NSNumber)?.doubleValue
        push = obj["push"] as? Bool ?? false
        relevanceScore = (obj["relevanceScore"] as? NSNumber)?.doubleValue
        timestamp = (obj["timestamp"] as? NSNumber)?.doubleValue
        alert = (obj["alert"] as? [String: Any]).flatMap { a in
            (a["title"] as? String).map { ($0, a["body"] as? String ?? "", a["sound"] as? String) }
        }
    }
}

/// `activity-content` event payload.
struct ActivityContentEvent: Encodable {
    let id: String
    let content: JSONValue
}

/// Minimal JSON tree for re-encoding plugin args.
enum JSONValue: Codable {
    case null, bool(Bool), number(Double), string(String), array([JSONValue]), object([String: JSONValue])

    /// As a Foundation object (`JSONSerialization` types), `nil` for non-objects.
    var dictionary: [String: Any]? {
        guard case .object = self, let d = try? JSONEncoder().encode(self) else { return nil }
        return (try? JSONSerialization.jsonObject(with: d)) as? [String: Any]
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() { self = .null }
        else if let b = try? c.decode(Bool.self) { self = .bool(b) }
        else if let n = try? c.decode(Double.self) { self = .number(n) }
        else if let s = try? c.decode(String.self) { self = .string(s) }
        else if let a = try? c.decode([JSONValue].self) { self = .array(a) }
        else { self = .object(try c.decode([String: JSONValue].self)) }
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .null: try c.encodeNil()
        case .bool(let b): try c.encode(b)
        case .number(let n):
            if n.rounded() == n, abs(n) < 9e15 { try c.encode(Int64(n)) } else { try c.encode(n) }
        case .string(let s): try c.encode(s)
        case .array(let a): try c.encode(a)
        case .object(let o): try c.encode(o)
        }
    }
}

#if canImport(ActivityKit)
/// Must match `TauriActivityAttributes` in the TauriWidgets Swift package (ActivityKit pairs
/// the app's request with the extension's `ActivityConfiguration` by this type).
@available(iOS 16.2, *)
struct TauriActivityAttributes: ActivityAttributes {
    struct ContentState: Codable, Hashable {
        /// The full `ActivityContent` request as JSON (layout already filled by Rust).
        var json: String
    }
    var id: String
    var group: String
}
#endif

class WidgetPlugin: Plugin {
    private var registeredWidgets: [String] = []
    // `#` separates widget id and placement id (`config:weather#nyc`).
    private let safeRegex = try! NSRegularExpression(pattern: "[^A-Za-z0-9._:#-]", options: [])
    private let groupSafeRegex = try! NSRegularExpression(pattern: "[^A-Za-z0-9._-]", options: [])

    private let configPrefix = "config:"
    private let pendingActionsKey = "pending_actions"
    private let metaNonceKey = "__meta_nonce__"
    private let metaUpdatedAtKey = "__meta_updated_at__"

    private func sanitize(_ value: String, fallback: String = "_") -> String {
        let range = NSRange(location: 0, length: value.utf16.count)
        let cleaned = safeRegex.stringByReplacingMatches(in: value, options: [], range: range, withTemplate: "_")
        let trimmed = cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? fallback : trimmed
    }

    private func sanitizeGroup(_ value: String, fallback: String = "_") -> String {
        let range = NSRange(location: 0, length: value.utf16.count)
        let cleaned = groupSafeRegex.stringByReplacingMatches(in: value, options: [], range: range, withTemplate: "_")
        let trimmed = cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? fallback : trimmed
    }

    /// App Group must be explicit `group.*` — no silent substitution.
    private func resolveAppGroup(_ group: String) throws -> String {
        let clean = sanitizeGroup(group, fallback: "")
        guard clean.hasPrefix("group.") else {
            throw NSError(
                domain: "tauri-plugin-widgets",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "group must be an App Group id starting with 'group.'"]
            )
        }
        return clean
    }

    private func dataFileURL(group: String) throws -> URL {
        let appGroup = try resolveAppGroup(group)
        guard let url = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroup
        )?.appendingPathComponent("widget_data.json") else {
            throw NSError(
                domain: "tauri-plugin-widgets",
                code: 2,
                userInfo: [NSLocalizedDescriptionKey: "App Group container unavailable for \(appGroup). Enable App Groups."]
            )
        }
        return url
    }

    /// Exclusive `flock` on `widget_data.json.lock`, shared with the widget extension
    /// (`TauriWidgetDataStore.withStoreLock`): every read-modify-write of the map runs
    /// under it, so a widget tap is never wiped by a host write or drained twice.
    private func withStoreLock<T>(group: String, _ body: () throws -> T) throws -> T {
        let url = try dataFileURL(group: group)
        let fd = open(url.path + ".lock", O_RDWR | O_CREAT, 0o644)
        guard fd >= 0 else { return try body() }
        defer { close(fd) }
        flock(fd, LOCK_EX)
        defer { flock(fd, LOCK_UN) }
        return try body()
    }

    private func readDataMap(group: String) throws -> [String: String] {
        let url = try dataFileURL(group: group)
        guard let data = try? Data(contentsOf: url),
              let map = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return [:]
        }
        return map
    }

    private func touchMeta(_ map: inout [String: String]) {
        let next = (UInt64(map[metaNonceKey] ?? "0") ?? 0) &+ 1
        map[metaNonceKey] = String(next)
        map[metaUpdatedAtKey] = String(UInt64(Date().timeIntervalSince1970 * 1000))
    }

    private func writeDataMap(_ map: [String: String], group: String) throws {
        let url = try dataFileURL(group: group)
        let data = try JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: url, options: .atomic)

        // Also mirror into App Group UserDefaults when suite is available.
        let appGroup = try resolveAppGroup(group)
        if let defaults = UserDefaults(suiteName: appGroup) {
            defaults.set(map, forKey: "widget_data")
            defaults.synchronize()
        }
    }

    private func configKey(_ widgetId: String) -> String {
        configPrefix + widgetId
    }

    @objc public override func load(webview: WKWebView) {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            // Tokens rotate for activities started in an earlier launch too.
            Activity<TauriActivityAttributes>.activities
                .filter { $0.activityState == .active || $0.activityState == .stale }
                .forEach(observe)
            // Activities your server starts with push-to-start appear here too.
            Task { [weak self] in
                for await activity in Activity<TauriActivityAttributes>.activityUpdates {
                    self?.observe(activity)
                }
            }
        }
        if #available(iOS 17.2, *) {
            Task { [weak self] in
                for await token in Activity<TauriActivityAttributes>.pushToStartTokenUpdates {
                    let hex = Self.hex(token) ?? ""
                    self?.storePushToStartToken(hex)
                    self?.trigger("activity-push-to-start-token", data: ["pushToStartToken": hex])
                }
            }
        }
        #endif
    }

    private func storePushToStartToken(_ hex: String) {
        tokenLock.lock()
        pushToStartToken = hex
        tokenLock.unlock()
    }

    /// Latest push-to-start token (iOS 17.2+): your server starts a Live Activity with it.
    @objc func getPushToStartToken(_ invoke: Invoke) throws {
        tokenLock.lock()
        let token = pushToStartToken
        tokenLock.unlock()
        #if canImport(ActivityKit)
        if token == nil, #available(iOS 17.2, *), let data = Activity<TauriActivityAttributes>.pushToStartToken {
            invoke.resolve(["token": Self.hex(data) as Any])
            return
        }
        #endif
        invoke.resolve(["token": token as Any])
    }

    @objc func setItems(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetItemsArgs.self)
        let safeKey = sanitize(args.key)
        try withStoreLock(group: args.group) {
            var map = try readDataMap(group: args.group)
            map[safeKey] = args.value
            touchMeta(&map)
            try writeDataMap(map, group: args.group)
        }
        invoke.resolve(["results": true])
    }

    @objc func getItems(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GetItemsArgs.self)
        let safeKey = sanitize(args.key)
        let map = try readDataMap(group: args.group)
        if let value = map[safeKey] {
            invoke.resolve(["results": value])
        } else {
            invoke.resolve(["results": NSNull()])
        }
    }

    @objc func setRegisterWidget(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetRegisterWidgetArgs.self)
        guard !args.widgets.isEmpty else {
            invoke.reject("widgets must be a non-empty array")
            return
        }
        registeredWidgets = args.widgets
        invoke.resolve(["results": true])
    }

    @objc func setWidgetConfig(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetWidgetConfigArgs.self)
        guard !args.widgetId.isEmpty else {
            invoke.reject("widgetId must not be empty")
            return
        }
        try withStoreLock(group: args.group) {
            var map = try readDataMap(group: args.group)
            map[configKey(args.widgetId)] = args.config
            touchMeta(&map)
            try writeDataMap(map, group: args.group)
        }
        invoke.resolve(["results": true])
    }

    @objc func getWidgetConfig(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GetWidgetConfigArgs.self)
        guard !args.widgetId.isEmpty else {
            invoke.reject("widgetId must not be empty")
            return
        }
        let map = try readDataMap(group: args.group)
        if let value = map[configKey(args.widgetId)] {
            invoke.resolve(["results": value])
        } else {
            invoke.resolve(["results": NSNull()])
        }
    }

    // MARK: - Local state (contract v3; the extension's `WidgetStateIntent` writes the same keys)

    private func jsonObject(_ raw: String?) -> [String: Any]? {
        guard let raw, let d = raw.data(using: .utf8) else { return nil }
        return (try? JSONSerialization.jsonObject(with: d)) as? [String: Any]
    }

    private func scoped(_ widgetId: String, _ instance: String?) -> String {
        guard let instance, !instance.isEmpty else { return widgetId }
        return widgetId + "#" + instance
    }

    /// `config.state` < `state:{widgetId}` < `state:{widgetId}#{instance}`.
    private func mergedState(_ map: [String: String], _ args: StateArgs) -> [String: Any] {
        var out = (jsonObject(map[configKey(args.widgetId)])?["state"] as? [String: Any]) ?? [:]
        var keys = [sanitize("state:" + args.widgetId)]
        if let i = args.instance, !i.isEmpty { keys.append(sanitize("state:" + scoped(args.widgetId, i))) }
        for k in keys { jsonObject(map[k])?.forEach { out[$0.key] = $0.value } }
        return out
    }

    /// Same operators as a `set` tap: `$toggle`, `$inc`, `$dec`, `null` removes the key.
    static func applyStatePatch(_ patch: [String: Any], current: [String: Any], stored: [String: Any]) -> [String: Any] {
        var out = stored
        for (k, v) in patch {
            let cur = current[k] as? NSNumber
            let isBool = cur.map { CFGetTypeID($0) == CFBooleanGetTypeID() } ?? false
            switch v as? String {
            case "$toggle": out[k] = !(isBool && cur!.boolValue)
            case "$inc", "$dec":
                let r = (isBool ? 0 : cur?.doubleValue ?? 0) + (v as? String == "$inc" ? 1 : -1)
                out[k] = r.rounded() == r && abs(r) < 9e15 ? NSNumber(value: Int64(r)) : NSNumber(value: r)
            default: out[k] = v is NSNull ? nil : v
            }
        }
        return out
    }

    @objc func setWidgetState(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(StateArgs.self)
        guard !args.widgetId.isEmpty else {
            invoke.reject("widgetId must not be empty")
            return
        }
        let state: [String: Any] = try withStoreLock(group: args.group) {
            var map = try readDataMap(group: args.group)
            let key = sanitize("state:" + scoped(args.widgetId, args.instance))
            let next: [String: Any]?
            if let full = args.state?.dictionary {
                next = full
            } else if let patch = args.patch?.dictionary {
                next = Self.applyStatePatch(patch, current: mergedState(map, args), stored: jsonObject(map[key]) ?? [:])
            } else {
                next = nil // `patch: null`: clear the placement's layer
            }
            map[key] = try next.map { String(decoding: try JSONSerialization.data(withJSONObject: $0, options: [.sortedKeys]), as: UTF8.self) } ?? ""
            touchMeta(&map)
            try writeDataMap(map, group: args.group)
            return mergedState(map, args)
        }
        #if canImport(WidgetKit)
        if #available(iOS 14.0, macOS 11.0, *) { WidgetCenter.shared.reloadAllTimelines() }
        #endif
        invoke.resolve(["state": state])
    }

    @objc func getWidgetState(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(StateArgs.self)
        invoke.resolve(["state": mergedState(try readDataMap(group: args.group), args)])
    }

    /// iOS 26 widget push token (hex) recorded by the extension's `TauriWidgetPushHandler`.
    @objc func getWidgetPushToken(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GroupArgs.self)
        let token = UserDefaults(suiteName: try resolveAppGroup(args.group))?.string(forKey: "tauri_widget_push_token")
        invoke.resolve(["token": token as Any])
    }

    @objc func pollPendingActions(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GroupArgs.self)
        // Take the whole queue under the store lock: the extension appends under it too.
        let claimed: [[String: Any]] = try withStoreLock(group: args.group) {
            var map = try readDataMap(group: args.group)
            let raw = map[pendingActionsKey] ?? "[]"
            guard let data = raw.data(using: .utf8),
                  let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]],
                  !arr.isEmpty else { return [] }
            map[pendingActionsKey] = "[]"
            touchMeta(&map)
            try writeDataMap(map, group: args.group)
            return arr
        }
        invoke.resolve(["results": claimed])
    }

    @objc func reloadAllTimelines(_ invoke: Invoke) throws {
        #if canImport(WidgetKit)
        if #available(iOS 14.0, macOS 11.0, *) {
            WidgetCenter.shared.reloadAllTimelines()
            invoke.resolve(["results": true])
        } else {
            invoke.reject("WidgetKit requires iOS 14.0+ / macOS 11.0+")
        }
        #else
        invoke.reject("WidgetKit is not available on this platform")
        #endif
    }

    @objc func reloadTimelines(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(ReloadTimelinesArgs.self)

        #if canImport(WidgetKit)
        if #available(iOS 14.0, macOS 11.0, *) {
            WidgetCenter.shared.reloadTimelines(ofKind: args.ofKind)
            invoke.resolve(["results": true])
        } else {
            invoke.reject("WidgetKit requires iOS 14.0+ / macOS 11.0+")
        }
        #else
        invoke.reject("WidgetKit is not available on this platform")
        #endif
    }

    /// Placed widgets from WidgetCenter. The Edit Widget choice of an intent widget is
    /// only known to the extension, so instances come from its render receipts (Rust).
    @objc func getActiveWidgets(_ invoke: Invoke) throws {
        // WidgetCenter knows kinds; the extension's providers record kind -> widgetId
        // (`TauriWidgetsConfig.kindsKey`), which receipts and configs use.
        let group = (try? invoke.parseArgs(GroupArgs.self)).flatMap { try? resolveAppGroup($0.group) }
        let kinds = group.flatMap { UserDefaults(suiteName: $0)?.dictionary(forKey: "tauri_widget_kinds") as? [String: String] } ?? [:]
        #if canImport(WidgetKit)
        guard #available(iOS 14.0, macOS 11.0, *) else {
            invoke.resolve(["widgets": [[String: Any]](), "exactInstances": true])
            return
        }
        WidgetCenter.shared.getCurrentConfigurations { result in
            switch result {
            case .success(let infos):
                let widgets: [[String: Any]] = infos.map { info in
                    let size: String
                    switch info.family {
                    case .systemSmall: size = "small"
                    case .systemLarge, .systemExtraLarge: size = "large"
                    default: size = "medium"
                    }
                    return ["widgetId": kinds[info.kind] ?? info.kind, "size": size]
                }
                invoke.resolve(["widgets": widgets, "exactInstances": false])
            case .failure(let error):
                invoke.reject(error.localizedDescription)
            }
        }
        #else
        invoke.resolve(["widgets": [], "exactInstances": true])
        #endif
    }

    @objc func startActivity(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(ActivityArgs.self)
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *) else {
            invoke.reject("Live Activities need iOS 16.2+")
            return
        }
        guard ActivityAuthorizationInfo().areActivitiesEnabled else {
            invoke.reject("Live Activities are turned off for this app (Settings) or NSSupportsLiveActivities is missing in Info.plist")
            return
        }
        if let running = runningActivity(args.content.id) {
            if args.content.push { observe(running) }
            Task {
                let applied = await self.apply(args.content, to: running)
                invoke.resolve(["id": args.content.id, "surface": "ios", "pushToken": Self.hex(running.pushToken) as Any, "applied": applied])
            }
            return
        }
        do {
            let activity = try Activity.request(
                attributes: TauriActivityAttributes(id: args.content.id, group: args.group),
                content: Self.content(args.content),
                pushType: args.content.push ? .token : nil
            )
            _ = remember(args.content)
            observe(activity)
            invoke.resolve(["id": args.content.id, "surface": "ios", "pushToken": Self.hex(activity.pushToken) as Any])
        } catch {
            invoke.reject("Activity.request failed: \(error.localizedDescription)")
        }
        #else
        invoke.reject("ActivityKit is not available")
        #endif
    }

    @objc func updateActivity(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(ActivityArgs.self)
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *),
              let running = runningActivity(args.content.id) else {
            invoke.reject("activity \"\(args.content.id)\" is not running")
            return
        }
        Task {
            let applied = await self.apply(args.content, to: running)
            invoke.resolve(["results": true, "applied": applied])
        }
        #else
        invoke.reject("ActivityKit is not available")
        #endif
    }

    @objc func endActivity(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(EndActivityArgs.self)
        #if canImport(ActivityKit)
        guard #available(iOS 16.2, *),
              let running = runningActivity(args.id) else {
            invoke.resolve(["ended": false])
            return
        }
        let policy = Self.dismissalPolicy(afterMs: args.dismissAfterMs, named: args.dismissalPolicy)
        // Final content stays on the Lock Screen until dismissed.
        let final = args.content.map(Self.content)
        forget(args.id)
        Task {
            await running.end(final, dismissalPolicy: policy)
            invoke.resolve(["ended": true])
        }
        #else
        invoke.resolve(["ended": false])
        #endif
    }

    #if canImport(ActivityKit)
    /// `dismissAfterMs` > 0 → after that delay; else `dismissalPolicy` (`default` | `immediate`,
    /// default `immediate` as before).
    @available(iOS 16.2, *)
    static func dismissalPolicy(afterMs: Double?, named: String?) -> ActivityUIDismissalPolicy {
        if let ms = afterMs, ms > 0 { return .after(Date().addingTimeInterval(ms / 1000)) }
        return named == "default" ? .default : .immediate
    }

    @available(iOS 16.2, *)
    static func content(_ box: JSObjectBox) -> ActivityContent<TauriActivityAttributes.ContentState> {
        ActivityContent(
            state: .init(json: box.json),
            staleDate: box.staleAt.map { Date(timeIntervalSince1970: $0 / 1000) },
            relevanceScore: box.relevanceScore ?? 0
        )
    }

    /// Records `timestamp`; false when it is older than the last applied one (out-of-order
    /// delivery must not roll the activity back).
    private func remember(_ box: JSObjectBox) -> Bool {
        tokenLock.lock()
        defer { tokenLock.unlock() }
        return timestamps.accept(id: box.id, timestamp: box.timestamp)
    }

    private func forget(_ id: String) {
        tokenLock.lock()
        timestamps.forget(id)
        tokenLock.unlock()
    }

    /// Update with relevance, alert and timestamp; returns false when dropped as stale.
    @available(iOS 16.2, *)
    private func apply(_ box: JSObjectBox, to activity: Activity<TauriActivityAttributes>) async -> Bool {
        guard remember(box) else { return false }
        let alert = box.alert.map {
            AlertConfiguration(
                title: LocalizedStringResource(stringLiteral: $0.title),
                body: LocalizedStringResource(stringLiteral: $0.body),
                sound: $0.sound.map { $0 == "default" ? .default : .named($0) } ?? .default
            )
        }
        if #available(iOS 17.2, *), let ts = box.timestamp {
            await activity.update(Self.content(box), alertConfiguration: alert, timestamp: Date(timeIntervalSince1970: ts / 1000))
        } else {
            await activity.update(Self.content(box), alertConfiguration: alert)
        }
        return true
    }
    #endif

    @objc func listActivities(_ invoke: Invoke) throws {
        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            let ids = Activity<TauriActivityAttributes>.activities
                .filter { $0.activityState == .active || $0.activityState == .stale }
                .map(\.attributes.id)
            invoke.resolve(["ids": ids])
            return
        }
        #endif
        invoke.resolve(["ids": [String]()])
    }

    #if canImport(ActivityKit)
    /// Ended activities stay in `activities` until dismissed; only live ones count.
    @available(iOS 16.2, *)
    private func runningActivity(_ id: String) -> Activity<TauriActivityAttributes>? {
        Activity<TauriActivityAttributes>.activities.first {
            $0.attributes.id == id && ($0.activityState == .active || $0.activityState == .stale)
        }
    }
    private var pushToStartToken: String?

    private let tokenLock = NSLock()
    /// ActivityKit `Activity.id`s with observers.
    private var tokenObserved = Set<String>()
    /// Last applied `timestamp` per request id.
    private var timestamps = ActivityTimestampGuard()

    /// One set of observers per activity (started here, earlier, or by push-to-start):
    /// `activity-token` on every (re)issued push token, `activity-state`
    /// `{id, state: active|stale|ended|dismissed|pending}` and `activity-content`
    /// `{id, content}` when the content changes (also from your server's pushes).
    @available(iOS 16.2, *)
    private func observe(_ activity: Activity<TauriActivityAttributes>) {
        tokenLock.lock()
        let fresh = tokenObserved.insert(activity.id).inserted
        tokenLock.unlock()
        guard fresh else { return }
        let id = activity.attributes.id
        Task { [weak self] in
            for await token in activity.pushTokenUpdates {
                self?.trigger("activity-token", data: ["id": id, "pushToken": Self.hex(token) ?? ""])
            }
        }
        Task { [weak self] in
            for await state in activity.activityStateUpdates {
                self?.trigger("activity-state", data: ["id": id, "state": "\(state)"])
                if state == .ended || state == .dismissed { self?.forget(id) }
            }
        }
        Task { [weak self] in
            for await content in activity.contentUpdates {
                let json = (try? JSONDecoder().decode(JSONValue.self, from: Data(content.state.json.utf8))) ?? .null
                try? self?.trigger("activity-content", data: ActivityContentEvent(id: id, content: json))
            }
        }
    }
    #endif

    private static func hex(_ data: Data?) -> String? {
        data.map { $0.map { String(format: "%02x", $0) }.joined() }
    }

    @objc func requestWidget(_ invoke: Invoke) throws {
        invoke.reject("requestWidget is not supported on iOS/macOS. Users add widgets manually via the widget gallery.")
    }
}

@_cdecl("init_plugin_widgets")
func initPlugin() -> Plugin {
    return WidgetPlugin()
}
