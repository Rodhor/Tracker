---
name: TaskMetadata
description: Eisenhower priority fields (urgent/important), task description, migration 3, Eisenhower sorting in task list with Needs Sorting section
type: feature
---

# TaskMetadata

- **Status:** Planned
- **Created:** 2026-04-02
- **Last updated:** 2026-04-02
- **Touches:** `internal/db/db.go`, `internal/tasks/model.go`, `internal/tasks/store.go`, `internal/ui/task_list.go`, `internal/ui/status_bar.go`

---

## What It Does

Adds `urgent` and `important` boolean fields and a `description` text field to tasks. The task list sorts by Eisenhower quadrant (Q1 Do First → Q2 Schedule → Q3 Delegate → Q4 Eliminate). Tasks with neither flag set appear in a separate "Needs sorting" section. An edit dialog on each task row allows setting all metadata.

---

## Why It Was Built

Without priority context, all tasks look equal. The Eisenhower matrix gives a fast, low-friction way to express what actually matters vs. what is just noise. The "Needs sorting" section ensures that newly captured tasks surface clearly rather than getting lost.

---

## Implementation Checklist

### Initial implementation — 2026-04-02

- [ ] Migration 3: `ALTER TABLE tasks` — add `urgent`, `important`, `description` columns with `columnExists` guard
- [ ] Update `tasks/model.go` — add `Urgent bool`, `Important bool`, `Description *string` fields
- [ ] Add `Quadrant()` and `HasPriority()` helpers to `Task`
- [ ] Add `tasks.Store.UpdateMetadata(id, urgent, important bool, description *string) error`
- [ ] Sort tasks by `Quadrant()` in `task_list.go` before rendering
- [ ] Split tasks into `sorted` and `unsorted` slices; insert sentinel row as divider
- [ ] Render section header row for the "Needs sorting" divider
- [ ] Add quadrant indicator label to each sorted task row
- [ ] Add edit icon button (`theme.SettingsIcon()`, `LowImportance`) to each task row
- [ ] Edit dialog: Description entry, Urgent checkbox, Important checkbox — pre-filled
- [ ] On edit save: call `UpdateMetadata`, then `refresh()`
- [ ] Add Urgent/Important checkboxes and Description field to New Task dialog in `status_bar.go`
- [ ] On new task: call `UpdateMetadata` after `Add()` if any fields set

---

## Technical Notes

_To be filled in after implementation. Say "I finished TaskMetadata" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
