package git.s00d.widgets

import android.app.Activity
import android.graphics.Color
import android.graphics.PixelFormat
import android.graphics.drawable.ColorDrawable
import android.os.Bundle
import android.widget.FrameLayout

/** Transparent window that hosts the widget view for WidgetRenderTest (PixelCopy keeps alpha). */
class CaptureActivity : Activity() {
    lateinit var root: FrameLayout
        private set

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.setFormat(PixelFormat.TRANSLUCENT)
        window.setBackgroundDrawable(ColorDrawable(Color.TRANSPARENT))
        root = FrameLayout(this)
        setContentView(root)
    }
}
