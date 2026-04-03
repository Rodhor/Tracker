---
name: FyneUI
description: Native desktop UI using Fyne — replaces the web server UI with a taskbar app, system tray, and global hotkey quick-add
type: feature
---

# FyneUI

- **Status:** Planned
- **Created:** 2026-03-29
- **Last updated:** 2026-03-29
- **Touches:** `internal/ui/`, `cmd/main.go`, `internal/tasks/store.go`

---

## What It Does

Provides the entire user interface for tasktracker as a native Fyne desktop window. Shows the active timer in a fixed top bar, a scrollable task list with per-row start/stop controls, and a status bar with today's totals. Minimises to the system tray on close. A separate quick-add window can be opened by global hotkey from any app.

---

## Why It Was Built

The previous plan used a Go HTTP server + browser UI. That required starting the server in a terminal before the app was usable. This feature replaces it with a proper desktop app: launch from the taskbar, no terminal needed, timer keeps running in the background when the window is hidden.

---

## Implementation Checklist

### Initial implementation — 2026-03-29

#### Store additions (prerequisite)
- [x] `tasks.Store.UpdateStatus(id, status)` — used by status dot and timer stop
- [ ] `tasks.Store.ListRecent(days)` — returns active tasks + recently completed ones

#### Task 1 — Fyne dependency + blank window
- [x] Add `fyne.io/fyne/v2` dependency
- [x] Create `internal/ui/app.go` with `Run(taskStore, timerStore)` function
- [x] Update `cmd/main.go` to call `ui.Run()` instead of test code

#### Task 2 — Window layout skeleton
- [x] Three-zone `container.NewBorder` layout (timer bar / task list / status bar)
- [x] Placeholder labels in each zone

#### Task 3 — Live timer bar
- [x] `internal/ui/timer_bar.go` with elapsed binding + goroutine
- [x] Idle and running states (label, button icon/colour change)

#### Task 4 — Task list
- [x] `internal/ui/task_list.go` using `widget.List` callbacks
- [x] Load from `taskStore.ListRecent(3)`
- [x] Row layout: name (left) + ▶ button (right, placeholder)

#### Task 5 — Start/stop timer wiring
- [x] ▶ button calls `timerStore.Start(taskID)` + `refresh()`
- [x] ■ button calls `timerStore.Stop()` + `taskStore.UpdateStatus(todo)` + `refresh()`
- [x] Shared `refresh()` function in `app.go`

#### Task 6 — Status dot cycling + status bar
- [x] Status dot cycles todo → in_progress → done
- [x] `internal/ui/status_bar.go` — today's total + New Task button
- [x] `dialog.ShowForm` for new task input

#### Task 7 — Minimise to system tray
- [x] `win.SetCloseIntercept` hides instead of quitting
- [x] Tray menu: Open / Stop Timer / Quit
- [x] Tray icon changes when timer is running

#### Task 8 — Quick-add window
- [x] `internal/ui/quick_add.go` — separate `fyne.Window`
- [x] Two modes: capture / capture and start timer
- [x] `Ctrl+Enter` in-window shortcut to open it

#### Task 9 — Global hotkey
- [ ] `go get golang.design/x/hotkey`
- [ ] `Ctrl+Shift+Space` opens quick-add from any app
- [ ] Unregister on app stop via `app.Lifecycle().SetOnStopped()`

---

## Technical Notes

_To be filled in after implementation. Say "I finished FyneUI" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
