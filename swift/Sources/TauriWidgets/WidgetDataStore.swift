import Foundation
import os.log

private let logger = Logger(subsystem: "com.tauri.widgets", category: "DataStore")

/// Storage contract (0.4+):
/// - `config:{widgetId}` — widget UI JSON
/// - `pending_actions` — JSON array of action envelopes
/// - `__meta_nonce__` / `__meta_updated_at__` — freshness for multi-transport pick
/// - Receipts live in a **sibling** file/key (`widget_receipt.json` / `widget_receipt`) —
///   never inside the config map (must not bump nonce).
public enum TauriWidgetStoreKeys {
    public static let configPrefix = "config:"
    public static let pendingActions = "pending_actions"
    public static let metaNonce = "__meta_nonce__"
    public static let metaUpdatedAt = "__meta_updated_at__"
    public static let receiptDefaultsKey = "widget_receipt"

    public static func configKey(_ widgetId: String) -> String {
        configPrefix + widgetId
    }

    public static let dataPrefix = "data:"
    /// Placement state written by `set` taps / `setWidgetState` (contract v3).
    public static let statePrefix = "state:"
    /// Live Activity layouts from `registerActivityTemplate` (contract v3).
    public static let activityTemplatePrefix = "activity-template:"
    public static let instancesPrefix = "instances:"
    /// Matches Rust `store::INSTANCE_SEPARATOR`.
    public static let instanceSeparator = "#"

    /// `widgetId` or `widgetId#instance`.
    public static func scopedId(_ widgetId: String, _ instance: String?) -> String {
        guard let instance, !instance.isEmpty else { return widgetId }
        return widgetId + instanceSeparator + instance
    }

    /// Placement's own key first, then the widget's (Rust `store::lookup_keys`).
    public static func lookupKeys(_ prefix: String, _ widgetId: String, _ instance: String?) -> [String] {
        var keys: [String] = []
        if let instance, !instance.isEmpty { keys.append(prefix + scopedId(widgetId, instance)) }
        keys.append(prefix + widgetId)
        return keys
    }
}

/// What a provider needs for one placement: raw config JSON plus its data layers.
public struct TauriWidgetSnapshot {
    public let rawConfig: String?
    public let hostData: Any?
    public let source: String
    public let nonce: UInt64
    /// Stored `state:{widgetId}` then `state:{widgetId}#{instance}` (missing ones nil).
    public var storedState: [Any?] = []

    /// `config.data` < `setWidgetData` < fetched `source` response.
    public func data(fetched: Any?) -> Any? {
        let cfgData = rawConfig.flatMap(TauriWidgetSnapshot.object)?["data"]
        return TauriResolve.mergeData([cfgData, hostData, fetched])
    }

    /// `config.state` < `state:{widgetId}` < `state:{widgetId}#{instance}`.
    public var state: Any? {
        TauriResolve.mergeData([rawConfig.flatMap(TauriWidgetSnapshot.object)?["state"]] + storedState)
    }

    static func object(_ raw: String) -> [String: Any]? {
        guard let d = raw.data(using: .utf8) else { return nil }
        return (try? JSONSerialization.jsonObject(with: d)) as? [String: Any]
    }

    static func json(_ raw: String?) -> Any? {
        guard let raw, !raw.isEmpty, let d = raw.data(using: .utf8) else { return nil }
        return try? JSONSerialization.jsonObject(with: d, options: [.fragmentsAllowed])
    }
}

/// Last `source` response per placement. Lives outside the shared config map (the host
/// never reads it), so a fetch can never overwrite a newer config written by the app.
public enum TauriFetchedCache {
    static let key = "tauri_widget_fetched"

    static func defaults(_ appGroup: String) -> UserDefaults {
        UserDefaults(suiteName: appGroup) ?? .standard
    }

    public static func read(appGroup: String, scopedId: String) -> (raw: String, ts: UInt64)? {
        guard let all = defaults(appGroup).dictionary(forKey: key) as? [String: [String: Any]],
              let e = all[scopedId], let raw = e["raw"] as? String else { return nil }
        return (raw, (e["ts"] as? NSNumber)?.uint64Value ?? 0)
    }

    /// Most recent responses kept; older placements refetch on their next timeline.
    static let maxEntries = 32
    // ponytail: in-process lock only; timelines of one extension share a process.
    private static let lock = NSLock()

    public static func write(appGroup: String, scopedId: String, raw: String, ts: UInt64) {
        lock.lock()
        defer { lock.unlock() }
        let d = defaults(appGroup)
        var all = (d.dictionary(forKey: key) as? [String: [String: Any]]) ?? [:]
        all[scopedId] = ["raw": raw, "ts": NSNumber(value: ts)]
        if all.count > maxEntries {
            func ts(_ e: [String: Any]) -> UInt64 { (e["ts"] as? NSNumber)?.uint64Value ?? 0 }
            let keep = all.sorted { ts($0.value) > ts($1.value) }.prefix(maxEntries)
            all = Dictionary(uniqueKeysWithValues: keep.map { ($0.key, $0.value) })
        }
        d.set(all, forKey: key)
    }
}

/// Transport names — must match Rust `transport::NAME_*`.
public enum TauriWidgetTransportName {
    public static let appGroup = "appgroup"
    public static let defaults = "defaults"
    public static let container = "container"
}

public struct WidgetActionEnvelope: Codable {
    public let action: String
    public let payload: String?
    public let ts: UInt64
    public let widgetId: String
    public let group: String

    public init(action: String, payload: String? = nil, ts: UInt64, widgetId: String, group: String) {
        self.action = action
        self.payload = payload
        self.ts = ts
        self.widgetId = widgetId
        self.group = group
    }

    enum CodingKeys: String, CodingKey {
        case action, payload, ts, widgetId, group
    }
}

/// Async delivery / render receipt (outside config nonce space).
public struct WidgetTransportReceipt: Codable {
    /// Transport / channel that supplied the config (`appgroup`|`defaults`|`container`|…).
    public let source: String
    /// Legacy alias kept in-memory for older callers; not encoded (Rust serde rejects duplicate fields).
    public let readFrom: String
    public let widgetId: String
    public let group: String
    public let instance: String
    public let nonce: UInt64
    public let size: String?
    public let theme: String?
    public let schema: UInt32
    public let rendered: [String]
    public let skipped: [SkippedElement]
    public let ts: UInt64
    /// Why this paint ran: `reload` | `timeline` | `action` | `added` | `resize` | `snapshot`.
    public let trigger: String?

    public struct SkippedElement: Codable {
        public let type: String
        public let reason: String
        public init(type: String, reason: String) {
            self.type = type
            self.reason = reason
        }
    }

    enum CodingKeys: String, CodingKey {
        case source, readFrom, widgetId, group, instance, nonce, size, theme, schema, rendered, skipped, ts, trigger
    }

    public init(
        source: String,
        widgetId: String,
        group: String,
        instance: String,
        nonce: UInt64,
        size: String? = nil,
        theme: String? = nil,
        schema: UInt32 = 1,
        rendered: [String] = [],
        skipped: [SkippedElement] = [],
        ts: UInt64 = UInt64(Date().timeIntervalSince1970 * 1000),
        trigger: String? = nil
    ) {
        self.source = source
        self.readFrom = source
        self.widgetId = widgetId
        self.group = group
        self.instance = instance
        self.nonce = nonce
        self.size = size
        self.theme = theme
        self.schema = schema
        self.rendered = rendered
        self.skipped = skipped
        self.ts = ts
        self.trigger = trigger
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let src = try c.decodeIfPresent(String.self, forKey: .source)
            ?? c.decode(String.self, forKey: .readFrom)
        self.source = src
        self.readFrom = try c.decodeIfPresent(String.self, forKey: .readFrom) ?? src
        self.widgetId = try c.decode(String.self, forKey: .widgetId)
        self.group = try c.decode(String.self, forKey: .group)
        self.instance = try c.decode(String.self, forKey: .instance)
        self.nonce = try c.decode(UInt64.self, forKey: .nonce)
        self.size = try c.decodeIfPresent(String.self, forKey: .size)
        self.theme = try c.decodeIfPresent(String.self, forKey: .theme)
        self.schema = try c.decodeIfPresent(UInt32.self, forKey: .schema) ?? 1
        self.rendered = try c.decodeIfPresent([String].self, forKey: .rendered) ?? []
        self.skipped = try c.decodeIfPresent([SkippedElement].self, forKey: .skipped) ?? []
        self.ts = try c.decode(UInt64.self, forKey: .ts)
        self.trigger = try c.decodeIfPresent(String.self, forKey: .trigger)
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        // Emit only `source` — Rust Receipt has `read_from` with serde alias `source`.
        try c.encode(source, forKey: .source)
        try c.encode(widgetId, forKey: .widgetId)
        try c.encode(group, forKey: .group)
        try c.encode(instance, forKey: .instance)
        try c.encode(nonce, forKey: .nonce)
        try c.encodeIfPresent(size, forKey: .size)
        try c.encodeIfPresent(theme, forKey: .theme)
        try c.encode(schema, forKey: .schema)
        try c.encode(rendered, forKey: .rendered)
        try c.encode(skipped, forKey: .skipped)
        try c.encode(ts, forKey: .ts)
        try c.encodeIfPresent(trigger, forKey: .trigger)
    }
}

/// `set` semantics shared by every renderer (contract v3).
public enum TauriWidgetState {
    /// New stored state of one placement: `"$toggle"` flips the current (merged) boolean
    /// (missing → `true`), `"$inc"` / `"$dec"` add / subtract 1 (missing → 0 first), `null`
    /// removes the key (the default shows again), anything else is stored as is.
    public static func apply(_ patch: [String: Any], current: [String: Any], stored: [String: Any]) -> [String: Any] {
        var out = stored
        for (k, v) in patch {
            let cur = current[k]
            switch v as? String {
            case "$toggle":
                out[k] = !((cur as? NSNumber).map { TauriResolve.isBool($0) && $0.boolValue } ?? false)
            case "$inc", "$dec":
                let n = (cur as? NSNumber).flatMap { TauriResolve.isBool($0) ? nil : $0.doubleValue } ?? 0
                let r = n + (v as? String == "$inc" ? 1 : -1)
                out[k] = r.rounded() == r && abs(r) < 9e15 ? NSNumber(value: Int64(r)) : NSNumber(value: r)
            default:
                if v is NSNull { out[k] = nil } else { out[k] = v }
            }
        }
        return out
    }
}

public struct TauriWidgetDataStore {

    /// Load config and plant a delivery receipt (does not bump config nonce).
    public static func loadConfig(appGroup: String, widgetId: String = "default") -> WidgetUIConfig? {
        loadConfigAcknowledging(appGroup: appGroup, widgetId: widgetId)
    }

    public static func loadConfigAcknowledging(appGroup: String, widgetId: String = "default") -> WidgetUIConfig? {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else { return nil }
        #endif
        let (map, source) = loadFreshestMapWithSource(appGroup: appGroup, widgetId: widgetId)
        guard let raw = map[TauriWidgetStoreKeys.configKey(widgetId)],
              let data = raw.data(using: .utf8) else {
            return nil
        }
        let nonce = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
        let receipt = WidgetTransportReceipt(
            source: source,
            widgetId: widgetId,
            group: appGroup,
            instance: "default",
            nonce: nonce,
            size: nil,
            schema: 1,
            rendered: [],
            skipped: []
        )
        writeReceiptEverywhere(receipt, appGroup: appGroup)
        do {
            return try JSONDecoder().decode(WidgetUIConfig.self, from: data)
        } catch {
            logger.error("loadConfig decode error: \(error.localizedDescription)")
            return nil
        }
    }

    /// Raw config + data layers for one placement (own keys first, then the widget's).
    public static func loadSnapshot(
        appGroup: String,
        widgetId: String,
        instance: String?
    ) -> TauriWidgetSnapshot {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else {
            return TauriWidgetSnapshot(rawConfig: nil, hostData: nil, source: "unavailable", nonce: 0)
        }
        #endif
        let scoped = TauriWidgetStoreKeys.scopedId(widgetId, instance)
        var (map, source) = loadFreshestMapWithSource(appGroup: appGroup, widgetId: scoped)
        if map[TauriWidgetStoreKeys.configKey(scoped)] == nil, scoped != widgetId {
            (map, source) = loadFreshestMapWithSource(appGroup: appGroup, widgetId: widgetId)
        }
        let nonce = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
        let raw = TauriWidgetStoreKeys.lookupKeys(TauriWidgetStoreKeys.configPrefix, widgetId, instance)
            .lazy.compactMap { map[$0] }.first
        let host = TauriWidgetStoreKeys.lookupKeys(TauriWidgetStoreKeys.dataPrefix, widgetId, instance)
            .lazy.compactMap { TauriWidgetSnapshot.json(map[$0]) }.first
        var snap = TauriWidgetSnapshot(rawConfig: raw, hostData: host, source: source, nonce: nonce)
        snap.storedState = storedState(map, widgetId, instance)
        return snap
    }

    /// Widget-level layer first, the placement's own last (it wins in the merge).
    static func storedState(_ map: [String: String], _ widgetId: String, _ instance: String?) -> [Any?] {
        TauriWidgetStoreKeys.lookupKeys(TauriWidgetStoreKeys.statePrefix, widgetId, instance)
            .reversed().map { TauriWidgetSnapshot.json(map[$0]) }
    }

    /// Apply a `set` patch to the placement's stored state under the store lock (the host
    /// writes the same map). Returns the new merged state.
    @discardableResult
    public static func applyStatePatch(
        _ patch: [String: Any], appGroup: String, widgetId: String, instance: String?
    ) -> [String: Any] {
        withStoreLock(appGroup: appGroup) {
            var map = loadFreshestMap(appGroup: appGroup)
            let key = TauriWidgetStoreKeys.statePrefix + TauriWidgetStoreKeys.scopedId(widgetId, instance)
            let defaults = map[TauriWidgetStoreKeys.configKey(widgetId)].flatMap(TauriWidgetSnapshot.object)?["state"]
            let current = TauriResolve.mergeData([defaults] + storedState(map, widgetId, instance)) as? [String: Any] ?? [:]
            let own = TauriWidgetSnapshot.json(map[key]) as? [String: Any] ?? [:]
            let next = TauriWidgetState.apply(patch, current: current, stored: own)
            if let data = try? JSONSerialization.data(withJSONObject: next, options: [.sortedKeys]) {
                map[key] = String(decoding: data, as: UTF8.self)
                touchMeta(&map)
                fanoutWrite(map, appGroup: appGroup)
            }
            return TauriResolve.mergeData([defaults] + storedState(map, widgetId, instance)) as? [String: Any] ?? [:]
        }
    }

    /// `instances:{widgetId}` lists of every widget, merged (Edit Widget choices).
    public static func loadInstanceOptions(appGroup: String) -> [(id: String, title: String)] {
        let map = loadFreshestMap(appGroup: appGroup)
        var seen = Set<String>()
        var out: [(String, String)] = []
        for key in map.keys.sorted() where key.hasPrefix(TauriWidgetStoreKeys.instancesPrefix) {
            guard let list = TauriWidgetSnapshot.json(map[key]) as? [[String: Any]] else { continue }
            for item in list {
                guard let id = item["id"] as? String, !id.isEmpty, seen.insert(id).inserted else { continue }
                out.append((id, (item["title"] as? String) ?? id))
            }
        }
        return out
    }

    /// Load config + which transport won (for providers that set instance/size themselves).
    public static func loadConfigWithSource(
        appGroup: String,
        widgetId: String = "default"
    ) -> (WidgetUIConfig?, String, UInt64) {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else {
            return (nil, "unavailable", 0)
        }
        #endif
        let (map, source) = loadFreshestMapWithSource(appGroup: appGroup, widgetId: widgetId)
        let nonce = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
        guard let raw = map[TauriWidgetStoreKeys.configKey(widgetId)],
              let data = raw.data(using: .utf8),
              let cfg = try? JSONDecoder().decode(WidgetUIConfig.self, from: data) else {
            return (nil, source, nonce)
        }
        return (cfg, source, nonce)
    }
    /// iOS: App Group must work — silent fallback looks like an empty widget.
    /// Returns false when the group container is unavailable (callers should stop the load path).
    @discardableResult
    public static func assertAppGroupAvailable(_ appGroup: String) -> Bool {
        #if os(iOS)
        if ProcessInfo.processInfo.environment["WIDGET_APP_GROUP_DATA_FILE"] != nil { return true }
        if FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup) == nil {
            logger.error(
                "App Group '\(appGroup)' unavailable: enable App Groups on BOTH the app and widget extension targets. Silent fallback disabled on iOS."
            )
            return false
        }
        #endif
        return true
    }

    /// Read a single key from the freshest available transport.
    public static func readValue(forKey key: String, appGroup: String) -> String? {
        let map = loadFreshestMap(appGroup: appGroup)
        return map[key]
    }

    /// Fan-out write of a single key into all writable transports.
    public static func writeValue(_ value: String, forKey key: String, appGroup: String) {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else { return }
        #endif
        var map = loadFreshestMap(appGroup: appGroup)
        map[key] = value
        touchMeta(&map)
        fanoutWrite(map, appGroup: appGroup)
    }

    /// Append one action to `pending_actions` under the cross-process store lock.
    ///
    /// The host drains the queue under the same lock, so a tap can neither be lost
    /// (host clearing after our read) nor delivered twice (us re-writing drained items).
    public static func appendPendingAction(_ action: WidgetActionEnvelope, appGroup: String) {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else { return }
        #endif
        withStoreLock(appGroup: appGroup) {
            var map = loadFreshestMap(appGroup: appGroup)
            var queue = pendingActions(in: map)
            queue.append(action)
            guard let out = try? JSONEncoder().encode(queue),
                  let outStr = String(data: out, encoding: .utf8) else { return }
            map[TauriWidgetStoreKeys.pendingActions] = outStr
            touchMeta(&map)
            fanoutWrite(map, appGroup: appGroup)
        }
    }

    /// Exclusive `flock` on `widget_data.json.lock` next to the App Group data file,
    /// shared with the host app (iOS plugin, macOS Rust host). Runs `body` unlocked when
    /// there is no App Group container (ad-hoc macOS builds).
    public static func withStoreLock<T>(appGroup: String, _ body: () -> T) -> T {
        guard let url = dataFileURL(appGroup: appGroup) else { return body() }
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        let fd = open(url.path + ".lock", O_RDWR | O_CREAT, 0o644)
        guard fd >= 0 else { return body() }
        defer { close(fd) }
        flock(fd, LOCK_EX)
        defer { flock(fd, LOCK_UN) }
        return body()
    }

    private static func pendingActions(in map: [String: String]) -> [WidgetActionEnvelope] {
        guard let raw = map[TauriWidgetStoreKeys.pendingActions],
              let data = raw.data(using: .utf8),
              let arr = try? JSONDecoder().decode([WidgetActionEnvelope].self, from: data) else {
            return []
        }
        return arr
    }

    public static func readPendingActions(appGroup: String) -> [WidgetActionEnvelope] {
        guard let raw = readValue(forKey: TauriWidgetStoreKeys.pendingActions, appGroup: appGroup),
              let data = raw.data(using: .utf8),
              let arr = try? JSONDecoder().decode([WidgetActionEnvelope].self, from: data) else {
            return []
        }
        return arr
    }

    // MARK: - Multi-transport

    public static func loadFreshestMap(appGroup: String) -> [String: String] {
        loadFreshestMapWithSource(appGroup: appGroup, widgetId: nil).0
    }

    /// Same as `loadFreshestMap`, but also returns which transport won.
    ///
    /// File transports (container / App Group) win over UserDefaults when they
    /// carry the requested `config:{widgetId}` (or any config when widgetId is nil).
    public static func loadFreshestMapWithSource(
        appGroup: String,
        widgetId: String? = nil
    ) -> ([String: String], String) {
        var fileCandidates: [(String, [String: String])] = []
        var defaultsCandidates: [(String, [String: String])] = []

        // The extension's own container is the macOS `widgetContainer` transport. iOS has
        // only App Groups: a copy there could outrank a reset App Group store by nonce
        // and pin the widget to stale data forever.
        #if os(macOS)
        if let m = readOwnContainerMap() {
            fileCandidates.append((TauriWidgetTransportName.container, m))
        }
        #endif
        if let m = readAppGroupFileMap(appGroup: appGroup) {
            fileCandidates.append((TauriWidgetTransportName.appGroup, m))
        }
        for map in readAllUserDefaultsMaps(appGroup: appGroup) {
            defaultsCandidates.append((TauriWidgetTransportName.defaults, map))
        }

        let filesBest = pickFreshestWithSource(filterByWidgetId(fileCandidates, widgetId))
        if mapHasConfig(filesBest.0, widgetId: widgetId) {
            return filesBest
        }
        return pickFreshestWithSource(
            filterByWidgetId(fileCandidates + defaultsCandidates, widgetId)
        )
    }

    private static func filterByWidgetId(
        _ maps: [(String, [String: String])],
        _ widgetId: String?
    ) -> [(String, [String: String])] {
        guard let widgetId else { return maps }
        let filtered = maps.filter { mapHasConfig($0.1, widgetId: widgetId) }
        // Never fall back to a fresher map that lacks this widget's config.
        return filtered
    }

    private static func mapHasConfig(_ map: [String: String], widgetId: String? = nil) -> Bool {
        if let widgetId {
            return map[TauriWidgetStoreKeys.configKey(widgetId)] != nil
        }
        return map.keys.contains { $0.hasPrefix(TauriWidgetStoreKeys.configPrefix) }
    }

    private static func pickFreshestWithSource(
        _ maps: [(String, [String: String])]
    ) -> ([String: String], String) {
        var best: [String: String] = [:]
        var bestSource = TauriWidgetTransportName.container
        var bestNonce: UInt64 = 0
        var bestTs: UInt64 = 0
        for (source, map) in maps {
            let n = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
            let t = UInt64(map[TauriWidgetStoreKeys.metaUpdatedAt] ?? "0") ?? 0
            if best.isEmpty || n > bestNonce || (n == bestNonce && t > bestTs) {
                best = map
                bestSource = source
                bestNonce = n
                bestTs = t
            }
        }
        return (best, bestSource)
    }

    private static func touchMeta(_ map: inout [String: String]) {
        let next = (UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0) &+ 1
        map[TauriWidgetStoreKeys.metaNonce] = String(next)
        map[TauriWidgetStoreKeys.metaUpdatedAt] = String(UInt64(Date().timeIntervalSince1970 * 1000))
    }

    private static func fanoutWrite(_ map: [String: String], appGroup: String) {
        #if os(macOS)
        writeOwnContainerMap(map)
        #endif
        writeUserDefaultsMap(map, appGroup: appGroup)
        writeAppGroupFileMap(map, appGroup: appGroup)
    }

    // MARK: - Receipts (separate namespace — never touch config map / nonce)

    public static func writeReceiptEverywhere(_ receipt: WidgetTransportReceipt, appGroup: String) {
        guard let data = try? JSONEncoder().encode(receipt),
              let json = String(data: data, encoding: .utf8) else { return }

        // Own container file
        let receiptURL = URL(fileURLWithPath: ownContainerReceiptPath())
        try? FileManager.default.createDirectory(
            at: receiptURL.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try? data.write(to: receiptURL, options: .atomic)

        // UserDefaults suite (sibling key, not inside widget_data)
        let plainSuite = appGroup.hasPrefix("group.") ? String(appGroup.dropFirst(6)) : appGroup
        for suite in [appGroup, plainSuite] {
            if let defaults = UserDefaults(suiteName: suite) {
                defaults.set(json, forKey: TauriWidgetStoreKeys.receiptDefaultsKey)
                defaults.synchronize()
            }
        }

        // App Group shared file
        if let dataURL = dataFileURL(appGroup: appGroup) {
            let url = dataURL.deletingLastPathComponent()
                .appendingPathComponent("widget_receipt.json")
            try? FileManager.default.createDirectory(
                at: url.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            try? data.write(to: url, options: .atomic)
        }
    }

    /// Own-container / WidgetSandboxFile path.
    ///
    /// Overrides (tests / host parity with Rust):
    /// - `WIDGET_SANDBOX_DATA_FILE` — absolute path to `widget_data.json`
    /// - else `WIDGET_CONTAINER_ROOT` + `WIDGET_EXTENSION_BUNDLE` →
    ///   `{root}/Library/Containers/{bundle}/Data/widget_data.json`
    /// - else extension home: `NSHomeDirectory()/widget_data.json`
    public static func ownContainerDataPath() -> String {
        let env = ProcessInfo.processInfo.environment
        if let p = env["WIDGET_SANDBOX_DATA_FILE"], !p.isEmpty {
            return p
        }
        if let root = env["WIDGET_CONTAINER_ROOT"], !root.isEmpty,
           let bundle = env["WIDGET_EXTENSION_BUNDLE"], !bundle.isEmpty {
            return URL(fileURLWithPath: root)
                .appendingPathComponent("Library/Containers")
                .appendingPathComponent(bundle)
                .appendingPathComponent("Data")
                .appendingPathComponent("widget_data.json")
                .path
        }
        return NSHomeDirectory() + "/widget_data.json"
    }

    public static func ownContainerReceiptPath() -> String {
        URL(fileURLWithPath: ownContainerDataPath())
            .deletingLastPathComponent()
            .appendingPathComponent("widget_receipt.json")
            .path
    }

    // WidgetSandboxFile (macOS ad-hoc / no App Group share)
    private static func readOwnContainerMap() -> [String: String]? {
        let ownFile = ownContainerDataPath()
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: ownFile)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return nil
        }
        return json
    }

    private static func writeOwnContainerMap(_ map: [String: String]) {
        let ownFile = ownContainerDataPath()
        let url = URL(fileURLWithPath: ownFile)
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        guard let out = try? JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys]) else {
            return
        }
        try? out.write(to: url, options: .atomic)
    }

    // App Group UserDefaults
    private static func userDefaultsSuites(_ appGroup: String) -> [String] {
        let plain = appGroup.hasPrefix("group.") ? String(appGroup.dropFirst(6)) : appGroup
        return plain == appGroup ? [appGroup] : [appGroup, plain]
    }

    /// Every suite that has a `widget_data` map (both `group.*` and bare id).
    private static func readAllUserDefaultsMaps(appGroup: String) -> [[String: String]] {
        var out: [[String: String]] = []
        for suite in userDefaultsSuites(appGroup) {
            if let defaults = UserDefaults(suiteName: suite),
               let dict = defaults.dictionary(forKey: "widget_data") as? [String: String] {
                out.append(dict)
            }
        }
        return out
    }

    private static func readUserDefaultsMap(appGroup: String) -> [String: String]? {
        let maps = readAllUserDefaultsMaps(appGroup: appGroup)
        guard !maps.isEmpty else { return nil }
        return pickFreshestWithSource(maps.map { (TauriWidgetTransportName.defaults, $0) }).0
    }

    private static func writeUserDefaultsMap(_ map: [String: String], appGroup: String) {
        for suite in userDefaultsSuites(appGroup) {
            if let defaults = UserDefaults(suiteName: suite) {
                // remove+set so CFPreferences notices a change across processes
                defaults.removeObject(forKey: "widget_data")
                defaults.set(map, forKey: "widget_data")
                defaults.synchronize()
            }
        }
    }

    // App Group shared container file
    private static func readAppGroupFileMap(appGroup: String) -> [String: String]? {
        guard let url = dataFileURL(appGroup: appGroup),
              let data = try? Data(contentsOf: url),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return nil
        }
        return json
    }

    private static func writeAppGroupFileMap(_ map: [String: String], appGroup: String) {
        guard let url = dataFileURL(appGroup: appGroup),
              let out = try? JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys]) else {
            return
        }
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try? out.write(to: url, options: .atomic)
    }

    public static func dataFileURL(appGroup: String) -> URL? {
        let env = ProcessInfo.processInfo.environment
        if let p = env["WIDGET_APP_GROUP_DATA_FILE"], !p.isEmpty {
            return URL(fileURLWithPath: p)
        }
        guard let container = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroup
        ) else { return nil }
        return container.appendingPathComponent("widget_data.json")
    }
}
