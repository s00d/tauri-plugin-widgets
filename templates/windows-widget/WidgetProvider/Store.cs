using System.Linq;
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

    /// <summary>Placed Widgets Board instances, read by the host's <c>getActiveWidgets</c>.</summary>
    public const string BoardWidgetsKey = "board_widgets";

    /// <summary>Logical widget id the host writes configs under (<c>setWidgetConfig(…, widgetId)</c>).</summary>
    public static string LogicalId() =>
        Environment.GetEnvironmentVariable("TAURI_WIDGET_LOGICAL_ID") is { Length: > 0 } id ? id : "default";

    public void TrackWidget(string widgetId)
    {
        if (_knownWidgets.TryAdd(widgetId, 0)) PublishBoardWidgets();
    }

    public void UntrackWidget(string widgetId)
    {
        if (_knownWidgets.TryRemove(widgetId, out _)) PublishBoardWidgets();
    }

    /// <summary><c>[{widgetId, instance}]</c> of every tracked board widget.</summary>
    private void PublishBoardWidgets()
    {
        lock (_gate)
        {
            WithFileLock(() =>
            {
                ReloadUnlocked();
                var logical = LogicalId();
                var list = _knownWidgets.Keys.OrderBy(k => k, StringComparer.Ordinal)
                    .Select(k => new Dictionary<string, string> { ["widgetId"] = logical, ["instance"] = k })
                    .ToList();
                _map[BoardWidgetsKey] = JsonSerializer.Serialize(list);
                BumpMetaUnlocked();
                WriteUnlocked();
            });
        }
    }

    public IReadOnlyCollection<string> KnownWidgets => _knownWidgets.Keys.ToArray();

    public string? Get(string key)
    {
        lock (_gate)
        {
            return _map.TryGetValue(key, out var v) ? v : null;
        }
    }

    /// <summary>Card of one board instance: its own (<c>setWidgetConfig(…, instance)</c>), then the logical widget's.</summary>
    public (string Template, string Data) GetInstanceCard(string boardId)
    {
        var logical = LogicalId();
        foreach (var key in new[] { $"{logical}#{boardId}", boardId, logical, "default" })
        {
            if (Get(TemplateKey(key)) is not null) return GetAdaptiveCard(key);
        }
        return GetAdaptiveCard(logical);
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
            WithFileLock(() =>
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
                WriteUnlocked();
            });
        }
    }

    /// <summary>Appends a board report (analytics / error JSON) to <c>provider.log</c> next to the map.</summary>
    public void Log(string kind, string widgetId, string? json)
    {
        try
        {
            var line = JsonSerializer.Serialize(new Dictionary<string, object?>
            {
                ["ts"] = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(),
                ["kind"] = kind,
                ["widgetId"] = widgetId,
                ["json"] = json,
            });
            lock (_gate)
            {
                File.AppendAllText(Path.Combine(Path.GetDirectoryName(_path)!, "provider.log"), line + "\n");
            }
        }
        catch
        {
            // diagnostics only
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

    /// <summary>
    /// Runs reload → modify → write under the cross-process <c>{path}.lock</c> (FileShare.None), the same
    /// lock the Rust host takes in <c>persist_windows_shared_map</c>. The disk copy read under the lock is the
    /// base, so a host write (or a host-drained <c>pending_actions</c>) is never overwritten by stale memory.
    /// </summary>
    private void WithFileLock(Action body)
    {
        var lockPath = _path + ".lock";
        FileStream? lockStream = null;
        for (var attempt = 0; lockStream is null; attempt++)
        {
            try
            {
                lockStream = new FileStream(lockPath, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None);
            }
            catch (IOException) when (attempt < 100)
            {
                // Held by the host (sharing violation): retry like the Rust side, 100 × 20 ms.
                Thread.Sleep(20);
            }
        }
        using (lockStream)
        {
            body();
        }
    }

    /// <summary>Writes <c>_map</c>; call only inside <see cref="WithFileLock"/> after <see cref="ReloadUnlocked"/>.</summary>
    private void WriteUnlocked()
    {
        var tmp = _path + ".tmp";
        File.WriteAllText(tmp, JsonSerializer.Serialize(_map));
        File.Copy(tmp, _path, overwrite: true);
        File.Delete(tmp);
    }
}
