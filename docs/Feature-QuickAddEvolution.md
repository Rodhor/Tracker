# Feature — Quick-Add Evolution

**Date:** 2026-04-02
**Architecture reference:** `docs/Architecture.md`
**Decision record:** `docs/Brainstorm-UIEvolution.md`

---

## What this does

Transforms the quick-add window from a simple capture form into the primary daily workflow surface. One text field that both filters existing tasks and creates new ones. Starting a timer auto-stops the previous one. Stopping without starting a new timer prompts for a note and a status decision.

The entire capture flow should take under 5 seconds.

---

## What changes

| File | Change |
|------|--------|
| `internal/timer/store.go` | `Start()` auto-stops previous timer instead of returning an error |
| `internal/timer/store.go` | Add `StopWithNote(note string) error` |
| `internal/ui/quick_add.go` | Replace mode select with filter/create field + Urgent/Important toggles |
| `internal/ui/time_bar.go` | Stop button triggers stop-prompt modal instead of stopping directly |

---

## Task 1 — Auto-stop on timer start

**File:** `internal/timer/store.go`

**What to change:**

Currently `Start()` returns an error if a timer is already running:
```go
if activeTimer != nil {
    return *activeTimer, fmt.Errorf("other timer is already running")
}
```

Change this to stop the running timer first, then start the new one:

```go
func (s *Store) Start(taskID int64) (TimeEntry, error) {
    // Auto-stop any running timer before starting a new one
    active, err := s.Active()
    if err != nil {
        return TimeEntry{}, fmt.Errorf("failed to check active timer: %w", err)
    }
    if active != nil {
        if err := s.Stop(); err != nil {
            return TimeEntry{}, fmt.Errorf("failed to stop previous timer: %w", err)
        }
    }
    // Now start the new timer
    ...
}
```

**Why this is the right place:**
The auto-stop rule is a business rule ("you can only track one thing at a time"), not a UI concern. It belongs in the store, not in the button handler. Any caller of `Start()` — quick-add, task list ▶ button, tray menu — gets the correct behaviour for free.

**Verify:** Start a timer on Task A. Click ▶ on Task B — Task A's timer stops and Task B's starts. The timer bar shows Task B immediately.

**Commit:**
```
feat(timer): auto-stop previous timer when starting a new one
```

---

## Task 2 — Stop with notes

**File:** `internal/timer/store.go`

Add a new method that stops the active timer and saves a note in one operation:

```go
// Why: Stop() is kept simple for auto-stop (no note needed).
// StopWithNote() is for intentional stops where the user has provided a note.
func (s *Store) StopWithNote(note string) error {
    active, err := s.Active()
    if err != nil {
        return fmt.Errorf("error getting active timer: %w", err)
    }
    if active == nil {
        return fmt.Errorf("no active timer")
    }
    startTime, err := time.Parse(time.RFC3339, active.StartedAt)
    if err != nil {
        return fmt.Errorf("failed to parse start time: %w", err)
    }
    minutes := int64(time.Since(startTime).Minutes())
    stoppedAt := time.Now().UTC().Format(time.RFC3339)

    query := `UPDATE time_entries SET ended_at = ?, minutes = ?, notes = ? WHERE ended_at IS NULL`
    _, err = s.db.Exec(query, stoppedAt, minutes, note)
    if err != nil {
        return fmt.Errorf("failed to stop timer with note: %w", err)
    }
    return nil
}
```

**Note:** This requires the `notes` column migration from `Feature-TaskMetadata.md` (migration 4 — time entries). That migration must be applied before this method is called.

**What to look up:**
- Why two separate methods (`Stop` and `StopWithNote`) are better than one method with an optional note parameter — in Go, optional parameters are expressed via separate functions or an options struct, not default arguments

**Commit:**
```
feat(timer): add StopWithNote method for saving note on manual stop
```

---

## Task 3 — Stop-prompt modal

**File:** `internal/ui/time_bar.go`

**What to change:**

Currently the stop button calls `timerStore.Stop()` directly. Instead, it should open a small modal asking for a note and a status decision.

**New concept: `dialog.ShowCustom`**

For custom modal content that does not fit a form, use `ShowCustom`:

```go
// Why: ShowCustom lets you place any CanvasObject inside a modal dialog
dialog.ShowCustom("Stop Timer", "Stop", content, win)
```

For a modal with a confirm/cancel pair and custom content, use `ShowCustomConfirm`:

```go
dialog.ShowCustomConfirm("Stop Timer", "Stop", "Cancel", content,
    func(confirmed bool) {
        if confirmed {
            // user clicked Stop
        }
    }, win)
```

**What to build in `time_bar.go`:**

```go
stopButton.OnTapped = func() {
    noteEntry := widget.NewEntry()
    noteEntry.SetPlaceHolder("What did you do? (optional)")

    statusSelect := widget.NewSelect(
        []string{"Keep In Progress", "Reset to To Do", "Mark Done"},
        nil,
    )
    statusSelect.SetSelected("Reset to To Do")

    content := container.NewVBox(noteEntry, statusSelect)

    dialog.ShowCustomConfirm("Stop Timer", "Stop", "Cancel", content,
        func(confirmed bool) {
            if !confirmed {
                return
            }
            // Stop the timer with the note
            _ = timerStore.StopWithNote(noteEntry.Text)

            // Update task status based on selection
            if active != nil {
                switch statusSelect.Selected {
                case "Reset to To Do":
                    taskStore.UpdateStatus(active.TaskID, tasks.TODO)
                case "Mark Done":
                    taskStore.UpdateStatus(active.TaskID, tasks.Done)
                // "Keep In Progress" — no change needed
                }
            }
            refresh()
        }, win)
}
```

**Note:** `win` (the parent window) needs to be passed into `newTimerBar`. Update the function signature.

**What to look up:**
- `dialog.ShowCustomConfirm` — parent window parameter is required; dialogs must be anchored to a window
- How to pre-focus the note entry when the dialog opens — `dialog.Dialog.Show()` returns a `*dialog.CustomDialog` which has a `Show()` method you can call before focusing

**Verify:** Click the stop button — a modal appears with a note field and status selector. Clicking Stop saves the note and updates the task status. Clicking Cancel — nothing changes.

**Commit:**
```
feat(ui): stop timer modal with note and status selection
```

---

## Task 4 — Quick-add: filter/create task field

**File:** `internal/ui/quick_add.go`

This is the most significant change to the quick-add window.

**What to build:**

Replace the mode select with a single entry field that does double duty:
- If the text matches an existing task name (case-insensitive prefix match) — a dropdown of matching tasks appears
- The user can select an existing task from the list
- If no match or the user ignores the dropdown — a new task is created on submit

**New concept: `widget.List` as a floating suggestion panel**

Fyne does not have a native autocomplete widget. Build one using a `widget.Entry` and a `widget.List` shown/hidden based on what the user types:

```go
nameEntry := widget.NewEntry()
nameEntry.SetPlaceHolder("Task name or search...")

var suggestions []tasks.Task
suggestionList := widget.NewList(
    func() int { return len(suggestions) },
    func() fyne.CanvasObject { return widget.NewLabel("") },
    func(i widget.ListItemID, o fyne.CanvasObject) {
        o.(*widget.Label).SetText(suggestions[i].Name)
    },
)
suggestionList.Hide()  // hidden until user types

// Update suggestions as the user types
nameEntry.OnChanged = func(text string) {
    if text == "" {
        suggestionList.Hide()
        return
    }
    suggestions = filterTasks(allTasks, text)  // case-insensitive prefix match
    if len(suggestions) > 0 {
        suggestionList.Refresh()
        suggestionList.Show()
    } else {
        suggestionList.Hide()
    }
}
```

When the user selects from the suggestion list, populate `nameEntry` with the selected task name and store the selected task's ID:

```go
var selectedTaskID *int64  // nil = create new task

suggestionList.OnSelected = func(i widget.ListItemID) {
    id := suggestions[i].ID
    selectedTaskID = &id
    nameEntry.SetText(suggestions[i].Name)
    suggestionList.Hide()
}
```

On submit:
```go
submitBTN.OnTapped = func() {
    name := nameEntry.Text
    if name == "" {
        return
    }
    var taskID int64
    if selectedTaskID != nil {
        taskID = *selectedTaskID  // start timer on existing task
    } else {
        newTask, err := taskStore.Add(name)
        if err != nil { return }
        taskID = newTask.ID
    }
    _, _ = timerStore.Start(taskID)  // auto-stops previous timer (Task 1)
    selectedTaskID = nil
    nameEntry.SetText("")
    suggestionList.Hide()
    refresh()
    quickwin.Hide()
}
```

**Layout:**

```go
// Stack the suggestion list below the entry field
// Use a VBox so the list appears inline, not as an overlay
content := container.NewVBox(
    container.NewHBox(nameEntry, urgentCheck, importantCheck),
    suggestionList,
)
quickwin.SetContent(content)
```

**What to look up:**
- `strings.Contains` vs `strings.HasPrefix` for task filtering — prefix match feels more natural for task search
- `strings.ToLower` for case-insensitive matching
- How `widget.List` sizes itself when embedded in a `VBox` — you may need to set `suggestionList.SetMinSize()` to control its height

**New store method required — pass all tasks to quick-add at open time:**

The quick-add window needs the current task list to populate suggestions. The simplest approach: load tasks when the window is shown, not when it is created. Accept a `func() []tasks.Task` getter rather than the store directly, or call `taskStore.ListAll()` inside `newQuickAdd` and refresh it each time the window opens.

**Commit:**
```
feat(ui): quick-add with task filter/create and Urgent/Important toggles
```

---

## Task 5 — Urgent/Important toggles in quick-add

As part of Task 4 above, add two small checkboxes to the quick-add form:

```go
urgentCheck := widget.NewCheck("Urgent", nil)
importantCheck := widget.NewCheck("Important", nil)
```

On submit for new tasks: call `taskStore.UpdateMetadata(newTask.ID, urgentCheck.Checked, importantCheck.Checked, nil)` after `taskStore.Add()`.

For existing tasks: do not change their existing metadata — selecting an existing task and starting a timer should not silently overwrite its priority.

Reset both checkboxes after each submission.

**Verify:** Quick-add with Urgent checked — new task appears in Q3/Q1 in the main list immediately after refresh.

**Commit:**
```
feat(ui): urgent/important checkboxes in quick-add for new tasks
```

---

## Store additions summary

| Method | Signature | Notes |
|--------|-----------|-------|
| `timer.StopWithNote` | `(note string) error` | Saves note to `notes` column — requires migration 4 |

`timer.Start()` is modified, not added.
