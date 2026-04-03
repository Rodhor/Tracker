# Feature — Fyne Desktop UI

**Date:** 2026-03-29
**Architecture reference:** `docs/Architecture.md`
**Decision record:** `docs/Brainstorm-FyneUI.md`

---

## What this feature does

Replaces the web server UI (templ + htmx + HTTP handler) with a native Fyne desktop application. The app opens as a regular desktop window, minimises to the system tray when closed, and supports a global hotkey for quick-add without the window being open.

The data layer (`internal/db/`, `internal/tasks/`, `internal/timer/`) is untouched. Only the UI layer and entry point change.

---

## What needs to change first — store additions

Two new store methods are needed before any UI code is written. The UI cannot function without them.

### `internal/tasks/store.go` — two new methods

**`UpdateStatus(id int64, status TaskStatus) error`**
Used by the status dot click cycle and by the timer stop action (which sets status back to `todo`).

**`ListRecent(days int) ([]Task, error)`**
Returns all tasks where either:
- Status is not `done`, OR
- Status is `done` and `created_at` is within the last `days` days

This powers the task list: active tasks always show, completed tasks only show if recent.

### `internal/timer/store.go` — one change

**`Stop() error`** already exists but only sets `ended_at` and `minutes`. It does not touch the task's status. That is correct — the UI layer will call `timerStore.Stop()` and then separately call `taskStore.UpdateStatus(taskID, tasks.TODO)` to reset the task. The two stores stay independent.

---

## What gets removed

- `internal/handler/` — delete the entire directory
- `internal/templates/` — delete the entire directory
- `templ`, `htmx`, `golang.design/x/hotkey` references from `go.mod` (hotkey will be re-added as a real dependency once we reach Task 8)

The handler.go file currently exists but is uncommitted — it can simply be deleted rather than reverted.

---

## New structure — `internal/ui/`

```
internal/ui/
├── app.go          # owns: App struct, Fyne app + main window setup, wires stores to UI
│                   # does not own: individual widget layout or SQL
├── timer_bar.go    # owns: timer bar widget (top panel, always visible)
│                   # does not own: task list, store calls outside of its own refresh
├── task_list.go    # owns: task list widget, row layout, status dot, start/stop button per row
│                   # does not own: timer display, quick-add
├── status_bar.go   # owns: bottom bar — today's total time, New Task button
│                   # does not own: task data beyond total calculation
├── quick_add.go    # owns: quick-add window (separate fyne.Window), three-mode form
│                   # does not own: main window layout
└── theme.go        # owns: custom colour overrides (primary accent colour only — defer full theming)
```

---

## Implementation tasks — one concept at a time

---

### Task 1 — Add Fyne and open a blank window

**What you're building:** The minimal Fyne app: a window that opens, shows a title, and can be closed.

**New concept: Fyne app lifecycle**

Every Fyne program follows the same pattern:

```go
// Why: fyne.App is the process-level handle — create exactly one
app := app.New()

// Why: a Window is what the user sees — you can have more than one
win := app.NewWindow("tasktracker")
win.Resize(fyne.NewSize(420, 600))
win.ShowAndRun()  // blocks until the window is closed
```

`ShowAndRun()` starts the Fyne event loop. Everything after it is unreachable — any work that needs to happen (loading data, starting goroutines) must happen before this call or be launched in a goroutine first.

**What to do:**

1. Run `go get fyne.io/fyne/v2` to add the dependency
2. Create `internal/ui/app.go` with a `Run(taskStore, timerStore)` function that creates the app, creates a window, and calls `ShowAndRun()`
3. Update `cmd/main.go`: replace the test code with `db.Init()` → create stores → `ui.Run(taskStore, timerStore)`

**What to look up:**
- `fyne.io/fyne/v2/app` package — `app.New()` vs `app.NewWithID(id)`
- `win.SetMaster()` — what does marking a window as master do for the app lifecycle?
- Why `ShowAndRun()` must be called from the main goroutine (Fyne requirement)

**Verify:** `just run` — a blank titled window opens. Close it — the process exits.

**Commit:**
```
feat(ui): add Fyne dependency and open blank window
```

---

### Task 2 — Window layout skeleton

**What you're building:** The three-zone layout — placeholder content in each zone so you can see the structure before any real widgets exist.

**New concept: `container.NewBorder`**

Border is Fyne's most useful layout for app shells. It pins content to edges and lets one child fill the remaining space:

```go
// Why: Border gives fixed-height top/bottom, the centre expands to fill
content := container.NewBorder(
    topWidget,    // fixed height — top edge
    bottomWidget, // fixed height — bottom edge
    nil,          // left edge — none
    nil,          // right edge — none
    centreWidget, // fills all remaining space
)
win.SetContent(content)
```

**What to do:**

1. In `app.go`, build the border layout with three `widget.Label` placeholders: "timer bar", "task list", "status bar"
2. Set it as the window content

**What to look up:**
- What happens when you pass `nil` for an edge in `container.NewBorder`
- `container.NewVBox` vs `container.NewBorder` — when would you choose each for a top/middle/bottom layout?

**Verify:** Window shows three labelled zones stacked correctly. Resizing the window — only the middle zone grows.

**Commit:**
```
feat(ui): add three-zone window layout skeleton
```

---

### Task 3 — Live timer bar

**What you're building:** The top panel showing the active timer — task name + elapsed time + stop button. Updates every second.

**New concept: `binding.NewString()` for live UI updates**

In Fyne, you must never update a widget's text from a goroutine directly (it is not thread-safe). The correct pattern is a **data binding** — a value that both the goroutine writes to and the label reads from:

```go
// Why: binding.NewString is thread-safe — safe to Set() from any goroutine
elapsed := binding.NewString()
elapsed.Set("00:00:00")

// Why: NewLabelWithData re-renders automatically whenever the binding changes
label := widget.NewLabelWithData(elapsed)

// In a goroutine — safe to call
go func() {
    ticker := time.NewTicker(time.Second)
    defer ticker.Stop()
    for range ticker.C {
        elapsed.Set(computeElapsed())  // binding triggers label refresh
    }
}()
```

**What to build:**

`internal/ui/timer_bar.go`:
- A function `newTimerBar(timerStore) fyne.CanvasObject` that returns the bar as a container
- A `widget.Label` bound to an elapsed string — shows `00:00:00` when idle, live time when running
- A `widget.Label` for the task name — shows "No active timer" when idle
- A `widget.Button` — shows ▶ when idle (greyed out, no action yet), ■ when running (calls `timerStore.Stop()` and refreshes)
- A goroutine that ticks every second, calls `timerStore.Active()`, and updates the bound string

The elapsed format should be `HH:MM:SS`. Calculate it from `time.Since(startTime)`.

**What to look up:**
- `fyne.io/fyne/v2/data/binding` package — `NewString()`, `Set()`, `Get()`
- `widget.NewLabelWithData` vs `label.SetText()` — what is the difference in terms of thread safety?
- `widget.Button.Importance` — how do you make a button visually prominent (`widget.HighImportance`) or danger-coloured (`widget.DangerImportance`)?
- How to use `container.NewHBox` to lay out the three elements (name label, elapsed label, button) in a horizontal row

**Verify:** Run the app. Start a timer directly in `cmd/main.go` via the store (temporary). The timer bar should show elapsed time ticking up each second.

**Commit:**
```
feat(ui): add live timer bar with binding-driven elapsed display
```

---

### Task 4 — Task list

**What you're building:** A scrollable list of tasks, one row per task, loaded from the database.

**New concept: `widget.List` with callbacks**

Fyne's `widget.List` is efficient for large lists — it only renders the rows currently visible on screen. You provide three functions:

```go
list := widget.NewList(
    // 1. How many items are there?
    func() int { return len(tasks) },

    // 2. Create a blank row template (called once per visible slot)
    func() fyne.CanvasObject {
        return widget.NewLabel("template")
    },

    // 3. Fill in data for row i (called whenever a row scrolls into view)
    func(i widget.ListItemID, obj fyne.CanvasObject) {
        obj.(*widget.Label).SetText(tasks[i].Name)
    },
)
```

**What to build:**

`internal/ui/task_list.go`:
- A function `newTaskList(taskStore) *widget.List` that returns the list
- Load tasks once on startup using `taskStore.ListRecent(3)` (you'll add this store method as part of this task)
- Each row: task name label (left, expands), ▶ button (right, placeholder — wired in Task 5)
- A `Refresh()` mechanism: a function that reloads tasks from the store and calls `list.Refresh()`

In `app.go`:
- Pass `container.NewScroll(taskList)` as the centre of the border layout (makes it scrollable)

**What to look up:**
- How to extract the correct child widget from `obj` in `UpdateItem` when the row template contains multiple widgets (you'll need a container, not a bare label)
- `container.NewBorder` inside a list row — for left-expand, right-fixed layout
- `widget.List.OnSelected` — what happens when the user clicks a row?

**Verify:** Tasks from the database appear in the list. Adding a task directly via SQLite and restarting the app shows it.

**Commit:**
```
feat(ui): add task list loaded from database
```

---

### Task 5 — Start timer from task row + stop from timer bar

**What you're building:** Wiring the ▶ button in each task row to start a timer, and the ■ button in the timer bar to stop it. Both refresh the UI.

**New concept: shared state and refresh propagation**

The timer bar and the task list both need to react when the timer changes. The simplest approach at this scale: pass a shared `refresh()` function to both widgets. When either changes the timer, it calls `refresh()`, which reloads both the timer bar state and the task list.

```go
// In app.go
refresh := func() {
    timerBar.reload()   // re-reads active timer from store
    taskList.reload()   // re-reads tasks from store
}

// Pass refresh to each widget so they can trigger it
newTimerBar(timerStore, taskStore, refresh)
newTaskList(taskStore, timerStore, refresh)
```

**What to do:**

- In `task_list.go`: wire the ▶ button's `OnTapped` to call `timerStore.Start(task.ID)` then `refresh()`
- In `timer_bar.go`: wire the ■ button's `OnTapped` to call `timerStore.Stop()`, then `taskStore.UpdateStatus(taskID, tasks.TODO)`, then `refresh()`
- Add `UpdateStatus` to `internal/tasks/store.go` (you'll write this as part of this task)
- The timer goroutine should check `timerStore.Active()` — when it returns nil, show the idle state

**What to look up:**
- How to store the current active task ID in the timer bar so `Stop()` knows which task to reset to `todo`
- `widget.Button.SetText()` and `widget.Button.SetIcon()` — how to update a button after it's been created
- Error handling in `OnTapped` — what should happen if `timerStore.Start()` returns an error because a timer is already running?

**Verify:** Click ▶ on a task — timer bar starts showing elapsed time for that task. Click ■ — timer stops, task resets to todo, timer bar returns to idle.

**Commit:**
```
feat(ui): wire start/stop timer between task rows and timer bar
```

---

### Task 6 — Status dot cycling + status bar

**What you're building:** Clicking the status dot on a task cycles `todo → in_progress → done`. A bottom bar showing today's total tracked time and a New Task button.

**New concept: `dialog.ShowForm` for new task input**

Fyne provides a built-in modal form dialog — no need to build a custom window for simple inputs:

```go
nameEntry := widget.NewEntry()
nameEntry.SetPlaceHolder("Task name...")

dialog.ShowForm("New Task", "Add", "Cancel",
    []*widget.FormItem{
        widget.NewFormItem("Name", nameEntry),
    },
    func(confirmed bool) {
        if confirmed && nameEntry.Text != "" {
            taskStore.Add(nameEntry.Text)
            refresh()
        }
    },
    win,  // parent window
)
```

**What to do:**

- In `task_list.go`: add a status dot (a `widget.Button` with no label, icon only) to each row. Its `OnTapped` reads the current status and calls `taskStore.UpdateStatus` with the next status in the cycle.
- Create `internal/ui/status_bar.go`: a horizontal container with a label showing "Today: Xh Xm" (summed from `timerStore.ListAll()` filtered to today) and a "＋ New Task" button that opens the form dialog.
- Connect the status bar to `refresh()` so it recalculates totals after each change.

**What to look up:**
- `dialog` package — `dialog.ShowForm`, `dialog.ShowInformation`, `dialog.ShowError`
- How to calculate today's total from a `[]TimeEntry` slice (sum `Minutes` where `ended_at` is today's date)
- Fyne icons: `theme.MediaPlayIcon()`, `theme.MediaStopIcon()`, `theme.ContentAddIcon()` — the built-in icon set

**Verify:** Click the status dot — it cycles through the three states visually. "＋ New Task" opens a dialog; submitting adds the task to the list.

**Commit:**
```
feat(ui): status dot cycling, new task dialog, today total in status bar
```

---

### Task 7 — Minimise to system tray on close

**What you're building:** Closing the main window hides it rather than quitting. The tray icon has a menu to reopen, stop the timer, and quit.

**New concept: `desktop.App` and `SetCloseIntercept`**

```go
// Intercept the close button — hide instead of quit
win.SetCloseIntercept(func() {
    win.Hide()
})

// System tray — only available on desktop platforms
if desk, ok := myApp.(desktop.App); ok {
    desk.SetSystemTrayMenu(fyne.NewMenu("Track",
        fyne.NewMenuItem("Open", func() { win.Show() }),
        fyne.NewMenuItem("Stop Timer", func() { /* stop + refresh */ }),
        fyne.NewMenuItemSeparator(),
        fyne.NewMenuItem("Quit", myApp.Quit),
    ))
}
```

**What to do:**

- In `app.go`: add `win.SetCloseIntercept` to hide instead of close
- Add tray menu with: Open, Stop Timer (greyed out when no timer running), separator, Quit
- The tray icon should change when a timer is running — use two different icon resources (a plain icon and a "recording" icon). For now, Fyne's built-in `theme.MediaRecordIcon()` works as a placeholder.

**What to look up:**
- `fyne.io/fyne/v2/driver/desktop` package — `desktop.App` interface
- `app.SetIcon(resource)` — how to set and change the tray icon
- `fyne.NewStaticResource(name, data)` — how to embed a custom icon from bytes
- What happens on Linux vs macOS vs Windows when there is no system tray available — does Fyne panic or silently skip?

**Verify:** Close the window — it disappears but the process keeps running (check with `ps`). Tray icon appears. Clicking Open re-shows the window.

**Commit:**
```
feat(tray): minimise to system tray on close with open/stop/quit menu
```

---

### Task 8 — Quick-add window

**What you're building:** A small separate window for quick task entry, with three modes: capture only, start timer, or both.

**New concept: multiple windows + mode switching**

Fyne supports multiple windows in one app. A quick-add window is just a second `fyne.Window` that you `Show()` and `Hide()` rather than creating and destroying:

```go
quickWin := app.NewWindow("Quick Add")
quickWin.Resize(fyne.NewSize(380, 120))
quickWin.SetFixedSize(true)
quickWin.CenterOnScreen()

// Hide instead of close when dismissed
quickWin.SetCloseIntercept(func() { quickWin.Hide() })
```

**What to build:**

`internal/ui/quick_add.go`:
- A `newQuickAddWindow(app, taskStore, timerStore, refresh)` function that returns the window (hidden initially)
- Inside: a `widget.Entry` for the task name, a `widget.RadioGroup` or `widget.Select` for mode (`"Capture task"`, `"Start timer"`, `"Capture + start timer"`)
- `OnSubmitted` on the entry (fires on Enter): executes the selected mode, clears the input, hides the window
- `Escape` key: hides without saving — add a canvas shortcut for `fyne.KeyEscape`

In `app.go`:
- Create the quick-add window once
- Add an in-window shortcut `Ctrl+Enter` to show it
- Expose a `showQuickAdd()` function for the tray menu and the global hotkey (next task)

**What to look up:**
- `win.SetFixedSize(true)` — prevents the user from resizing the quick-add window
- `win.CenterOnScreen()` vs `win.Canvas().Focus(entry)` — how to auto-focus the entry when the window opens
- `widget.RadioGroup` — horizontal layout option

**Verify:** Press `Ctrl+Enter` in the main window — quick-add appears. Type a task name, select a mode, press Enter — window closes, task appears / timer starts based on mode.

**Commit:**
```
feat(ui): quick-add window with capture/timer/both modes
```

---

### Task 9 — Global hotkey

**What you're building:** `Ctrl+Shift+Space` opens the quick-add window from anywhere — even when the app window is hidden in the tray.

**New concept: `golang.design/x/hotkey`**

This library registers a system-level keyboard shortcut that the OS delivers to your process regardless of which app is focused:

```go
hk := hotkey.New([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModShift}, hotkey.KeySpace)
if err := hk.Register(); err != nil {
    log.Println("hotkey registration failed:", err)
    return
}
defer hk.Unregister()

go func() {
    for range hk.Keydown() {
        // This fires on every keypress — must dispatch to the UI thread
        // In Fyne, use: fyne.CurrentApp().Driver().CanvasForObject(...)
        // or simply call showQuickAdd() — Fyne's Show/Hide are safe from goroutines
        showQuickAdd()
    }
}()
```

**What to do:**

1. `go get golang.design/x/hotkey` to re-add the dependency
2. In `app.go`, after creating the quick-add window: register the hotkey in a goroutine that ranges over `hk.Keydown()` and calls `quickWin.Show()` + `quickWin.RequestFocus()`
3. Unregister the hotkey when the app exits — use `app.Lifecycle().SetOnStopped()`

**What to look up:**
- `golang.design/x/hotkey` — `Modifier` and `Key` constants for `Ctrl`, `Shift`, `Space`
- `app.Lifecycle()` — the four lifecycle hooks Fyne provides
- Why the hotkey goroutine must not call `win.SetContent()` or other layout-changing Fyne calls directly (and `Show()`/`Hide()` being the exception)
- Wayland note: hotkey uses X11 under the hood — on pure Wayland without XWayland it will fail to register. Log the error but do not crash.

**Verify:** Hide the main window to tray. Focus a different app. Press `Ctrl+Shift+Space` — quick-add window appears on top.

**Commit:**
```
feat(tray): global hotkey Ctrl+Shift+Space opens quick-add from anywhere
```

---

## What to test after each task

There are no automated tests for UI code — manual verification is the right approach here. After each task:

1. Run `just run`
2. Test the new behaviour described in the "Verify" section
3. Test that previous behaviour still works (regression)
4. Try closing and reopening to ensure state survives a fresh start

---

## Store methods to add (summary)

| Package | Method | Used by |
|---------|--------|---------|
| `tasks` | `UpdateStatus(id int64, status TaskStatus) error` | Task 5 (timer stop), Task 6 (status dot) |
| `tasks` | `ListRecent(days int) ([]Task, error)` | Task 4 (task list) |
| `timer` | No changes needed — `Stop()` already correct | — |

---

## Commit scope reminder

Per `CLAUDE.md`, UI commits use the `ui` scope, tray commits use `tray`, and store additions use the domain scope (`tasks`, `timer`).
