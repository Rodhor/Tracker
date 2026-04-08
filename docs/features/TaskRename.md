# Task Rename

- **Status:** Planned
- **Created:** 2026-04-08
- **Last updated:** 2026-04-08
- **Touches:** `src/app.rs`, `src/message.rs`, `src/ui/task_list.rs`

---

## What It Does

Allows the user to rename a task from within the existing edit panel. The task name field changes from static text to an editable input, following the same temporary-state pattern used by all other editable task fields.

---

## Why It Was Built

Task names could never be changed after creation. The edit panel existed but only exposed urgent/important flags and description.

---

## Implementation Checklist

### Initial implementation — 2026-04-08
- [ ] Add `edit_task_name: String` to `App` struct and initialise in `new()`
- [ ] Add `EditTaskNameChanged(String)` to `Message`
- [ ] Populate `edit_task_name` in `OpenEditTask`
- [ ] Clear `edit_task_name` in `CloseEditTask`
- [ ] Restore `edit_task_name` in `CancelDeleteTask`
- [ ] Add `EditTaskNameChanged` arm in `update()`
- [ ] Write name back (with empty-guard) in `SaveEditTask`, then clear field
- [ ] Replace `text(&task.name)` with `text_input` in `edit_row`
- [ ] Verify: pre-fill, save, empty-guard, cancel, delete-then-cancel

---

## Technical Notes

_To be filled in after implementation._

---

## Change History

_No changes yet._
