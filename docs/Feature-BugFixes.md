# Feature — Bug Fixes

**Date:** 2026-04-02
**Architecture reference:** `docs/Architecture.md`
**Decision record:** `docs/Brainstorm-UIEvolution.md`

---

## What this fixes

Seven defects in the current UI that make the app incorrect or confusing in daily use. All are small, isolated fixes. None require new store methods except one (task name lookup in the timer bar).

Fix these before starting any new feature work — they affect the scaffolding everything else builds on.

---

## Bug 1 — Timer bar shows task ID instead of task name

**File:** `internal/ui/time_bar.go:43`

**What is wrong:**
```go
taskLabel.SetText(fmt.Sprintf("Task ID: %d", active.TaskID))
```
`taskStore` is already passed into `newTimerBar` but never used. The `TimeEntry` only stores a `task_id` — you need to look the task up.

**New store method required — `tasks.Store.GetByID`:**

```go
// Why: timer bar needs the task name given only a task_id from the active time entry
func (s *Store) GetByID(id int64) (Task, error) {
    task := Task{}
    err := s.db.Get(&task, `SELECT * FROM tasks WHERE id = ?`, id)
    if err != nil {
        return Task{}, fmt.Errorf("task not found: %w", err)
    }
    return task, nil
}
```

**Fix in `time_bar.go`:**
Replace the `fmt.Sprintf("Task ID: %d", ...)` line with a call to `taskStore.GetByID(active.TaskID)`. If the lookup fails, fall back to showing the ID as before — do not crash.

**What to look up:**
- `errors.Is(err, sql.ErrNoRows)` — how to distinguish "task not found" from a real DB error

**Verify:** Start a timer — the timer bar shows the task name, not its ID.

---

## Bug 2 — Elapsed display does not reset when timer stops

**File:** `internal/ui/time_bar.go:28–33`

**What is wrong:**
```go
for range ticker.C {
    active, _ := timerStore.Active()
    if active != nil {
        _ = elapsed.Set(computeElapsed(active))  // only updates when active
    }
    // when active == nil: nothing happens — binding freezes at last value
}
```

**Fix:** Add an `else` branch:
```go
if active != nil {
    _ = elapsed.Set(computeElapsed(active))
} else {
    _ = elapsed.Set("00:00:00")
}
```

**Verify:** Start a timer, let it run a few seconds, stop it — display resets to `00:00:00`.

---

## Bug 3 — Status cycle is incomplete: Done tap does nothing

**File:** `internal/ui/task_list.go:38–44`

**What is wrong:**
```go
switch task.Status {
case tasks.TODO:
    taskStore.UpdateStatus(taskID, tasks.InProgress)
case tasks.InProgress:
    taskStore.UpdateStatus(taskID, tasks.Done)
// Done: no case — tapping does nothing
}
```

**Fix:** Add the missing case to cycle `Done → TODO`:
```go
case tasks.Done:
    taskStore.UpdateStatus(taskID, tasks.TODO)
```

**Note:** The cycle is `TODO → InProgress → Done → TODO`. This matches the planned behaviour — the status stop-prompt (a later feature) handles *intentional* Done/TODO decisions on timer stop. The dot cycle is a quick manual override.

**Verify:** Click the confirm button on a Done task — it cycles back to TODO.

---

## Bug 4 — Layout placeholder still visible

**File:** `internal/ui/app.go:55, 74`

**What is wrong:**

The status bar was placed in the *top* HBox alongside the timer bar, and a placeholder label occupies the bottom:
```go
topWidget := container.NewHBox(timerBar, statusbar)   // wrong — status bar is here
bottomWidget := widget.NewLabel("bottomWidget")        // placeholder — still visible
```

**Fix:**

The intended three-zone layout is: timer bar at top, task list in centre, status bar at bottom.

```go
// Why: Border layout pins fixed-height widgets to top/bottom; centre expands to fill
content := container.NewBorder(timerBar, statusbar, nil, nil, centerWidget)
```

Remove `topWidget` and `bottomWidget`. Wire `timerBar` directly to the top slot and `statusbar` directly to the bottom slot of `container.NewBorder`.

**What to look up:**
- The first four arguments of `container.NewBorder` are `top, bottom, left, right` — then the variadic centre. Check the order before editing.

**Verify:** Timer bar is at the top, task list in the middle, today's total and New Task button at the bottom.

---

## Bug 5 — Quick-add: no default mode selected

**File:** `internal/ui/quick_add.go:29`

**What is wrong:**
```go
modeSelect := widget.NewSelect([]string{"Capture Task", "Capture and start Timer"}, nil)
```
No default is set. If the user presses Submit without selecting a mode, the `switch` falls through all cases silently.

**Fix:** Set the default immediately after creating the select:
```go
modeSelect.SetSelected(CaptureAndStart)
```
"Capture and start Timer" is the primary action — make it the default.

**Verify:** Open quick-add — "Capture and start Timer" is pre-selected. Pressing Submit immediately works without manually picking a mode.

---

## Bug 6 — Quick-add: Enter key does not submit

**File:** `internal/ui/quick_add.go`

**What is wrong:**
The `nameEntry` widget has no `OnSubmitted` handler. Pressing Enter in the text field does nothing — the user must click Submit with the mouse.

**New concept: `widget.Entry.OnSubmitted`**

```go
// Why: OnSubmitted fires when the user presses Enter in the field
// It receives the current text as a string — same as entry.Text
nameEntry.OnSubmitted = func(text string) {
    submitBTN.OnTapped()  // reuse the same logic already on the button
}
```

**Fix:** Wire `nameEntry.OnSubmitted` to call `submitBTN.OnTapped()`. This keeps the logic in one place.

**Verify:** Open quick-add, type a task name, press Enter — the task is created/timer starts without clicking.

---

## Bug 7 — Quick-add: input field is not cleared after submission

**File:** `internal/ui/quick_add.go:32–48`

**What is wrong:**
After `submitBTN.OnTapped` runs, `nameEntry.Text` is not cleared. Re-opening the quick-add window shows the previous task name still in the field.

**Fix:** After the store calls in each case, add:
```go
nameEntry.SetText("")
quickwin.Hide()
```

Both cases (Capture and CaptureAndStart) need this. Also call `quickwin.Hide()` after submitting — the window should close itself on success.

**Verify:** Submit a task — window closes. Reopen with Ctrl+Enter — text field is empty.

---

## Store additions summary

| Package | Method | Used by |
|---------|--------|---------|
| `tasks` | `GetByID(id int64) (Task, error)` | Bug 1 — timer bar task name |

---

## Implementation order

Fix in this order — each is independent, but Bug 4 (layout) should come after you can visually verify Bug 1 and Bug 2 are working correctly.

1. Bug 1 — add `GetByID` to task store, fix `time_bar.go`
2. Bug 2 — elapsed reset in goroutine
3. Bug 3 — Done case in status cycle
4. Bug 5, 6, 7 — quick-add trio (small, do together)
5. Bug 4 — layout fix (verify visually with all other fixes in place)

---

## Commit messages (drafts)

```
fix(tasks): add GetByID store method for timer bar name lookup
fix(ui): show task name in timer bar instead of task ID
fix(ui): reset elapsed display to 00:00:00 when timer stops
fix(ui): complete status cycle — Done now cycles back to TODO
fix(ui): correct three-zone layout, remove bottomWidget placeholder
fix(ui): set default mode, wire Enter key, and clear input in quick-add
```
