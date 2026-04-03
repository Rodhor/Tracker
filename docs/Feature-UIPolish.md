# Feature — UI Polish

**Date:** 2026-04-02
**Architecture reference:** `docs/Architecture.md`
**Decision record:** `docs/Brainstorm-UIEvolution.md`

---

## What this does

Makes the app look considered and consistent. Currently `theme.go` is an empty file. Buttons use Unicode characters instead of icons, have no visual weight hierarchy, and spacing is uneven throughout. This feature addresses all of that.

Scope: visual improvements only — no logic changes.

---

## New concept: implementing `fyne.Theme`

Fyne's theming system is based on an interface. You provide an object that answers questions: "what colour is X?", "how big is Y?", "which font is Z?". Fyne calls these methods when rendering every widget.

The minimal approach is to **extend the built-in theme** and only override what you need — everything else falls back to Fyne's defaults:

```go
// Why: embedding theme.Theme gives you all default methods for free.
// You only override the ones where the default is wrong for this app.
type appTheme struct{}

// fyne.Theme has four methods. All four must be present.
func (t appTheme) Color(name fyne.ThemeColorName, variant fyne.ThemeVariant) color.Color {
    return theme.DefaultTheme().Color(name, variant)  // passthrough
}

func (t appTheme) Font(style fyne.TextStyle) fyne.Resource {
    return theme.DefaultTheme().Font(style)  // passthrough
}

func (t appTheme) Icon(name fyne.ThemeIconName) fyne.Resource {
    return theme.DefaultTheme().Icon(name)  // passthrough
}

// Size is where spacing lives — override specific names, passthrough the rest
func (t appTheme) Size(name fyne.ThemeSizeName) float32 {
    switch name {
    case theme.SizeNameInnerPadding:
        return 6   // space inside widgets (default: 4)
    case theme.SizeNamePadding:
        return 8   // space between widgets (default: 4)
    default:
        return theme.DefaultTheme().Size(name)
    }
}
```

Apply the theme once in `app.go` before any window is created:
```go
app := app.New()
app.Settings().SetTheme(appTheme{})
```

**What to look up:**
- `fyne.ThemeSizeName` constants — `SizeNameInnerPadding`, `SizeNamePadding`, `SizeNameText`, `SizeNameHeadingText`, `SizeNameScrollBar`
- `fyne.ThemeColorName` — if you want to override the accent colour
- Why `app.Settings().SetTheme()` must be called before `ShowAndRun()`

---

## Task 1 — Custom theme with improved spacing

**File:** `internal/ui/theme.go`

**What to build:**
- Define `appTheme struct{}` implementing `fyne.Theme`
- Override `Size()` for inner padding and outer padding (start conservative — 6 and 8 are reasonable)
- Passthrough everything else to `theme.DefaultTheme()`
- Export a `Apply(app fyne.App)` function that calls `app.Settings().SetTheme(appTheme{})`

**In `app.go`:**
- Call `ApplyTheme(app)` (or whatever you name it) immediately after `app.New()`

**Verify:** The app looks slightly more spacious. Widgets are not cramped together.

**Commit:**
```
feat(ui): implement custom Fyne theme with improved spacing
```

---

## Task 2 — Button importance and icons

**New concept: `widget.Importance`**

Fyne buttons have an `Importance` field that maps to a visual style. Use it to create a clear hierarchy:

```go
btn := widget.NewButton("Submit", handler)
btn.Importance = widget.HighImportance   // filled, primary colour — main action
btn.Importance = widget.DangerImportance // red — destructive action
btn.Importance = widget.LowImportance   // muted/ghost — secondary action
// default (MediumImportance): bordered, no fill — standard action
```

**New concept: `widget.NewButtonWithIcon`**

```go
// Why: icon buttons communicate intent faster than text alone
// theme package provides built-in icons so you need no image files
btn := widget.NewButtonWithIcon("Stop", theme.MediaStopIcon(), handler)
```

**What to change:**

| Location | Current | Change to |
|----------|---------|-----------|
| `time_bar.go` — stop button | `widget.NewButton("■", nil)` | `NewButtonWithIcon("Stop", theme.MediaStopIcon(), nil)` + `DangerImportance` |
| `task_list.go` — play button | `widget.NewButton("▶", nil)` | `NewButtonWithIcon("", theme.MediaPlayIcon(), nil)` |
| `task_list.go` — confirm button | `NewButtonWithIcon("", theme.ConfirmIcon(), nil)` | keep icon, set `LowImportance` |
| `status_bar.go` — new task button | `widget.NewButton("+ New Task", ...)` | `NewButtonWithIcon("New Task", theme.ContentAddIcon(), ...)` + `HighImportance` |
| `quick_add.go` — submit button | `widget.NewButton("Submit", nil)` | `NewButtonWithIcon("Add", theme.ContentAddIcon(), nil)` + `HighImportance` |

**What to look up:**
- Full list of `theme.*Icon()` functions — `theme.MediaPlayIcon()`, `theme.MediaStopIcon()`, `theme.ContentAddIcon()`, `theme.ConfirmIcon()`, `theme.DeleteIcon()`
- `widget.Button.IconPlacement` — `widget.ButtonIconLeadingText` vs `widget.ButtonIconTrailingText`

**Verify:** Buttons are visually distinct — the stop button is red, the New Task button is filled/prominent, row controls are quiet.

**Commit:**
```
feat(ui): add button importance levels and built-in icons throughout
```

---

## Task 3 — Task list row layout

**What is wrong:**
The current row uses `container.NewHBox` which stacks widgets left-to-right with equal spacing. The task name does not expand to fill available width — all widgets are squished together.

**New concept: `container.NewBorder` inside a list row**

The same Border layout used for the main window works inside list rows:

```go
// Why: Border lets the name label expand to fill all available width
// while keeping controls pinned to the right at their natural size
func() fyne.CanvasObject {
    name := widget.NewLabel("template")
    name.Truncation = fyne.TextTruncateEllipsis  // ← long names truncate cleanly

    playBtn := widget.NewButtonWithIcon("", theme.MediaPlayIcon(), nil)
    confirmBtn := widget.NewButtonWithIcon("", theme.ConfirmIcon(), nil)
    statusLabel := widget.NewLabel("TODO")

    controls := container.NewHBox(statusLabel, confirmBtn, playBtn)
    return container.NewBorder(nil, nil, nil, controls, name)
    //                          top  bot  left right       centre (expands)
}
```

**What to change in `task_list.go`:**
- Replace the `HBox` template with the `Border`-based template above
- Update the `UpdateItem` function to extract children from the new structure
  - `row.Objects[0]` is now the name label (the centre child of Border)
  - `row.Objects[1]` is the controls HBox — get its children to reach the buttons

**What to look up:**
- `widget.Label.Truncation` — `fyne.TextTruncateEllipsis` vs `fyne.TextTruncateOff`
- How `container.NewBorder` exposes its children via `Objects` — the order is `top, bottom, left, right, centre` which may affect how you index them in `UpdateItem`

**Verify:** Task names fill available width. Long names are truncated with `…`. Controls are right-aligned.

**Commit:**
```
feat(ui): fix task list row layout — name expands, controls right-aligned
```

---

## Task 4 — Timer bar and status bar visual separation

**What is wrong:**
The timer bar and status bar are plain `HBox` containers — no padding, no separation from the task list. They blend in visually.

**What to build:**
- Wrap both the timer bar and the status bar in a `container.NewPadded()` to add breathing room
- Add a `widget.Separator` between the timer bar and the task list (a thin horizontal line)

```go
// Why: Separator draws a single-pixel horizontal line — clear visual boundary
separator := widget.NewSeparator()

// Why: NewPadded wraps any widget with the theme's standard padding on all sides
paddedTimerBar := container.NewPadded(timerBar)
paddedStatusBar := container.NewPadded(statusbar)

topSection := container.NewVBox(paddedTimerBar, separator)
content := container.NewBorder(topSection, paddedStatusBar, nil, nil, centerWidget)
```

**What to look up:**
- `widget.NewSeparator()` — horizontal vs vertical orientation
- `container.NewPadded()` vs manually adding padding via `layout.NewCustomPaddedLayout`

**Verify:** Clear visual boundaries between all three zones. Timer bar and status bar have breathing room.

**Commit:**
```
feat(ui): add padding and separators to timer bar and status bar
```

---

## Task 5 — Quick-add window sizing and focus

**What is wrong:**
The quick-add window opens but the text entry is not auto-focused, so the user has to click it before typing.

**Fix in `quick_add.go`:**
```go
// Auto-focus the entry when the window is shown
quickwin.Canvas().Focus(nameEntry)
```

This should be called each time the window is shown, not just when created. The cleanest place is wherever `quickwin.Show()` is called in `app.go`:
```go
quickAdd.Show()
quickAdd.Canvas().Focus(nameEntry)  // ← focus after Show()
```

Since `nameEntry` is inside `quick_add.go` and not accessible from `app.go`, expose it by returning the window and a `focus()` function from `newQuickAdd`, or export the entry. The simplest approach: return a `show()` helper from `newQuickAdd` that calls both `Show()` and `Focus()`:

```go
// newQuickAdd returns the window and a show() function that handles focus
func newQuickAdd(...) (fyne.Window, func()) {
    ...
    show := func() {
        quickwin.Show()
        quickwin.Canvas().Focus(nameEntry)
    }
    return quickwin, show
}
```

**Verify:** Press `Ctrl+Enter` in the main window — quick-add opens and the cursor is immediately in the text field. No mouse click needed.

**Commit:**
```
feat(ui): auto-focus text entry when quick-add window opens
```
