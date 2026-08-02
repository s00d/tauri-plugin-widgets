using System.Collections.Concurrent;
using System.Text.Json;

namespace TauriWidgets.WidgetProvider;

/// <summary>
/// Reads the same widget_data.json map the Rust host writes on desktop Windows.
/// Keys: config:{id}, ac:template:{id}, ac:data:{id}, pending_actions, __meta_nonce__.
/// </summary>
public sealed class WidgetStore : IDisposable
{
    public const string PendingActionsKey = "pending_actions";
    public const string MetaNonceKey = "__meta_nonce__";

    private readonly string _path;
    private readonly FileSystemWatcher _watcher;
    private readonly object _gate = new();
    private Dictionary<string, string> _map = new(StringComparer.Ordinal);
    private readonly ConcurrentDictionary<string, byte> _knownWidgets = new(StringComparer.Ordinal);

    public event Action? Changed;

    public WidgetStore(string? path = null)
    {
        _path = path ?? DefaultPath();
        Directory.CreateDirectory(Path.GetDirectoryName(_path)!);
        Reload();

        _watcher = new FileSystemWatcher(Path.GetDirectoryName(_path)!)
        {
            Filter = Path.GetFileName(_path),
            NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.Size | NotifyFilters.CreationTime,
            EnableRaisingEvents = true,
        };
        _watcher.Changed += (_, _) => OnDiskChanged();
        _watcher.Created += (_, _) => OnDiskChanged();
        _watcher.Renamed += (_, e) =>
        {
            if (string.Equals(e.FullPath, _path, StringComparison.OrdinalIgnoreCase)
                || string.Equals(e.OldFullPath, _path, StringComparison.OrdinalIgnoreCase)
                || string.Equals(Path.GetFileName(e.FullPath), Path.GetFileName(_path), StringComparison.OrdinalIgnoreCase))
            {
                OnDiskChanged();
            }
        };
        // Atomic host writes replace via rename — watch the directory for .tmp → final.
        _watcher.NotifyFilter |= NotifyFilters.FileName;
    }

    public static string DefaultPath()
    {
        var local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        // Prefer host-provided paths so provider and Rust share the same map.
        foreach (var key in new[] { "TAURI_WIDGETS_DATA", "TAURI_WIDGET_GROUP", "WIDGET_DATA_DIR" })
        {
            var env = Environment.GetEnvironmentVariable(key);
            if (!string.IsNullOrWhiteSpace(env))
            {
                // TAURI_WIDGETS_DATA is a directory or full .json path; TAURI_WIDGET_GROUP may be a group id.
                if (key == "TAURI_WIDGET_GROUP")
                {
                    return Path.Combine(local, "tauri-plugin-widgets", "widget_data.json");
                }
                var p = env.Trim();
                if (p.EndsWith(".json", StringComparison.OrdinalIgnoreCase))
                {
                    return p;
                }
                return Path.Combine(p, "widget_data.json");
            }
        }
        // Must match Rust desktop.rs Windows storage_path default.
        return Path.Combine(local, "tauri-plugin-widgets", "widget_data.json");
    }

    public static string TemplateKey(string widgetId) => $"ac:template:{widgetId}";
    public static string DataKey(string widgetId) => $"ac:data:{widgetId}";
    public static string ConfigKey(string widgetId) => $"config:{widgetId}";

    public void TrackWidget(string widgetId) => _knownWidgets[widgetId] = 0;

    public void UntrackWidget(string widgetId) => _knownWidgets.TryRemove(widgetId, out _);

    public IReadOnlyCollection<string> KnownWidgets => _knownWidgets.Keys.ToArray();

    public string? Get(string key)
    {
        lock (_gate)
        {
            return _map.TryGetValue(key, out var v) ? v : null;
        }
    }

    public (string Template, string Data) GetAdaptiveCard(string widgetId)
    {
        var template = Get(TemplateKey(widgetId))
            ?? """{"type":"AdaptiveCard","version":"1.5","body":[{"type":"TextBlock","text":"Waiting for host…","wrap":true}]}""";
        var data = Get(DataKey(widgetId)) ?? "{}";
        return (template, data);
    }

    public void EnqueueAction(string verb, string widgetId, JsonElement? data)
    {
        lock (_gate)
        {
            ReloadUnlocked();
            var list = new List<JsonElement>();
            if (_map.TryGetValue(PendingActionsKey, out var raw) && !string.IsNullOrWhiteSpace(raw))
            {
                try
                {
                    using var doc = JsonDocument.Parse(raw);
                    if (doc.RootElement.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var el in doc.RootElement.EnumerateArray())
                        {
                            list.Add(el.Clone());
                        }
                    }
                }
                catch
                {
                    // reset corrupt queue
                }
            }

            using var envelope = JsonDocument.Parse(JsonSerializer.Serialize(new Dictionary<string, object?>
            {
                ["action"] = verb,
                ["payload"] = data.HasValue ? data.Value.GetRawText() : null,
                ["ts"] = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(),
                ["widgetId"] = widgetId,
                ["group"] = Environment.GetEnvironmentVariable("TAURI_WIDGET_GROUP") ?? "default",
            }));
            list.Add(envelope.RootElement.Clone());

            _map[PendingActionsKey] = JsonSerializer.Serialize(list);
            BumpMetaUnlocked();
            PersistUnlocked();
        }
    }

    public void Dispose()
    {
        _watcher.Dispose();
    }

    private void OnDiskChanged()
    {
        try
        {
            // Brief settle — writers may replace atomically.
            Thread.Sleep(40);
            Reload();
            Changed?.Invoke();
        }
        catch
        {
            // ignore transient IO
        }
    }

    private void Reload()
    {
        lock (_gate)
        {
            ReloadUnlocked();
        }
    }

    private void ReloadUnlocked()
    {
        if (!File.Exists(_path))
        {
            _map = new Dictionary<string, string>(StringComparer.Ordinal);
            return;
        }

        try
        {
            var json = File.ReadAllText(_path);
            var parsed = JsonSerializer.Deserialize<Dictionary<string, string>>(json);
            _map = parsed ?? new Dictionary<string, string>(StringComparer.Ordinal);
        }
        catch
        {
            // keep previous map on parse failure
        }
    }

    private void BumpMetaUnlocked()
    {
        var nonce = 0UL;
        if (_map.TryGetValue(MetaNonceKey, out var s))
        {
            _ = ulong.TryParse(s, out nonce);
        }
        _map[MetaNonceKey] = (nonce + 1).ToString();
        _map["__meta_updated_at__"] = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds().ToString();
    }

    private void PersistUnlocked()
    {
        var json = JsonSerializer.Serialize(_map);
        var tmp = _path + ".tmp";
        // Cross-process coordination: exclusive lock file around read-modify-write.
        var lockPath = _path + ".lock";
        using var lockStream = new FileStream(
            lockPath,
            FileMode.OpenOrCreate,
            FileAccess.ReadWrite,
            FileShare.None);
        // Re-read under lock so concurrent host writes are not clobbered blindly.
        if (File.Exists(_path))
        {
            try
            {
                var disk = JsonSerializer.Deserialize<Dictionary<string, string>>(File.ReadAllText(_path))
                    ?? new Dictionary<string, string>(StringComparer.Ordinal);
                foreach (var kv in disk)
                {
                    if (!_map.ContainsKey(kv.Key))
                    {
                        _map[kv.Key] = kv.Value;
                    }
                }
                // Prefer non-empty pending_actions from either side.
                if (disk.TryGetValue(PendingActionsKey, out var diskPa)
                    && _map.TryGetValue(PendingActionsKey, out var memPa))
                {
                    var diskEmpty = string.IsNullOrWhiteSpace(diskPa) || diskPa.Trim() == "[]";
                    var memEmpty = string.IsNullOrWhiteSpace(memPa) || memPa.Trim() == "[]";
                    if (memEmpty && !diskEmpty)
                    {
                        _map[PendingActionsKey] = diskPa;
                    }
                }
                json = JsonSerializer.Serialize(_map);
            }
            catch
            {
                // keep in-memory map
            }
        }
        File.WriteAllText(tmp, json);
        File.Copy(tmp, _path, overwrite: true);
        File.Delete(tmp);
    }
}
