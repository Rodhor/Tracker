# Phase6Delete

- **Status:** Planned
- **Created:** 2026-04-05
- **Last updated:** 2026-04-05
- **Touches:** `src/app.rs`

---

## What It Does

Adds inline delete to two places: time entries in the review view, and tasks in the tracker
edit panel. Clicking Delete transforms the row into a confirmation state (task name + entry
count warning + Confirm / Cancel). Confirming removes the item permanently. Deleting a task
cascades to all its time entries. The active timer is discarded if its task is deleted.

---

## Why It Was Built

The review is the TANNSS reporting surface. Mistakes — wrong task attribution, overnight
timers, test tasks — need to be cleanly removable before reporting. Without delete, errors
are permanent and accumulate.

---

## Implementation Checklist

### Initial implementation — 2026-04-05

**State (Step 1)**

- [ ] Add `deleting_entry_id: Option<Uuid>` to `App`
- [ ] Add `deleting_task_id: Option<Uuid>` to `App`
- [ ] Initialise both to `None` in `new()`

**Messages (Step 2)**

- [ ] Add `RequestDeleteEntry(Uuid)`, `ConfirmDeleteEntry`, `CancelDeleteEntry`
- [ ] Add `RequestDeleteTask(Uuid)`, `ConfirmDeleteTask`, `CancelDeleteTask`

**update() arms (Step 3)**

- [ ] `RequestDeleteEntry` — clear `editing_note_id`, set `deleting_entry_id`
- [ ] `ConfirmDeleteEntry` — `entries.retain(...)`, clear state, save
- [ ] `CancelDeleteEntry` — clear `deleting_entry_id`
- [ ] `RequestDeleteTask` — close edit panel, set `deleting_task_id`
- [ ] `ConfirmDeleteTask` — discard active timer if task matches, cascade entries, remove task, save
- [ ] `CancelDeleteTask` — clear `deleting_task_id`

**Tracker view (Steps 4–6)**

- [ ] Add Delete button to first row of `edit_row()`
- [ ] Add `delete_task_confirm_row()` method — shows warning with entry count
- [ ] Update `task_row_or_edit()` — check `deleting_task_id` first, then `editing_task_id`

**Review view (Step 7)**

- [ ] Add Delete button to entry rows (`on_press_maybe` — disabled for active entry)
- [ ] When `deleting_entry_id == Some(entry.id)`, show confirmation row and `continue`

**Conflict clearing (Step 8)**

- [ ] `OpenEditNote` arm: clear `deleting_entry_id` at start
- [ ] `OpenEditTask` arm: clear `deleting_task_id` at start

**Verify (Step 9)**

- [ ] Entry delete: confirmation shows, Cancel restores, Confirm removes and persists
- [ ] Active entry Delete button is non-interactive
- [ ] Task delete: confirmation shows entry count, Cancel returns to edit panel
- [ ] Task delete cascade: entries gone from review after confirm
- [ ] Active timer discarded when its task is deleted; stop prompt closed if open
- [ ] All deletions survive restart

---

## Technical Notes

_To be filled in after implementation. Say "I finished Phase6Delete" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
