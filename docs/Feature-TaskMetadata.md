# Feature — Task Metadata (Eisenhower + Description)

**Date:** 2026-04-02
**Architecture reference:** `docs/Architecture.md`
**Decision record:** `docs/Brainstorm-UIEvolution.md`

---

## What this does

Adds two new fields to tasks: **Eisenhower priority** (urgent + important flags) and a **description** (what the task is). The task list then sorts by Eisenhower quadrant, and tasks with neither flag set are separated into a "Needs sorting" section.

This is a data-first feature — migrations and model changes come before any UI work.

---

## What changes at each layer

| Layer | Change |
|-------|--------|
| `internal/db/db.go` | New migration — ALTER TABLE tasks |
| `internal/tasks/model.go` | Add `Urgent`, `Important`, `Description` fields + `Quadrant()` helper |
| `internal/tasks/store.go` | Update `Add()` signature; add `UpdateMetadata()` method |
| `internal/ui/task_list.go` | Sort tasks before rendering; split into sorted and unsorted sections |
| `internal/ui/status_bar.go` | New Task dialog gains Urgent/Important checkboxes + Description field |
| `internal/ui/quick_add.go` | Add Urgent/Important toggles (later task, after quick-add evolution) |

---

## Task 1 — Database migration

**File:** `internal/db/db.go`

**New concept: ALTER TABLE for SQLite**

SQLite's `ALTER TABLE` supports adding columns but not removing or reordering them. The correct approach is to append a new migration step — never edit the existing schema strings.

```go
// Migration 3 — task metadata
// Why: added as a separate const and exec so earlier migrations are never touched
const taskMetaMigration string = `
ALTER TABLE tasks ADD COLUMN urgent      INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks ADD COLUMN important   INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks ADD COLUMN description TEXT;
`
```

**Important SQLite note:** SQLite does not support running multiple `ALTER TABLE` statements in a single `db.Exec()` call. You must execute each statement separately:

```go
_, err = db.Exec(`ALTER TABLE tasks ADD COLUMN urgent INTEGER NOT NULL DEFAULT 0`)
_, err = db.Exec(`ALTER TABLE tasks ADD COLUMN important INTEGER NOT NULL DEFAULT 0`)
_, err = db.Exec(`ALTER TABLE tasks ADD COLUMN description TEXT`)
```

**Problem: running this on an existing database**

If the columns already exist (because the user has run the app before), SQLite will return `"duplicate column name: urgent"`. The fix is to check whether the migration is needed first:

```go
// Why: SQLite has no IF NOT EXISTS for ALTER TABLE — check manually
func columnExists(db *sqlx.DB, table, column string) bool {
    var count int
    db.QueryRow(
        `SELECT COUNT(*) FROM pragma_table_info(?) WHERE name = ?`,
        table, column,
    ).Scan(&count)
    return count > 0
}

// Then in Init():
if !columnExists(db, "tasks", "urgent") {
    db.Exec(`ALTER TABLE tasks ADD COLUMN urgent INTEGER NOT NULL DEFAULT 0`)
    db.Exec(`ALTER TABLE tasks ADD COLUMN important INTEGER NOT NULL DEFAULT 0`)
    db.Exec(`ALTER TABLE tasks ADD COLUMN description TEXT`)
}
```

**What to look up:**
- `PRAGMA table_info(table_name)` — SQLite's way to inspect columns
- Why SQLite's `ALTER TABLE` is more limited than other databases

**Verify:** Run the app against an existing database — no crash. Check the DB with a SQLite browser — three new columns exist on the tasks table with correct defaults.

**Commit:**
```
feat(db): migration 3 — add urgent, important, description to tasks
```

---

## Task 2 — Update the Task model

**File:** `internal/tasks/model.go`

**What to add:**

```go
type Task struct {
    ID          int64      `db:"id"`
    Name        string     `db:"name"`
    Status      TaskStatus `db:"status"`
    CreatedAt   string     `db:"created_at"`
    Urgent      bool       `db:"urgent"`
    Important   bool       `db:"important"`
    Description *string    `db:"description"`  // pointer — NULL in DB maps to nil in Go
}
```

**New concept: pointer fields for nullable columns**

SQLite columns that allow NULL must be scanned into pointer types in Go. If you use a plain `string` for a nullable TEXT column and the value is NULL, `sqlx` will error. Use `*string` — when the DB value is NULL, Go gets `nil`; when it has a value, Go gets a pointer to it.

```go
// Accessing a nullable field safely:
if task.Description != nil {
    fmt.Println(*task.Description)
}
```

**Add a `Quadrant()` helper:**

```go
// Why: centralises the Eisenhower logic — UI code never needs to check both flags
func (t Task) Quadrant() int {
    switch {
    case t.Urgent && t.Important:
        return 1  // Do First
    case !t.Urgent && t.Important:
        return 2  // Schedule
    case t.Urgent && !t.Important:
        return 3  // Delegate
    default:
        return 4  // Eliminate
    }
}

// HasPriority returns false if neither flag is set — used to find unsorted tasks
func (t Task) HasPriority() bool {
    return t.Urgent || t.Important
}
```

**Verify:** The model compiles. Existing store methods still work — `ListAll()` will now scan the new columns automatically because it uses `SELECT *`.

**Commit:**
```
feat(tasks): add Urgent, Important, Description fields and Quadrant helper
```

---

## Task 3 — Update the task store

**File:** `internal/tasks/store.go`

**What to change:**

`Add()` currently only takes a name. Keep its signature simple — new tasks start with no priority and no description. They go into the "Needs sorting" section by default, which is intentional.

Add a new method for updating metadata:

```go
// Why: separate from Add() so the creation flow stays fast and minimal
// Called from the edit dialog after a task is created
func (s *Store) UpdateMetadata(id int64, urgent, important bool, description *string) error {
    query := `UPDATE tasks SET urgent = ?, important = ?, description = ? WHERE id = ?`
    _, err := s.db.Exec(query, urgent, important, description, id)
    if err != nil {
        return fmt.Errorf("failed to update task metadata: %w", err)
    }
    return nil
}
```

**What to look up:**
- How `sqlx` handles passing a `*string` (nil pointer) as a query argument — it maps to SQL NULL correctly

**Verify:** Call `UpdateMetadata` from a test in `main.go` temporarily. Inspect the DB — row updated correctly.

**Commit:**
```
feat(tasks): add UpdateMetadata store method for Eisenhower fields and description
```

---

## Task 4 — Eisenhower sorting in the task list

**File:** `internal/ui/task_list.go`

**What to build:**

The task list needs to sort tasks before rendering and visually separate "sorted" from "unsorted" tasks.

**New concept: sorting a slice in Go**

```go
import "sort"

// Why: sort.Slice reorders in-place using a comparison function you provide
sort.Slice(tasks, func(i, j int) bool {
    qi := tasks[i].Quadrant()
    qj := tasks[j].Quadrant()
    if qi != qj {
        return qi < qj  // Q1 before Q2 before Q3 before Q4
    }
    return tasks[i].Name < tasks[j].Name  // alphabetical within same quadrant
})
```

**Approach for "Needs sorting" section:**

Split the loaded tasks into two slices after sorting:

```go
var sorted, unsorted []tasks.Task
for _, t := range currentTasks {
    if t.HasPriority() {
        sorted = append(sorted, t)
    } else {
        unsorted = append(unsorted, t)
    }
}
```

Then render them in the list with a separator row between them. The separator row is a special case in `UpdateItem` — when the item index equals `len(sorted)`, render a label `"— Needs sorting —"` instead of a task row.

```go
// Combine into one slice with a sentinel Task{ID: -1} as the divider
allRows := append(sorted, Task{ID: -1})  // sentinel
allRows = append(allRows, unsorted...)

list := widget.NewList(
    func() int { return len(allRows) },
    func() fyne.CanvasObject { /* return row template */ },
    func(i widget.ListItemID, obj fyne.CanvasObject) {
        if allRows[i].ID == -1 {
            // render section header
        } else {
            // render normal task row
        }
    },
)
```

**What to add to each row:**

Show the Eisenhower quadrant as a small coloured label:

```go
// Q1 = "Q1 · Do First", Q2 = "Q2 · Schedule", etc.
// Only shown if the task HasPriority()
quadrantLabels := map[int]string{
    1: "Q1",
    2: "Q2",
    3: "Q3",
    4: "Q4",
}
```

**What to look up:**
- `sort.Slice` vs `slices.SortFunc` (Go 1.21+) — either works; `slices.SortFunc` is more readable
- How to make a list row visually different (e.g. a section header) — the template must accommodate both layouts, or use a `container.NewStack` with visibility toggling

**Verify:** Tasks with Q1 appear first, then Q2, Q3, Q4. Tasks with no flags appear below a "Needs sorting" separator. Adding a new task (no priority set) — it appears in the unsorted section.

**Commit:**
```
feat(ui): sort task list by Eisenhower quadrant with Needs Sorting section
```

---

## Task 5 — Edit Eisenhower flags and description from the task list

**What to build:**

A way to set `urgent`, `important`, and `description` on an existing task. The simplest approach: a right-click (secondary tap) on a task row opens a dialog.

**New concept: secondary tap on a list item**

Fyne's `widget.List` does not have a built-in right-click handler on rows. The workaround is to add a transparent overlay button to each row with a secondary tap listener:

```go
// Why: widget.DoubleTappable and widget.SecondaryTappable are interfaces
// A custom widget or a tappable container can implement them
// The simpler approach: add a context menu trigger directly to the row
```

Actually, the simplest approach in Fyne is to use `widget.List.OnSelected` combined with a long-press gesture, or to place a dedicated "edit" icon button in each row that is visually quiet (`LowImportance`).

**Recommended approach for now:** add a small `theme.SettingsIcon()` button at the far right of each task row (low importance, icon only). Tapping it opens a `dialog.ShowForm` with:
- Name entry (pre-filled)
- Description entry (pre-filled if exists)
- Urgent checkbox
- Important checkbox

```go
editBtn := widget.NewButtonWithIcon("", theme.SettingsIcon(), nil)
editBtn.Importance = widget.LowImportance
editBtn.OnTapped = func() {
    // build form items
    desc := ""
    if task.Description != nil {
        desc = *task.Description
    }
    descEntry := widget.NewMultiLineEntry()
    descEntry.SetText(desc)
    urgentCheck := widget.NewCheck("Urgent", nil)
    urgentCheck.SetChecked(task.Urgent)
    importantCheck := widget.NewCheck("Important", nil)
    importantCheck.SetChecked(task.Important)

    dialog.ShowForm("Edit Task", "Save", "Cancel",
        []*widget.FormItem{
            widget.NewFormItem("Description", descEntry),
            widget.NewFormItem("", urgentCheck),
            widget.NewFormItem("", importantCheck),
        },
        func(confirmed bool) {
            if confirmed {
                d := descEntry.Text
                taskStore.UpdateMetadata(taskID, urgentCheck.Checked, importantCheck.Checked, &d)
                refresh()
            }
        }, win)
}
```

**Note:** `win` (the parent window) needs to be passed into `newTaskList` for dialogs to anchor correctly.

**Verify:** Click the settings icon on a task — dialog opens pre-filled. Save — task moves to the correct quadrant in the list.

**Commit:**
```
feat(ui): add edit dialog for Eisenhower flags and description per task
```

---

## Task 6 — New Task dialog gains priority fields

**File:** `internal/ui/status_bar.go`

Add the same Urgent/Important checkboxes and Description entry to the existing New Task dialog. When confirmed, call both `taskStore.Add(name)` and `taskStore.UpdateMetadata(newTask.ID, ...)`.

**Verify:** Creating a new task with Urgent checked — it appears in Q3 or Q1 (depending on Important) in the sorted list immediately.

**Commit:**
```
feat(ui): add Eisenhower fields to new task dialog
```

---

## Store additions summary

| Method | Signature | Used by |
|--------|-----------|---------|
| `tasks.UpdateMetadata` | `(id int64, urgent, important bool, description *string) error` | Task 5, Task 6 |

`ListAll()` requires no changes — `SELECT *` picks up the new columns automatically once the model struct is updated.
