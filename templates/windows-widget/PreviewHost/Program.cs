using System.Drawing;
using System.Drawing.Imaging;
using System.Text.Json;

namespace TauriWidgets.PreviewHost;

/// <summary>
/// Headless Adaptive Card preview → PNG.
/// Smoke path: walks AC JSON (TextBlock / Image data URIs) with System.Drawing.
/// Full path (Smoke=false): AdaptiveCards.Rendering.Wpf when packages restore.
/// </summary>
internal static class Program
{
    [STAThread]
    private static int Main(string[] args)
    {
        try
        {
            return Run(args);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(ex.ToString());
            return 1;
        }
    }

    private static int Run(string[] args)
    {
        string? input = null;
        string? output = null;
        int width = 338;
        int height = 158;

        for (var i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "--in":
                case "-i":
                    input = args[++i];
                    break;
                case "--out":
                case "-o":
                    output = args[++i];
                    break;
                case "--width":
                    width = int.Parse(args[++i]);
                    break;
                case "--height":
                    height = int.Parse(args[++i]);
                    break;
            }
        }

        if (string.IsNullOrWhiteSpace(input) || string.IsNullOrWhiteSpace(output))
        {
            Console.Error.WriteLine("Usage: AcPreviewHost --in card.json --out out.png [--width 338] [--height 158]");
            return 2;
        }

        var json = File.ReadAllText(input);
        using var bmp = RenderSmoke(json, width, height);
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(output))!);
        bmp.Save(output, ImageFormat.Png);
        Console.WriteLine("wrote " + output);
        return 0;
    }

    private static Bitmap RenderSmoke(string json, int width, int height)
    {
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        var bmp = new Bitmap(width, height, PixelFormat.Format32bppArgb);
        using var g = Graphics.FromImage(bmp);
        g.Clear(Color.FromArgb(255, 30, 30, 30));
        g.TextRenderingHint = System.Drawing.Text.TextRenderingHint.ClearTypeGridFit;

        var y = 8f;
        if (root.TryGetProperty("body", out var body) && body.ValueKind == JsonValueKind.Array)
        {
            foreach (var el in body.EnumerateArray())
            {
                y = DrawElement(g, el, 8f, y, width - 16f);
                if (y > height - 8) break;
            }
        }

        return bmp;
    }

    private static float DrawElement(Graphics g, JsonElement el, float x, float y, float maxW)
    {
        if (!el.TryGetProperty("type", out var typeEl)) return y;
        var type = typeEl.GetString() ?? "";

        switch (type)
        {
            case "TextBlock":
            {
                var text = el.TryGetProperty("text", out var t) ? t.GetString() ?? "" : "";
                if (string.IsNullOrWhiteSpace(text)) return y + 4;
                using var font = new Font("Segoe UI", 11f, FontStyle.Regular);
                var size = g.MeasureString(text, font, (int)maxW);
                g.DrawString(text, font, Brushes.WhiteSmoke, new RectangleF(x, y, maxW, size.Height));
                return y + size.Height + 4;
            }
            case "Image":
            {
                var url = el.TryGetProperty("url", out var u) ? u.GetString() ?? "" : "";
                if (url.StartsWith("data:image", StringComparison.OrdinalIgnoreCase))
                {
                    var b64 = url[(url.IndexOf(',') + 1)..];
                    var bytes = Convert.FromBase64String(b64);
                    using var ms = new MemoryStream(bytes);
                    using var img = Image.FromStream(ms);
                    var drawH = Math.Min(img.Height, 120f);
                    var scale = drawH / img.Height;
                    var drawW = Math.Min(img.Width * scale, maxW);
                    g.DrawImage(img, x, y, drawW, drawH);
                    return y + drawH + 6;
                }
                return y + 8;
            }
            case "Container":
            case "Column":
            {
                if (el.TryGetProperty("items", out var items) && items.ValueKind == JsonValueKind.Array)
                {
                    foreach (var child in items.EnumerateArray())
                    {
                        y = DrawElement(g, child, x, y, maxW);
                    }
                }
                return y;
            }
            case "ColumnSet":
            {
                if (el.TryGetProperty("columns", out var cols) && cols.ValueKind == JsonValueKind.Array)
                {
                    var list = cols.EnumerateArray().ToList();
                    var colW = maxW / Math.Max(list.Count, 1);
                    var startY = y;
                    var maxY = y;
                    for (var i = 0; i < list.Count; i++)
                    {
                        var cy = DrawElement(g, list[i], x + i * colW, startY, colW - 4);
                        maxY = Math.Max(maxY, cy);
                    }
                    return maxY;
                }
                return y;
            }
            case "ActionSet":
                using (var font = new Font("Segoe UI", 10f, FontStyle.Bold))
                {
                    g.DrawString("[action]", font, Brushes.SkyBlue, x, y);
                }
                return y + 18;
            default:
                return y;
        }
    }
}
