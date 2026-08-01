namespace TauriWidgets.WidgetProvider;

/// <summary>
/// COM ExeServer host. Widgets Board activates <see cref="WidgetProvider"/> via CLSID.
/// Keep the process alive for out-of-proc COM.
/// </summary>
internal static class Program
{
    [STAThread]
    private static void Main(string[] args)
    {
        // Touch the type so the assembly loads the provider for COM activation.
        _ = typeof(WidgetProvider);

        using var exit = new ManualResetEvent(false);
        Console.CancelKeyPress += (_, e) =>
        {
            e.Cancel = true;
            exit.Set();
        };
        exit.WaitOne();
    }
}
