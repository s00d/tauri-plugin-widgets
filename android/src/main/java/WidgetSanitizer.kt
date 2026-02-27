package git.s00d.widgets

object WidgetSanitizer {
    private val allowed = Regex("[^A-Za-z0-9._-]")

    fun sanitizeGroup(group: String, fallback: String): String {
        val clean = group.trim().replace(allowed, "_")
        return if (clean.isNotEmpty()) clean else fallback
    }

    fun sanitizeKey(key: String): String {
        val clean = key.trim().replace(allowed, "_")
        return clean.ifEmpty { "_" }
    }
}
