using System.Runtime.InteropServices;
#if !WIDGET_SMOKE
using Microsoft.Windows.Widgets.Providers;
#endif

namespace TauriWidgets.WidgetProvider;

/// <summary>
/// Thin IWidgetProvider shim: reads Adaptive Card template/data from WidgetStore
/// (written by the Tauri Rust host) and pushes updates to Widgets Board.
/// The host card holds one body per size (<c>$when</c> on <c>$host.widgetSize</c>), so every
/// advertised size renders from the same template.
/// </summary>
[ComVisible(true)]
[Guid("{{WIDGET_PROVIDER_CLSID}}")]
[ClassInterface(ClassInterfaceType.None)]
public sealed class WidgetProvider
#if !WIDGET_SMOKE
    : IWidgetProvider, IWidgetProvider2, IWidgetProviderAnalytics, IWidgetProviderErrors
#endif
{
    /// <summary><see cref="WidgetUpdateRequestOptions.CustomState"/> while the customize card is shown.</summary>
    internal const string CustomizingState = "customizing";
    internal const string ExitCustomizationVerb = "exitCustomization";

    /// <summary>Per-instance CustomState, sent back on every update and recovered on start.</summary>
    private static readonly System.Collections.Concurrent.ConcurrentDictionary<string, string> CustomStates =
        new(StringComparer.Ordinal);
#if !WIDGET_SMOKE
    private static int _recovered;
#endif

    private static readonly WidgetStore Store = new();
    private static readonly ConcurrentWidgetSet Running = new();
    /// <summary>
    /// Minute tick so countdown/timer cards refresh without a live client clock.
    /// Not second-level live — Adaptive Cards have no client JS.
    /// </summary>
    private static readonly System.Threading.Timer RefreshTimer = new(
        _ =>
        {
            foreach (var id in Running.Ids)
            {
                try { Push(id); }
                catch { /* widget may have been removed */ }
            }
        },
        null,
        dueTime: TimeSpan.FromSeconds(15),
        period: TimeSpan.FromMinutes(1));

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
        // Keep the timer rooted for the process lifetime.
        GC.KeepAlive(RefreshTimer);
    }

#if !WIDGET_SMOKE
    public WidgetProvider()
    {
        // Provider (re)started while widgets are pinned: pick them up with their CustomState.
        if (Interlocked.Exchange(ref _recovered, 1) != 0) return;
        try
        {
            foreach (var info in WidgetManager.GetDefault().GetWidgetInfos() ?? Array.Empty<WidgetInfo>())
            {
                var id = info.WidgetContext.Id;
                CustomStates[id] = info.CustomState ?? "";
                Store.TrackWidget(id);
                Running.Add(id);
                Push(id);
            }
        }
        catch
        {
            // Board not ready / no widgets: CreateWidget / Activate will add them.
        }
    }

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
        CustomStates.TryRemove(widgetId, out _);
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
        if (verb == ExitCustomizationVerb)
        {
            CustomStates[widgetId] = "";
            Push(widgetId);
            return;
        }
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

    /// <summary>IWidgetProvider2: "Customize widget" in the board menu. Shows a card whose
    /// "Open app" button reaches the host as action <c>customize</c>; "Done" restores the card.</summary>
    public void OnCustomizationRequested(WidgetCustomizationRequestedArgs customizationRequestedArgs)
    {
        var id = customizationRequestedArgs.WidgetContext.Id;
        CustomStates[id] = CustomizingState;
        Push(id);
    }

    /// <summary>IWidgetProviderAnalytics (Windows 11 build 26900+).</summary>
    public void OnAnalyticsInfoReported(WidgetAnalyticsInfoReportedArgs args) =>
        Store.Log("analytics", args.WidgetContext?.Id ?? "", args.AnalyticsJson);

    /// <summary>IWidgetProviderErrors (Windows 11 build 26900+): e.g. a card the board failed to render.</summary>
    public void OnErrorInfoReported(WidgetErrorInfoReportedArgs args) =>
        Store.Log("error", args.WidgetContext?.Id ?? "", args.ErrorJson);
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
        // Own card of this board instance first, then the logical widget's (see Store.GetInstanceCard).
        var (template, data) = Store.GetInstanceCard(widgetId);
        var state = CustomStates.GetOrAdd(widgetId, "");
        if (state == CustomizingState)
        {
            (template, data) = (CustomizeCard, "{}");
        }

#if !WIDGET_SMOKE
        WidgetUpdateRequestOptions options = new(widgetId)
        {
            Template = template,
            Data = data,
            CustomState = state,
        };
        WidgetManager.GetDefault().UpdateWidget(options);
#else
        _ = (template, data);
#endif
    }

    internal const string CustomizeCard = """
        {"type":"AdaptiveCard","version":"1.5","body":[
          {"type":"TextBlock","text":"Customize this widget in the app.","wrap":true}],
         "actions":[
          {"type":"Action.Execute","title":"Open app","verb":"customize"},
          {"type":"Action.Execute","title":"Done","verb":"exitCustomization"}]}
        """;
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
