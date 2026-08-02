using System.Runtime.InteropServices;
using ComTypes = System.Runtime.InteropServices.ComTypes;
#if !WIDGET_SMOKE
using WinRT;
#endif

namespace TauriWidgets.WidgetProvider;

/// <summary>
/// COM ExeServer host. Widgets Board activates <see cref="WidgetProvider"/> via CLSID.
/// Must register an <see cref="IClassFactory"/> with <c>CoRegisterClassObject</c> before waiting.
/// </summary>
internal static class Program
{
    private const uint CLSCTX_LOCAL_SERVER = 0x4;
    private const uint REGCLS_MULTIPLEUSE = 0x1;

    /// <summary>Signaled when the last Widgets Board instance is deleted (or Ctrl+C).</summary>
    internal static readonly ManualResetEvent ExitEvent = new(false);

    [DllImport("ole32.dll")]
    private static extern int CoRegisterClassObject(
        [MarshalAs(UnmanagedType.LPStruct)] Guid rclsid,
        [MarshalAs(UnmanagedType.Interface)] IClassFactory pUnk,
        uint dwClsContext,
        uint flags,
        out uint lpdwRegister);

    [DllImport("ole32.dll")]
    private static extern int CoRevokeClassObject(uint dwRegister);

    [STAThread]
    private static void Main(string[] args)
    {
#if !WIDGET_SMOKE
        try
        {
            WinRT.ComWrappersSupport.InitializeComWrappers();
        }
        catch
        {
            // Older runtimes may not expose InitializeComWrappers; registration still required.
        }
#endif

        var factory = new WidgetProviderFactory();
        var hr = CoRegisterClassObject(
            typeof(WidgetProvider).GUID,
            factory,
            CLSCTX_LOCAL_SERVER,
            REGCLS_MULTIPLEUSE,
            out var cookie);
        if (hr < 0)
        {
            Marshal.ThrowExceptionForHR(hr);
        }

        try
        {
            Console.CancelKeyPress += (_, e) =>
            {
                e.Cancel = true;
                ExitEvent.Set();
            };
            ExitEvent.WaitOne();
        }
        finally
        {
            if (cookie != 0)
            {
                _ = CoRevokeClassObject(cookie);
            }
        }
    }
}

[ComVisible(true)]
[ClassInterface(ClassInterfaceType.None)]
internal sealed class WidgetProviderFactory : IClassFactory
{
    public int CreateInstance(nint pUnkOuter, ref Guid riid, out nint ppvObject)
    {
        ppvObject = 0;
        if (pUnkOuter != 0)
        {
            return unchecked((int)0x80040110); // CLASS_E_NOAGGREGATION
        }

#if !WIDGET_SMOKE
        // Widgets Board asks for IWidgetProvider (or IUnknown) — typeof(WidgetProvider).GUID is the CLSID, not an IID.
        var provider = new WidgetProvider();
        nint inspectable = MarshalInspectable<WidgetProvider>.FromManaged(provider);
        try
        {
            return Marshal.QueryInterface(inspectable, ref riid, out ppvObject);
        }
        finally
        {
            Marshal.Release(inspectable);
        }
#else
        var unk = Marshal.GetIUnknownForObject(new WidgetProvider());
        try
        {
            return Marshal.QueryInterface(unk, ref riid, out ppvObject);
        }
        finally
        {
            Marshal.Release(unk);
        }
#endif
    }

    public int LockServer(bool fLock) => 0;
}

[ComImport]
[ComVisible(false)]
[Guid("00000001-0000-0000-C000-000000000046")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IClassFactory
{
    [PreserveSig]
    int CreateInstance(nint pUnkOuter, ref Guid riid, out nint ppvObject);

    [PreserveSig]
    int LockServer([MarshalAs(UnmanagedType.Bool)] bool fLock);
}
