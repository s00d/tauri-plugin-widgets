package git.s00d.widgets

internal const val GLANCE_CONTAINER_LIMIT = 10
internal const val GLANCE_LIST_LIMIT = 50
// Glance Column hard-caps ~10 children. With per-item Spacers a chunk of N
// becomes 2N-1 composables — keep N≤5 so rows are not silently dropped.
internal const val GLANCE_LIST_CHUNK = 5
internal const val TAG = "TauriGlanceWidget"
