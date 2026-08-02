using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Text.Json;
#if !WIDGET_SMOKE
using AdaptiveCards;
using AdaptiveCards.Rendering;
using AdaptiveCards.Rendering.Wpf;
#endif

namespace TauriWidgets.PreviewHost;

/// <summary>
/// Headless Adaptive Card preview → PNG.
/// Full path (Smoke=false): AdaptiveCards.Rendering.Wpf RenderCardToImageAsync.
/// Smoke path: walks AC JSON (TextBlock / Image / layout) with System.Drawing.
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
        var forceSmoke = false;

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
                case "--smoke":
                    forceSmoke = true;
                    break;
            }
        }

        if (string.IsNullOrWhiteSpace(input) || string.IsNullOrWhiteSpace(output))
        {
            Console.Error.WriteLine(
                "Usage: AcPreviewHost --in card.json --out out.png [--width 338] [--height 158] [--smoke]");
            return 2;
        }

        var json = File.ReadAllText(input);
        using var bmp = Render(json, width, height, forceSmoke);
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(output))!);
        bmp.Save(output, ImageFormat.Png);
        Console.WriteLine("wrote " + output);
        return 0;
    }

    private static Bitmap Render(string json, int width, int height, bool forceSmoke)
    {
#if !WIDGET_SMOKE
        if (!forceSmoke)
        {
            try
            {
                return RenderAdaptive(json, width, height);
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine("AC WPF render failed, falling back to smoke walker: " + ex.Message);
            }
        }
#endif
        return RenderSmoke(json, width, height);
    }

#if !WIDGET_SMOKE
    private static Bitmap RenderAdaptive(string json, int width, int height)
    {
        var parse = AdaptiveCard.FromJson(json);
        if (parse.Card is null)
        {
            throw new InvalidOperationException(
                "AdaptiveCard.FromJson returned null card: "
                + string.Join("; ", parse.Warnings.Select(w => w.Message)));
        }

        // Dark chrome roughly matching Widgets Board for stable goldens.
        var host = AdaptiveHostConfig.FromJson("""
            {
              "supportsInteractivity": false,
              "containerStyles": {
                "default": {
                  "backgroundColor": "#FF1E1E1E",
                  "foregroundColors": {
                    "default": { "default": "#FFF5F5F5", "subtle": "#FFAAAAAA" },
                    "light": { "default": "#FFF5F5F5", "subtle": "#FFAAAAAA" }
                  }
                },
                "emphasis": {
                  "backgroundColor": "#FF2C2C2E"
                }
              }
            }
            """);
        var renderer = new AdaptiveCardRenderer(host);
        // createStaThread: true is required when not already on a dedicated STA (server/headless).
        var rendered = renderer
            .RenderCardToImageAsync(parse.Card, createStaThread: true, width: Math.Max(width, 1))
            .GetAwaiter()
            .GetResult();

        if (rendered?.ImageStream is null)
        {
            throw new InvalidOperationException("RenderCardToImageAsync produced no image stream");
        }

        using var stream = rendered.ImageStream;
        using var img = Image.FromStream(stream);
        return FitToCanvas(img, width, height, Color.FromArgb(255, 30, 30, 30));
    }
#endif

    private static Bitmap FitToCanvas(Image src, int width, int height, Color bg)
    {
        var bmp = new Bitmap(width, height, PixelFormat.Format32bppArgb);
        using var g = Graphics.FromImage(bmp);
        g.Clear(bg);
        g.InterpolationMode = InterpolationMode.HighQualityBicubic;
        g.SmoothingMode = SmoothingMode.HighQuality;
        g.PixelOffsetMode = PixelOffsetMode.HighQuality;

        var scale = Math.Min((float)width / Math.Max(src.Width, 1), (float)height / Math.Max(src.Height, 1));
        var drawW = src.Width * scale;
        var drawH = src.Height * scale;
        var x = (width - drawW) / 2f;
        var y = (height - drawH) / 2f;
        g.DrawImage(src, x, y, drawW, drawH);
        return bmp;
    }

    private static Bitmap RenderSmoke(string json, int width, int height)
    {
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        var bmp = new Bitmap(width, height, PixelFormat.Format32bppArgb);
        using var g = Graphics.FromImage(bmp);
        g.Clear(Color.FromArgb(255, 30, 30, 30));
        g.TextRenderingHint = System.Drawing.Text.TextRenderingHint.ClearTypeGridFit;
        g.SmoothingMode = SmoothingMode.AntiAlias;

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
                var sizeName = el.TryGetProperty("size", out var sz) ? sz.GetString() ?? "Default" : "Default";
                var em = sizeName switch
                {
                    "Small" => 9f,
                    "Medium" => 13f,
                    "Large" => 16f,
                    "ExtraLarge" => 20f,
                    _ => 11f,
                };
                var weight = el.TryGetProperty("weight", out var w) && w.GetString() == "Bolder"
                    ? FontStyle.Bold
                    : FontStyle.Regular;
                using var font = new Font("Segoe UI", em, weight);
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
                    var scale = Math.Min(drawH / img.Height, maxW / Math.Max(img.Width, 1f));
                    var drawW = img.Width * scale;
                    drawH = img.Height * scale;
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
            {
                if (el.TryGetProperty("actions", out var actions) && actions.ValueKind == JsonValueKind.Array)
                {
                    var ax = x;
                    using var font = new Font("Segoe UI", 10f, FontStyle.Bold);
                    foreach (var act in actions.EnumerateArray())
                    {
                        var title = act.TryGetProperty("title", out var ti) ? ti.GetString() ?? "action" : "action";
                        var size = g.MeasureString(title, font);
                        var rect = new RectangleF(ax, y, size.Width + 16, size.Height + 8);
                        var bg = Color.FromArgb(255, 59, 130, 246);
                        var fg = Color.White;
                        if (act.TryGetProperty("id", out var idEl))
                        {
                            var id = idEl.GetString() ?? "";
                            foreach (var part in id.Split(';'))
                            {
                                if (part.StartsWith("bg:#", StringComparison.OrdinalIgnoreCase))
                                {
                                    bg = ParseHexColor(part[3..], bg);
                                }
                                else if (part.StartsWith("fg:#", StringComparison.OrdinalIgnoreCase))
                                {
                                    fg = ParseHexColor(part[3..], fg);
                                }
                            }
                        }
                        using (var brush = new SolidBrush(bg))
                        {
                            g.FillRectangle(brush, rect);
                        }
                        using (var brush = new SolidBrush(fg))
                        {
                            g.DrawString(title, font, brush, ax + 8, y + 4);
                        }
                        ax += rect.Width + 8;
                        if (ax > x + maxW) break;
                    }
                    return y + 28;
                }
                using (var font = new Font("Segoe UI", 10f, FontStyle.Bold))
                {
                    g.DrawString("[action]", font, Brushes.SkyBlue, x, y);
                }
                return y + 18;
            }
            default:
                return y;
        }
    }

    private static Color ParseHexColor(string hex, Color fallback)
    {
        var h = hex.TrimStart('#');
        try
        {
            if (h.Length == 6)
            {
                return Color.FromArgb(
                    255,
                    Convert.ToInt32(h[..2], 16),
                    Convert.ToInt32(h[2..4], 16),
                    Convert.ToInt32(h[4..6], 16));
            }
            if (h.Length == 8)
            {
                return Color.FromArgb(
                    Convert.ToInt32(h[..2], 16),
                    Convert.ToInt32(h[2..4], 16),
                    Convert.ToInt32(h[4..6], 16),
                    Convert.ToInt32(h[6..8], 16));
            }
        }
        catch
        {
            // keep fallback
        }
        return fallback;
    }
}
