# Feature — Review Window

**Date:** 2026-04-02
**Architecture reference:** `docs/Architecture.md`
**Decision record:** `docs/Brainstorm-UIEvolution.md`

---

## What this does

A dedicated second window showing the day's time entries in chronological order. Each entry shows task name, start time, end time, duration, and an editable notes field. Gaps between entries are shown as "untracked" blocks. A summary bar shows the first start, last end, total tracked, and total untracked time. A date navigator lets you look back on previous days.

This is the reporting surface — you open it at the end of the day, read your entries with their notes, and fill in TANNSS.

---

## What changes

| File | Change |
|------|--------|
| `internal/timer/store.go` | Add `ListByDate`, `UpdateNotes`, `Delete` methods |
| `internal/tasks/store.go` | Add `Delete` method |
| `internal/ui/review.go` | New file — the review window |
| `internal/ui/app.go` | Wire review window; add "Review" button to status bar or toolbar |
| `internal/db/db.go` | Migration 4 — add `notes` column to `time_entries` |

---

## Task 1 — Migration 4: notes column on time_entries

**File:** `internal/db/db.go`

Same pattern as migration 3 from `Feature-TaskMetadata.md`:

```go
if !columnExists(db, "time_entries", "notes") {
    db.Exec(`ALTER TABLE time_entries ADD COLUMN notes TEXT`)
}
```

Add this in `Init()` after the migration 3 block.

**Also update `internal/timer/model.go`:**

```go
type TimeEntry struct {
    ID        int64   `db:"id"`
    TaskID    int64   `db:"task_id"`
    StartedAt string  `db:"started_at"`
    EndedAt   *string `db:"ended_at"`
    Minutes   *int64  `db:"minutes"`
    Notes     *string `db:"notes"`     // ← new
}
```

**Verify:** Existing entries load without error — `Notes` will be nil for all existing rows (NULL default). New entries from `StopWithNote()` will have a value.

**Commit:**
```
feat(db): migration 4 — add notes column to time_entries
```

---

## Task 2 — New store methods

**File:** `internal/timer/store.go`

### `ListByDate(date string) ([]TimeEntry, error)`

```go
// Why: the review window always works with one day's entries at a time
// date format: "2006-01-02" (matches the first 10 chars of RFC3339 started_at)
func (s *Store) ListByDate(date string) ([]TimeEntry, error) {
    entries := []TimeEntry{}
    query := `SELECT * FROM time_entries WHERE started_at LIKE ? ORDER BY started_at ASC`
    err := s.db.Select(&entries, query, date+"%")
    if err != nil {
        return nil, fmt.Errorf("failed to list entries for date: %w", err)
    }
    return entries, nil
}
```

**Why `LIKE date+"%"`:** The `started_at` column stores RFC3339 strings like `"2026-04-02T09:15:00Z"`. Matching with `LIKE "2026-04-02%"` finds all entries for that date regardless of time.

### `UpdateNotes(id int64, notes string) error`

```go
func (s *Store) UpdateNotes(id int64, notes string) error {
    _, err := s.db.Exec(`UPDATE time_entries SET notes = ? WHERE id = ?`, notes, id)
    if err != nil {
        return fmt.Errorf("failed to update notes: %w", err)
    }
    return nil
}
```

### `Delete(id int64) error`

```go
func (s *Store) Delete(id int64) error {
    _, err := s.db.Exec(`DELETE FROM time_entries WHERE id = ?`, id)
    if err != nil {
        return fmt.Errorf("failed to delete time entry: %w", err)
    }
    return nil
}
```

**File:** `internal/tasks/store.go`

### `tasks.Store.Delete(id int64) error`

```go
// Why: must delete time entries first (foreign key constraint)
func (s *Store) Delete(id int64) error {
    _, err := s.db.Exec(`DELETE FROM time_entries WHERE task_id = ?`, id)
    if err != nil {
        return fmt.Errorf("failed to delete time entries for task: %w", err)
    }
    _, err = s.db.Exec(`DELETE FROM tasks WHERE id = ?`, id)
    if err != nil {
        return fmt.Errorf("failed to delete task: %w", err)
    }
    return nil
}
```

**What to look up:**
- Why you must delete child rows (time_entries) before parent rows (tasks) when a foreign key exists — SQLite enforces this if `PRAGMA foreign_keys = ON` is set
- Whether to enable `PRAGMA foreign_keys = ON` in `db.Init()` — SQLite disables FK enforcement by default

**Commit:**
```
feat(timer): add ListByDate, UpdateNotes, Delete store methods
feat(tasks): add Delete store method with cascade to time entries
```

---

## Task 3 — Review window skeleton

**File:** `internal/ui/review.go`

**What to build:**

A new `fyne.Window` with a three-zone layout:
- Top: date navigation (← label →)
- Centre: scrollable list of entries
- Bottom: summary bar

```go
func newReviewWindow(app fyne.App, timerStore *timer.Store, taskStore *tasks.Store) (fyne.Window, func()) {
    win := app.NewWindow("Daily Review")
    win.Resize(fyne.NewSize(600, 500))
    win.SetCloseIntercept(func() { win.Hide() })

    currentDate := time.Now().Format("2006-01-02")

    // Date navigation
    dateLabel := widget.NewLabel(currentDate)
    prevBtn := widget.NewButtonWithIcon("", theme.NavigateBackIcon(), func() {
        d, _ := time.Parse("2006-01-02", currentDate)
        currentDate = d.AddDate(0, 0, -1).Format("2006-01-02")
        reload()
    })
    nextBtn := widget.NewButtonWithIcon("", theme.NavigateNextIcon(), func() {
        d, _ := time.Parse("2006-01-02", currentDate)
        currentDate = d.AddDate(0, 0, 1).Format("2006-01-02")
        reload()
    })
    dateNav := container.NewHBox(prevBtn, dateLabel, nextBtn)

    // Summary bar
    summaryLabel := widget.NewLabel("Loading...")

    var reload func()
    reload = func() {
        // load entries, rebuild list, update summary
    }

    content := container.NewBorder(dateNav, summaryLabel, nil, nil, /* entry list */)
    win.SetContent(content)

    show := func() {
        currentDate = time.Now().Format("2006-01-02")
        reload()
        win.Show()
    }
    return win, show
}
```

**Verify:** The window opens, shows today's date, prev/next buttons change the date.

**Commit:**
```
feat(ui): review window skeleton with date navigation
```

---

## Task 4 — Entry list with gap rows

**New concept: building a mixed list with gap rows**

The review window shows two types of rows interleaved:
- **Entry rows** — a real time entry (task name, start, end, duration, notes)
- **Gap rows** — an untracked block between two entries ("⊘ Untracked — 32 min")

Build a `reviewRow` type to unify them:

```go
type reviewRow struct {
    entry    *timer.TimeEntry  // nil for gap rows
    taskName string
    gapDur   time.Duration     // only meaningful when entry == nil
}
```

**Building the row list from entries:**

```go
func buildRows(entries []timer.TimeEntry, taskNames map[int64]string) []reviewRow {
    var rows []reviewRow
    for i, e := range entries {
        rows = append(rows, reviewRow{entry: &entries[i], taskName: taskNames[e.TaskID]})

        // Check for gap to next entry
        if i+1 < len(entries) && e.EndedAt != nil {
            end, _ := time.Parse(time.RFC3339, *e.EndedAt)
            nextStart, _ := time.Parse(time.RFC3339, entries[i+1].StartedAt)
            gap := nextStart.Sub(end)
            if gap > time.Minute {  // only show gaps longer than 1 minute
                rows = append(rows, reviewRow{gapDur: gap})
            }
        }
    }
    return rows
}
```

**Entry row layout:**

```
[Task name]          [09:15 → 10:45]  [1h 30m]
[Notes field — editable inline                ]
```

Use a `container.NewVBox` for the two lines, inside a `container.NewBorder` for the top line.

**Gap row layout:**

```
⊘  Untracked — 32 min
```

Rendered as a `widget.Label` with muted styling (use a custom rich text or just a label with `Importance = widget.LowImportance` — labels do not have importance, so set a small text size or use `widget.RichText`).

**What to look up:**
- `widget.RichText` and `widget.RichTextSegment` — for styled inline text (muted colour for gap rows)
- `time.Duration.Hours()`, `.Minutes()` — for formatting durations as "1h 30m"
- How to get task names efficiently — load all tasks once with `taskStore.ListAll()` and build a `map[int64]string` (id → name) rather than calling `GetByID` per row

**Commit:**
```
feat(ui): review entry list with interleaved gap rows
```

---

## Task 5 — Inline notes editing

**New concept: entry widget that saves on blur**

Rather than a Save button per row, save when the user clicks away (blur):

```go
notesEntry := widget.NewEntry()
notesEntry.SetPlaceHolder("Add notes...")
if e.Notes != nil {
    notesEntry.SetText(*e.Notes)
}

// OnChanged fires on every keystroke — too frequent for DB writes
// Instead, save on the FocusLost event

// Fyne does not expose OnFocusLost directly on widget.Entry.
// The clean approach: use a custom focusable entry or save on window-level events.
// Simplest working approach: save on OnSubmitted (Enter key) and add a "save" icon button.
notesEntry.OnSubmitted = func(text string) {
    timerStore.UpdateNotes(entryID, text)
}
```

For the "save on blur" behaviour specifically, the cleanest Fyne approach is to add a small save button that appears when the field has been edited (track a `dirty` flag in `OnChanged`).

**What to look up:**
- `fyne.Focusable` interface — `FocusGained()` and `FocusLost()` — to build a custom focusable entry that saves on blur
- Whether wrapping `widget.Entry` in a custom struct is worthwhile here, or whether Enter-to-save is sufficient

**Verify:** Type in a notes field, press Enter — the note persists across window close and reopen.

**Commit:**
```
feat(ui): inline note editing in review window, saved on Enter
```

---

## Task 6 — Delete time entry

Add a delete button to each entry row in the review window:

```go
deleteBtn := widget.NewButtonWithIcon("", theme.DeleteIcon(), nil)
deleteBtn.Importance = widget.DangerImportance
deleteBtn.OnTapped = func() {
    dialog.ShowConfirm("Delete entry",
        "Delete this time entry? This cannot be undone.",
        func(confirmed bool) {
            if confirmed {
                timerStore.Delete(entryID)
                reload()
            }
        }, win)
}
```

**Verify:** Click delete on an entry — confirmation dialog appears. Confirm — entry disappears from the list.

**Commit:**
```
feat(ui): delete time entry from review window with confirmation
```

---

## Task 7 — Summary bar

At the bottom of the review window, show a one-line summary:

```
First: 09:15  ·  Last: 17:42  ·  Tracked: 6h 15m  ·  Untracked: 2h 12m
```

Calculate from the loaded `[]reviewRow`:
- **First:** `started_at` of the first entry
- **Last:** `ended_at` of the last completed entry
- **Tracked:** sum of `minutes` for all completed entries
- **Untracked:** sum of all gap durations

```go
func buildSummary(rows []reviewRow) string {
    // ... walk rows, accumulate tracked minutes and gap durations
}
```

**Verify:** Summary line updates when navigating to different dates. Shows "No entries" when the date has no data.

**Commit:**
```
feat(ui): summary bar in review window — first, last, tracked, untracked
```

---

## Task 8 — Wire review window into the app

**File:** `internal/ui/app.go` and `internal/ui/status_bar.go`

- Create the review window once in `app.go`: `reviewWin, showReview := newReviewWindow(app, timerStore, taskStore)`
- Add a "Review" button to the status bar (alongside "New Task"):
  ```go
  widget.NewButtonWithIcon("Review", theme.ListIcon(), func() { showReview() })
  ```
- Add "Review" to the tray menu as well

**Verify:** Clicking Review in the status bar or tray opens the review window. Closing it hides it (SetCloseIntercept). Reopening shows the current date's entries fresh.

**Commit:**
```
feat(ui): wire review window into status bar and tray menu
```

---

## Task 9 — Delete task from main window

**File:** `internal/ui/task_list.go`

Add a delete option to the task row edit dialog (built in `Feature-TaskMetadata.md`, Task 5). Below the form fields, add a "Delete Task" button with `DangerImportance`:

```go
dialog.ShowConfirm("Delete task",
    fmt.Sprintf("Delete '%s' and all its time entries?", task.Name),
    func(confirmed bool) {
        if confirmed {
            taskStore.Delete(task.ID)
            refresh()
        }
    }, win)
```

**Verify:** Deleting a task removes it from the list and removes its time entries from the review window.

**Commit:**
```
feat(ui): delete task with confirmation from task list
```

---

## Store additions summary

| Package | Method | Signature |
|---------|--------|-----------|
| `timer` | `ListByDate` | `(date string) ([]TimeEntry, error)` |
| `timer` | `UpdateNotes` | `(id int64, notes string) error` |
| `timer` | `Delete` | `(id int64) error` |
| `tasks` | `Delete` | `(id int64) error` (cascades to time_entries) |
