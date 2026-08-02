using System.Runtime.InteropServices;
#if !WIDGET_SMOKE
using Microsoft.Windows.Widgets.Providers;
#endif

namespace TauriWidgets.WidgetProvider;

/// <summary>
/// Thin IWidgetProvider shim: reads Adaptive Card template/data from WidgetStore
/// (written by the Tauri Rust host) and pushes updates to Widgets Board.
/// </summary>
[ComVisible(true)]
[Guid("{{WIDGET_PROVIDER_CLSID}}")]
[ClassInterface(ClassInterfaceType.None)]
public sealed class WidgetProvider
#if !WIDGET_SMOKE
    : IWidgetProvider
#endif
{
    private static readonly WidgetStore Store = new();
    private static readonly ConcurrentWidgetSet Running = new();

    static WidgetProvider()
    {
        Store.Changed += () =>
        {
            foreach (var id in Running.Ids)
            {
                try
                {
                    Push(id);
                }
                catch
                {
                    // host may have removed the widget mid-update
                }
            }
        };
    }

#if !WIDGET_SMOKE
    public void CreateWidget(WidgetContext widgetContext)
    {
        var id = widgetContext.Id;
        Store.TrackWidget(id);
        Running.Add(id);
        Push(id);
    }

    public void DeleteWidget(string widgetId, string customState)
    {
        Running.Remove(widgetId);
        Store.UntrackWidget(widgetId);
        if (Running.Ids.Length == 0)
        {
            Program.ExitEvent.Set();
        }
    }

    public void OnActionInvoked(WidgetActionInvokedArgs actionInvokedArgs)
    {
        var verb = actionInvokedArgs.Verb ?? "action";
        var widgetId = actionInvokedArgs.WidgetContext.Id;
        System.Text.Json.JsonElement? data = null;
        try
        {
            var raw = actionInvokedArgs.Data;
            if (!string.IsNullOrWhiteSpace(raw))
            {
                using var doc = System.Text.Json.JsonDocument.Parse(raw);
                data = doc.RootElement.Clone();
            }
        }
        catch
        {
            // ignore malformed action data
        }
        Store.EnqueueAction(verb, widgetId, data);
        Push(widgetId);
    }

    public void OnWidgetContextChanged(WidgetContextChangedArgs contextChangedArgs)
    {
        Push(contextChangedArgs.WidgetContext.Id);
    }

    public void Activate(WidgetContext widgetContext)
    {
        Running.Add(widgetContext.Id);
        Store.TrackWidget(widgetContext.Id);
        Push(widgetContext.Id);
    }

    public void Deactivate(string widgetId)
    {
        Running.Remove(widgetId);
    }
#else
    // WIDGET_SMOKE: compile-check path without WinAppSDK / VS Appx tools.
    public void CreateWidget(string widgetId)
    {
        Store.TrackWidget(widgetId);
        Running.Add(widgetId);
        Push(widgetId);
    }

    public void DeleteWidget(string widgetId)
    {
        Running.Remove(widgetId);
        Store.UntrackWidget(widgetId);
        if (Running.Ids.Length == 0)
        {
            Program.ExitEvent.Set();
        }
    }
#endif

    private static void Push(string widgetId)
    {
        // Host may key Adaptive Cards by logical widget id ("default") rather than
        // the Widgets Board instance id. Prefer instance id, then fall back to logical.
        var (template, data) = Store.GetAdaptiveCard(widgetId);
        if (template.Contains("Waiting for host", StringComparison.Ordinal)
            && !string.IsNullOrEmpty(Environment.GetEnvironmentVariable("TAURI_WIDGET_LOGICAL_ID")))
        {
            (template, data) = Store.GetAdaptiveCard(
                Environment.GetEnvironmentVariable("TAURI_WIDGET_LOGICAL_ID")!);
        }

        // Also try common logical id used by the plugin.
        if (template.Contains("Waiting for host", StringComparison.Ordinal))
        {
            (template, data) = Store.GetAdaptiveCard("default");
        }

#if !WIDGET_SMOKE
        WidgetUpdateRequestOptions options = new(widgetId)
        {
            Template = template,
            Data = data,
        };
        WidgetManager.GetDefault().UpdateWidget(options);
#else
        _ = (template, data);
#endif
    }
}

/// <summary>Thread-safe set of active Widgets Board instance ids.</summary>
internal sealed class ConcurrentWidgetSet
{
    private readonly HashSet<string> _ids = new(StringComparer.Ordinal);
    private readonly object _gate = new();

    public void Add(string id)
    {
        lock (_gate) { _ids.Add(id); }
    }

    public void Remove(string id)
    {
        lock (_gate) { _ids.Remove(id); }
    }

    public string[] Ids
    {
        get { lock (_gate) { return _ids.ToArray(); } }
    }
}
