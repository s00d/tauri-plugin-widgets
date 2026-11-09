# Capability matrix (element × platform)

Table is authored in `src/capabilities.rs`; this page embeds it automatically.

### Element lists

**Core** (`CORE_ELEMENTS` in `src/snapshot.rs`): [`vstack`](/elements/layout#vstack), [`hstack`](/elements/layout#hstack), [`zstack`](/elements/layout#zstack), [`container`](/elements/layout#container), [`grid`](/elements/layout#grid), [`text`](/elements/text#text), [`image`](/elements/media#image), [`spacer`](/elements/spacing#spacer), [`divider`](/elements/spacing#divider), [`progress`](/elements/data#progress), [`button`](/elements/interactive#button), [`link`](/elements/interactive#link), [`shape`](/elements/media#shape)

**Extended** (`EXTENDED_ELEMENTS`): [`gauge`](/elements/data#gauge), [`toggle`](/elements/interactive#toggle), [`date`](/elements/text#date), [`chart`](/elements/data#chart), [`list`](/elements/data#list), [`timer`](/elements/text#timer), [`canvas`](/elements/media#canvas), [`label`](/elements/text#label)

### Core

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| [`vstack`](/elements/layout#vstack) | full | full | full | full | full<sup>1</sup> |
| [`hstack`](/elements/layout#hstack) | full | full | full | full | full<sup>1</sup> |
| [`zstack`](/elements/layout#zstack) | full | full | full | full | degraded<sup>2</sup> |
| [`container`](/elements/layout#container) | full | full | full | full | full<sup>1</sup> |
| [`grid`](/elements/layout#grid) | full | full | full | full | full<sup>1</sup> |
| [`text`](/elements/text#text) | full | full | full | full | full<sup>1</sup> |
| [`image`](/elements/media#image) | full | full | full<sup>3</sup> | full | full<sup>4</sup> |
| [`spacer`](/elements/spacing#spacer) | full | full | full | full | full<sup>1</sup> |
| [`divider`](/elements/spacing#divider) | full | full | full | full | full<sup>1</sup> |
| [`progress`](/elements/data#progress) | full | full | full | full | full<sup>1</sup> |
| [`button`](/elements/interactive#button) | full | full | full | full | full<sup>1</sup> |
| [`link`](/elements/interactive#link) | full | full | full | full | full<sup>1</sup> |
| [`shape`](/elements/media#shape) | full | full | full | full | degraded<sup>5</sup> |

### Extended

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| [`gauge`](/elements/data#gauge) | full | full | full | full | degraded<sup>5</sup> |
| [`toggle`](/elements/interactive#toggle) | full | full | full | full | full<sup>1</sup> |
| [`date`](/elements/text#date) | full | full | full | full | full<sup>1</sup> |
| [`chart`](/elements/data#chart) | full | full | full<sup>6</sup> | full<sup>7</sup> | degraded<sup>5</sup> |
| [`list`](/elements/data#list) | full | full | full<sup>8</sup> | full | degraded<sup>9</sup> |
| [`timer`](/elements/text#timer) | full | full | full<sup>10</sup> | full<sup>11</sup> | degraded<sup>12</sup> |
| [`canvas`](/elements/media#canvas) | full | full | degraded<sup>13</sup> | full<sup>7</sup> | degraded<sup>5</sup> |
| [`label`](/elements/text#label) | full | full | full | full | full<sup>1</sup> |

### Feature notes

| Feature | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| [`image.url`](/elements/media#image) | full<sup>14</sup> | full<sup>14</sup> | full<sup>15</sup> | full | full<sup>16</sup> |
| [`image.systemName`](/elements/media#image) | full<sup>17</sup> | full<sup>17</sup> | degraded<sup>18</sup> | degraded<sup>18</sup> | degraded<sup>19</sup> |
| [`background.gradient`](/elements/style) | full<sup>20</sup> | full<sup>20</sup> | full<sup>21</sup> | full<sup>22</sup> | degraded<sup>23</sup> |
| [`canvas.path`](/elements/media#canvas) | full<sup>24</sup> | full<sup>24</sup> | full<sup>25</sup> | full<sup>26</sup> | degraded<sup>27</sup> |
| [`timer.live`](/elements/text#timer) | full<sup>28</sup> | full<sup>28</sup> | full<sup>10</sup> | full<sup>29</sup> | degraded<sup>30</sup> |

### Choosing a surface set

Pick the platforms you ship, then stay in the **full** set for that profile. Degraded cells still render, but check the [Notes](#notes) and element pages.

#### Apple only

Platforms: **iOS + macOS**.

- **Full core:** [`vstack`](/elements/layout#vstack), [`hstack`](/elements/layout#hstack), [`zstack`](/elements/layout#zstack), [`container`](/elements/layout#container), [`grid`](/elements/layout#grid), [`text`](/elements/text#text), [`image`](/elements/media#image), [`spacer`](/elements/spacing#spacer), [`divider`](/elements/spacing#divider), [`progress`](/elements/data#progress), [`button`](/elements/interactive#button), [`link`](/elements/interactive#link), [`shape`](/elements/media#shape)
- **Full extended:** [`gauge`](/elements/data#gauge), [`toggle`](/elements/interactive#toggle), [`date`](/elements/text#date), [`chart`](/elements/data#chart), [`list`](/elements/data#list), [`timer`](/elements/text#timer), [`canvas`](/elements/media#canvas), [`label`](/elements/text#label)

#### Apple + Android

Platforms: **iOS + macOS + Android**.

- **Full core:** [`vstack`](/elements/layout#vstack), [`hstack`](/elements/layout#hstack), [`zstack`](/elements/layout#zstack), [`container`](/elements/layout#container), [`grid`](/elements/layout#grid), [`text`](/elements/text#text), [`image`](/elements/media#image), [`spacer`](/elements/spacing#spacer), [`divider`](/elements/spacing#divider), [`progress`](/elements/data#progress), [`button`](/elements/interactive#button), [`link`](/elements/interactive#link), [`shape`](/elements/media#shape)
- **Full extended:** [`gauge`](/elements/data#gauge), [`toggle`](/elements/interactive#toggle), [`date`](/elements/text#date), [`chart`](/elements/data#chart), [`list`](/elements/data#list), [`timer`](/elements/text#timer), [`label`](/elements/text#label)
- **Extended with degraded/unsupported cells:** [`canvas`](/elements/media#canvas)

#### All five matrix columns

Platforms: **iOS + macOS + Android + Desktop + Windows**.

- **Full core:** [`vstack`](/elements/layout#vstack), [`hstack`](/elements/layout#hstack), [`container`](/elements/layout#container), [`grid`](/elements/layout#grid), [`text`](/elements/text#text), [`image`](/elements/media#image), [`spacer`](/elements/spacing#spacer), [`divider`](/elements/spacing#divider), [`progress`](/elements/data#progress), [`button`](/elements/interactive#button), [`link`](/elements/interactive#link)
- **Core with degraded/unsupported cells:** [`zstack`](/elements/layout#zstack), [`shape`](/elements/media#shape)
- **Full extended:** [`toggle`](/elements/interactive#toggle), [`date`](/elements/text#date), [`label`](/elements/text#label)
- **Extended with degraded/unsupported cells:** [`gauge`](/elements/data#gauge), [`chart`](/elements/data#chart), [`list`](/elements/data#list), [`timer`](/elements/text#timer), [`canvas`](/elements/media#canvas)


### Notes

1. Adaptive Cards 1.5
2. rasterized PNG overlay when possible; else flattened Container
3. url via localPath preprocess; see image.systemName
4. url/data URI; see image.systemName
5. rasterized PNG
6. bitmap bar/line/area/pie
7. SVG
8. Column chunking; soft cap ~50 items
9. Adaptive Cards Table
10. Chronometer via AndroidRemoteViews
11. setInterval
12. provider minute push + static TextBlock
13. bitmap canvas (full SVG path via PathParser)
14. host prefetch to data URI on setWidgetConfig
15. preprocess to localPath on setWidgetConfig
16. Adaptive Cards Image.url
17. SF Symbols
18. SF→Material / emoji map (not SF Symbols)
19. emoji TextBlock by default; glyph PNG Image with feature rasterize
20. linear/radial/angular SwiftUI
21. baked bitmap at LocalSize / frame
22. linear/radial/angular CSS/SVG
23. rasterized PNG backgroundImage when rasterize enabled
24. SVG path grammar
25. PathParser full SVG path
26. SVG path
27. rasterized via SVG
28. Text(..., .timer)
29. JS interval
30. provider pushes UpdateWidget ~1/min
