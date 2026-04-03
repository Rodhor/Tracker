# Brainstorm — Fyne Desktop UI

**Date:** 2026-03-29
**Status:** Approved — decisions recorded 2026-03-29
**Supersedes:** `docs/plans/2026-03-05-ui-design.md`, `docs/Brainstorm-WebStack.md`

---

## Why this direction

The previous plan used a Go HTTP server + templ + htmx. That approach required you to start the server in a terminal before the app was usable. The new requirement is:

- Launch like any other desktop app (double-click, taskbar, app launcher)
- No terminal needed
- Cross-platform: Windows, Linux, macOS
- Keyboard-driven, with a quick-add shortcut
- Working quickly — functional first, polish later

**Fyne** is the right fit. It produces a real native binary — a `.exe` on Windows, an `.app` bundle on macOS, a binary on Linux that integrates with any app launcher. No web server, no terminal.

The entire data layer (`internal/db/`, `internal/tasks/`, `internal/timer/`) stays unchanged. Only the UI layer is replaced.

---

## What Fyne is and what it gives you

Fyne is a Go GUI framework that renders using OpenGL. It has its own widget set (buttons, lists, inputs, dialogs), a layout system, and a theming system. Styling is not HTML/CSS — you work with Fyne's theme colours and layout containers. The trade-off is real: you lose pixel-perfect CSS control, but you gain a self-contained binary with no browser dependency.

Key capabilities relevant to this app:
- **Dark mode automatic:** follows the OS setting by default
- **Keyboard shortcuts:** registered on the window canvas, fire regardless of which widget is focused
- **System tray menu:** supported on all three platforms via the `desktop.App` interface
- **Live-updating labels:** use `binding.NewString()` — a bound string that updates a label automatically when set from a goroutine (no manual UI refresh needed)
- **Custom widgets:** for anything the standard widgets don't cover, you can embed `widget.BaseWidget` and draw what you need

---

## Reference apps studied

### Toggl Track
The most widely studied app in this category. Key UX decisions that work well:
- Narrow, tall window (~380px wide) — sits beside your work, doesn't dominate the screen
- A persistent **timer bar at the top** that never scrolls away — task name + elapsed time + start/stop button always visible
- The input field in the timer bar doubles as the quick-add entry point: start typing, press Enter, timer starts
- Past time entries grouped by date below the timer bar, each with a "restart" play button
- Right-click the tray icon → quick actions (Stop timer, Continue last)

### Clockify
Wider, more traditional app window. The standout feature worth copying:
- A **"Total today: Xh Xm"** summary line always visible at the bottom of the screen — you can always see your total without scrolling or switching views

### TimeWarrior / TaskWarrior (CLI)
Distil time tracking to its minimum. UX lessons:
- "Continue last timer" is used constantly — it should be a single keypress
- Showing tasks grouped by status (active / todo / done today) reduces visual noise — not everything at once
- A filter/search input to narrow the list is more useful than pagination

### Super Productivity (Electron, open source)
The most feature-complete personal task+time tracker in this category. Useful negative lessons:
- It tries to do everything (Pomodoro, notes, integrations, scheduling) — the overhead of all those features is exactly what a minimal personal tool should avoid
- The "today" queue concept (drag tasks to today from the backlog) is genuinely useful as a workflow, even if the full implementation is overkill here

---

## Proposed layout

### Main window

```
┌──────────────────────────────────────────┐
│  ┌────────────────────────────────────┐  │
│  │ TIMER BAR (fixed, top)             │  │
│  │ "Task name..."          00:42:17 ■ │  │
│  └────────────────────────────────────┘  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │ TASK LIST (scrollable, fills space)│  │
│  │                                    │  │
│  │  ○  Write Q1 report          [▶]   │  │
│  │  ○  Review PR #14            [▶]   │  │
│  │  ●  Draft blog post  2h 3m   [■]   │  │
│  │  ✓  Fix login bug    0h 45m        │  │
│  │                                    │  │
│  └────────────────────────────────────┘  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │ STATUS BAR (fixed, bottom)         │  │
│  │ Today: 3h 22m          [+ New Task]│  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

**Three zones, each with a clear responsibility:**

| Zone | Widget | Responsibility |
|------|--------|----------------|
| Timer bar (top) | Custom widget | Always shows what's running (or idle). Start/stop from here. |
| Task list (middle) | `widget.List` + custom rows | Shows all active tasks. Click ▶ to start timer on that task. |
| Status bar (bottom) | `container.NewHBox` | Today's total tracked time. New task button. |

**Timer bar states:**

- **Idle:** Shows placeholder text "No active timer" in muted colour. A green ▶ button on the right.
- **Running:** Shows task name (truncated if long) + live elapsed time in monospace font (`HH:MM:SS`). A red ■ stop button on the right. Background slightly tinted or a coloured left border to make the running state visually obvious at a glance.

**Task list rows:**

Each row:
- Status dot (○ todo, ● in-progress, ✓ done) — clickable to cycle status
- Task name (left-aligned, expands to fill)
- Total time logged today for this task (right-aligned, monospace, hidden if zero)
- ▶ / ■ button (right side) — start timer on this task / stop if running

Completed tasks: shown with strikethrough and muted colour at the bottom of the list, or hidden behind a toggle.

---

### Quick-add window

A second, minimal window — not the main window. Small (~380 × 90px). Appears when the global hotkey is pressed.

```
┌──────────────────────────────────────┐
│  Add task:                           │
│  ┌──────────────────────────────┐    │
│  │ What are you working on?  ↵  │    │
│  └──────────────────────────────┘    │
└──────────────────────────────────────┘
```

- Input is auto-focused — start typing immediately
- `Enter` adds the task (and optionally starts its timer — open question below)
- `Escape` closes without saving
- Window closes itself after submit

This is implemented as a second `fyne.Window` that is shown/hidden, not created fresh each time. It starts hidden; the hotkey calls `window.Show()` and `window.RequestFocus()`.

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Open new task dialog |
| `Ctrl+Space` | Start/stop timer on selected task (in-window) |
| `Ctrl+Enter` | Open quick-add window (in-window shortcut) |
| `Enter` | Confirm / submit in any input |
| `Escape` | Dismiss dialog / deselect |
| Arrow keys | Navigate task list |

**Global hotkey (works when app is not focused):** `Ctrl+Shift+Space` — opens quick-add window. Uses `golang.design/x/hotkey` (already planned as a dependency).

---

## What changes vs. what stays

### Stays the same (zero changes needed)
- `internal/db/db.go` — database connection and schema
- `internal/tasks/model.go`, `internal/tasks/store.go` — task data layer
- `internal/timer/model.go`, `internal/timer/store.go` — timer data layer
- `go.mod` module name, justfile

### Removed
- `internal/handler/` — the HTTP handler package (no longer needed)
- `internal/templates/` — templ templates (no longer needed)
- `templ`, `htmx` dependencies

### Added
- `internal/ui/` — new package for all Fyne UI code
  - `ui.go` — App struct, window setup, wires everything together
  - `timer_bar.go` — custom timer bar widget
  - `task_list.go` — task list widget and row layout
  - `quick_add.go` — quick-add window
  - `theme.go` — custom colour overrides (optional, can be skipped initially)
- `fyne.io/fyne/v2` dependency
- `golang.design/x/hotkey` dependency (already planned)

### Changed
- `cmd/main.go` — starts the Fyne app instead of the HTTP server

---

## Fyne implementation approach

### Live timer (the most Go-specific part)

The timer display updates every second. In Fyne you must not call UI methods from a goroutine — instead, use a **bound string**:

```go
// In ui.go — create the binding once
elapsed := binding.NewString()
elapsed.Set("00:00:00")

// In timer_bar.go — the label reads from the binding automatically
label := widget.NewLabelWithData(elapsed)

// In a goroutine — safe to call Set() from any thread
go func() {
    ticker := time.NewTicker(time.Second)
    for range ticker.C {
        elapsed.Set(formatElapsed(timerStore.Active()))
    }
}()
```

This is the correct Fyne pattern — `binding.Set()` is thread-safe and triggers a UI refresh.

### Task list

`widget.List` uses a callback model for efficiency — it only renders the visible rows:

```go
list := widget.NewList(
    func() int { return len(tasks) },  // how many items
    func() fyne.CanvasObject {          // create a blank row template
        return container.NewBorder(nil, nil, nil, startBtn, nameLabel)
    },
    func(i widget.ListItemID, obj fyne.CanvasObject) {  // fill in data
        // update nameLabel.SetText(tasks[i].Name), etc.
    },
)
```

When the task list changes (task added, status changed), call `list.Refresh()`.

### System tray

```go
if desk, ok := app.(desktop.App); ok {
    desk.SetSystemTrayMenu(fyne.NewMenu("Track",
        fyne.NewMenuItem("Quick Add", showQuickAdd),
        fyne.NewMenuItem("Stop Timer", stopTimer),
        fyne.NewMenuItemSeparator(),
        fyne.NewMenuItem("Quit", app.Quit),
    ))
}
```

The tray icon can be updated to show a "running" indicator (a different icon) while a timer is active.

---

## Decisions — 2026-03-29

1. **Quick-add behaviour:** The quick-add window supports three modes, selectable within the same window:
   - **Capture only** — creates the task, does not start a timer
   - **Start timer** — creates the task and immediately starts a timer on it
   - **Timer only** — starts a timer without creating a named task (for ad-hoc tracking)
   These can be implemented as a toggle or tab within the quick-add window, or as keyboard modifiers (`Enter` vs `Ctrl+Enter`).

2. **Task status cycling:** Clicking the status dot cycles `todo → in_progress → done`. Stopping a running timer without finishing the task sets status back to `todo` (not `in_progress`). This means the timer stop button and the "mark done" action are two distinct operations.

3. **Completed tasks:** Shown in the same list, muted and at the bottom. Only tasks completed within the last few days are shown (exact cutoff TBD during implementation — 2–3 days is a reasonable default). Older completed tasks are hidden automatically.

4. **Window behaviour on close:** Closing the main window minimises to the system tray — the app keeps running. Timer continues in the background. Quit is only available from the tray menu.

5. **Global hotkey:** `Ctrl+Shift+Space` to open quick-add. Can be changed later.

---

## Suggested next step

Move to **FEATURE** mode to plan the implementation: the new `internal/ui/` package structure, the Fyne widget tree, and the step-by-step build order — one concept at a time.
