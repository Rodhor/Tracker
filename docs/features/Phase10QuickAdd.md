---
name: Phase 10 — Quick-Add Filter Modal
description: Modal that filters tasks or creates a new one; arrow key navigation; starts timer on confirm
status: Planned
created: 2026-04-06
last_updated: 2026-04-06
touches: src/message.rs, src/app.rs, src/ui/quick_add.rs (new), src/ui/mod.rs
---

# Phase 10 — Quick-Add Filter Modal

- **Status:** Planned
- **Created:** 2026-04-06
- **Last updated:** 2026-04-06
- **Touches:** `src/message.rs`, `src/app.rs`, `src/ui/quick_add.rs` (new), `src/ui/mod.rs`

---

## What It Does

Opens a modal with a single text input that filters existing tasks in real time. Arrow keys
move a cursor through the filtered list. Enter starts the timer for the selected task. If
the input does not match any task, Enter creates a new task and starts it. Escape closes
without action. Mouse clicks on result rows start immediately.

---

## Why It Was Built

The main tracker screen requires finding a task by eye, then clicking Start. With many tasks
this becomes slow. Quick-add is keyboard-first: one shortcut opens it, a few characters
narrow the list, Enter starts the timer. The create path means you never have to visit the
task list just to add something.

---

## Implementation Checklist

### Initial implementation — 2026-04-06

- [ ] Add 5 messages to `src/message.rs`: `OpenQuickAdd`, `CloseQuickAdd`, `QuickAddInputChanged(String)`,
  `QuickAddMoveUp`, `QuickAddMoveDown`, `QuickAddConfirm`
- [ ] Add 3 state fields to `App` struct: `quick_add_open: bool`, `quick_add_input: String`, `quick_add_selected: usize`
- [ ] Initialise all three in `App::new()`
- [ ] Implement 5 new `update()` arms
- [ ] Add modal-close to the top of the existing `StartTimer` arm
- [ ] Update `view()` — extend the tuple match to 3 booleans, add `quick_add` branch
- [ ] Create `src/ui/quick_add.rs`
- [ ] Add `pub mod quick_add;` to `src/ui/mod.rs`
- [ ] Update `subscription()` — extend `event::listen_with` for arrow keys and Escape (`CloseActiveModal`)
- [ ] Add `CloseActiveModal` message and arm (may be done in Phase 11 instead)
- [ ] `cargo check` — zero errors
- [ ] Manual test: filter, navigate, start; create new; Escape; mouse click; Ctrl+Enter in text input

---

## Technical Notes

_Fill in after implementation. Say "I finished Phase 10" to trigger this._

---

## Change History

_No changes yet._
