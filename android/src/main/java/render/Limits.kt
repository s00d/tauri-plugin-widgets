package git.s00d.widgets

internal const val GLANCE_CONTAINER_LIMIT = 10
internal const val GLANCE_LIST_LIMIT = 50
// Glance Column hard-caps ~10 children. With per-item Spacers a chunk of N
// becomes 2N-1 composables — keep N≤5 so rows are not silently dropped.
internal const val GLANCE_LIST_CHUNK = 5
// Glance caps the emittable tree at 50 levels; one IR element can emit up to ~3 nested
// containers (margin box, background box, layout), so element nesting stops at 15.
internal const val GLANCE_MAX_ELEMENT_DEPTH = 15
internal const val TAG = "TauriGlanceWidget"
