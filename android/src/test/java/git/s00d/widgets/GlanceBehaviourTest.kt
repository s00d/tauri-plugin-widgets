package git.s00d.widgets

import android.content.Context
import androidx.glance.action.actionParametersOf
import androidx.glance.appwidget.testing.unit.assertHasRunCallbackClickAction
import androidx.glance.appwidget.testing.unit.assertIsChecked
import androidx.glance.appwidget.testing.unit.assertIsNotChecked
import androidx.glance.appwidget.testing.unit.runGlanceAppWidgetUnitTest
import androidx.glance.testing.unit.hasText
import androidx.test.core.app.ApplicationProvider
import org.json.JSONObject
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/** `glance-appwidget-testing`: what a tap sends and what a toggle shows, on the Glance tree. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class GlanceBehaviourTest {
    private val c: Context get() = ApplicationProvider.getApplicationContext()

    /** Given a resolved layout with `set` taps and state-bound toggles, when it composes on the Glance tree, then taps carry action / payload / `set` and toggles follow state. */
    @Test
    fun tapsAndTogglesOnTheGlanceTree() = runGlanceAppWidgetUnitTest {
        val el = Resolve.resolve(
            JSONObject(
                """{"small":{"type":"vstack","children":[
                  {"type":"button","label":"Go","action":"go","payload":"1","set":{"n":"${'$'}inc"}},
                  {"type":"link","label":"Tab B","set":{"tab":"b"}},
                  {"type":"toggle","variant":"checkbox","label":"Milk","bind":{"isOn":"state.milk"},"set":{"milk":"${'$'}toggle"}},
                  {"type":"toggle","variant":"checkbox","label":"Eggs","bind":{"isOn":"state.eggs"},"set":{"eggs":"${'$'}toggle"}}
                ]}}""",
            ),
            Resolve.Context(0, "en", state = JSONObject("""{"milk":true}""")),
        ).getJSONObject("small")
        setContext(c)
        provideComposable { RenderElement(c, el) }
        onNode(hasText("Go")).assertHasRunCallbackClickAction<WidgetActionCallback>(
            actionParametersOf(ACTION_KEY to "go", PAYLOAD_KEY to "1", URL_KEY to "", SET_KEY to """{"n":"${'$'}inc"}"""),
        )
        // `set` only: still clickable.
        onNode(hasText("Tab B")).assertHasRunCallbackClickAction<WidgetActionCallback>(
            actionParametersOf(ACTION_KEY to "", PAYLOAD_KEY to "", URL_KEY to "", SET_KEY to """{"tab":"b"}"""),
        )
        onNode(hasText("Milk")).assertIsChecked()
        // No state for it: unchecked.
        onNode(hasText("Eggs")).assertIsNotChecked()
    }

    /** Given a stored active config with `state` and `data`, when the generated preview composes, then it shows both filled in. */
    @Test
    fun generatedPreviewUsesStoredConfigDataAndState() = runGlanceAppWidgetUnitTest {
        WidgetStoreKeys.metaPrefs(c).edit()
            .putString(WidgetStoreKeys.KEY_ACTIVE_GROUP, "g.prev")
            .putString(WidgetStoreKeys.KEY_ACTIVE_WIDGET_ID, "pw")
            .commit()
        c.getSharedPreferences("g.prev", Context.MODE_PRIVATE).edit().putString(
            "config:pw",
            """{"state":{"n":2},"data":{"x":"ok"},"small":{"type":"text","content":"N={{state.n}} {{data.x}}"}}""",
        ).commit()
        setContext(c)
        provideComposable { WidgetPreview(c, "small") }
        onNode(hasText("N=2 ok")).assertExists()
    }
}
