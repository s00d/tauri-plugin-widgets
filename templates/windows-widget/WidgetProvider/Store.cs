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
    }

    public static string DefaultPath()
    {
        var local = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        // Prefer the Tauri app local data dir if set by the host; else fall back.
        var env = Environment.GetEnvironmentVariable("TAURI_WIDGETS_DATA");
        if (!string.IsNullOrWhiteSpace(env))
        {
            return Path.Combine(env, "widget_data.json");
        }
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
        File.WriteAllText(tmp, json);
        File.Copy(tmp, _path, overwrite: true);
        File.Delete(tmp);
    }
}
