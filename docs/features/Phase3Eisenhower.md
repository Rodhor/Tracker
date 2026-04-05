# Phase 3 — Eisenhower Sorting and Task Editing

- **Status:** Complete
- **Created:** 2026-04-04
- **Last updated:** 2026-04-05
- **Touches:** `src/app.rs`

---

## What It Does

Splits the task list into a priority-sorted section (Q1 → Q4) and a "Needs sorting"
section for tasks that have not been categorised yet. Each task row gains an Edit button
that opens an inline panel for setting urgent/important flags and an optional description.

---

## Why It Was Built

The data model has had Eisenhower fields since Phase 1. This phase surfaces them in the UI
so the user can actually organise their work, not just add and time tasks.

---

## Implementation Checklist

### Initial implementation — 2026-04-04

- [x] Add `editing_task_id`, `edit_urgent`, `edit_important`, `edit_description` to `App`
- [x] Initialise edit fields in `new()`
- [x] Add `OpenEditTask`, `CloseEditTask`, `EditUrgentChanged`, `EditImportantChanged`, `EditDescriptionChanged`,
  `SaveEditTask` to `Message`
- [x] Add `checkbox` to imports
- [x] Implement all 6 new `update()` arms
- [x] Add `edit_row()` method
- [x] Add `task_row_or_edit()` helper
- [x] Add Edit button to `task_row()`
- [x] Update `task_list()` — sort + split + route to edit panel
- [x] Verify: sorting, editing, cancel, persist across restart

---

## Technical Notes

- **Sorting:** `sort_by_key(|t| t.quadrant())` on a `Vec<&AppTask>` collected from the
  filtered iterator. Rust's sort is stable — tasks with the same quadrant keep their
  original insertion order.

- **Section headers:** User chose "TODOS" and "Needs Sorting" as labels. Only "Needs
  Sorting" is shown when there are unsorted tasks; the header for sorted tasks is always
  shown when sorted tasks exist. Both use `iced::Padding::new(8.0).bottom(4)` — the
  correct iced 0.14 asymmetric padding API (`.padding([t, r, b, l])` does not exist).

- **Temporary edit fields pattern:** `edit_urgent`, `edit_important`, `edit_description`
  are copied from the task on `OpenEditTask` and applied to the task on `SaveEditTask`.
  `CloseEditTask` discards them. This pattern ensures Cancel truly reverts changes.

- **`task_row_or_edit()` helper:** Checks `editing_task_id == Some(task.id)` and routes
  to `edit_row()` or `task_row()` accordingly. Extracted to keep `task_list()` readable.

- **`checkbox(value).label("...").on_toggle(fn)`:** In iced 0.14, `checkbox` takes the
  bool value as the first argument, then `.label()` and `.on_toggle()` are builder methods.
  The plan showed `checkbox(label, value)` — the actual API has them reversed.

- **`Container` import:** The type `Container` was imported alongside `container` (the
  function) and is unused. Generates a warning but does not affect behaviour.

- **Pattern followed:** Same temporary-fields / copy-on-open / apply-on-save pattern
  used here for task editing will be reused in Phase 4 for the stop prompt.

### Design decision — status reset on save

`SaveEditTask` calls `task.status.reset_status()` which returns `TaskStatus::Todo`,
resetting the task's status every time edits are saved. This is intentional: a task
should not be In Progress or Done without having been consciously placed in an
Eisenhower quadrant first. Saving the edit panel is the moment the user commits to a
priority — at that point the task starts fresh in Todo.

---

## Change History

*No changes yet.*
