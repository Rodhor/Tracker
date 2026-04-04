# Phase 3 — Eisenhower Sorting and Task Editing

- **Status:** Planned
- **Created:** 2026-04-04
- **Last updated:** 2026-04-04
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
- [ ] Add `editing_task_id`, `edit_urgent`, `edit_important`, `edit_description` to `App`
- [ ] Initialise edit fields in `new()`
- [ ] Add `OpenEditTask`, `CloseEditTask`, `EditUrgentChanged`, `EditImportantChanged`, `EditDescriptionChanged`, `SaveEditTask` to `Message`
- [ ] Add `checkbox` to imports
- [ ] Implement all 6 new `update()` arms
- [ ] Add `edit_row()` method
- [ ] Add `task_row_or_edit()` helper
- [ ] Add Edit button to `task_row()`
- [ ] Update `task_list()` — sort + split + route to edit panel
- [ ] Verify: sorting, editing, cancel, persist across restart

---

## Technical Notes

*To be filled in after implementation. Say "I finished Phase 3" to trigger this.*

---

## Change History

*No changes yet.*
