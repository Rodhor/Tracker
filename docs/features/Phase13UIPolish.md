---
name: Phase 13 — UI Polish
description: Separators, button hierarchy, active task highlight, muted header, note modal focus, window size
type: project
---

# Phase 13 — UI Polish

- **Status:** Planned
- **Created:** 2026-04-07
- **Last updated:** 2026-04-07
- **Touches:** `src/ui/style.rs`, `src/main.rs`, `src/app.rs`, `src/ui/task_list.rs`, `src/ui/stop_prompt.rs`, `src/ui/note_modal.rs`, `src/ui/quick_add.rs`, `src/ui/review.rs`

---

## What It Does

Visual improvements that make the app feel finished. No logic changes.

---

## Why It Was Built

All functionality is complete after Phase 12. This pass makes the UI scannable and consistent
before the app goes into daily use.

---

## Implementation Checklist

### Initial implementation — 2026-04-07

- [ ] Define `active_row` container style in `src/ui/style.rs`
- [ ] Set window size / minimum size in `src/main.rs`
- [ ] Add `rule::horizontal(1)` separators in `src/app.rs` between the three zones
- [ ] Highlight active task row in `src/ui/task_list.rs` — pass `is_active: bool` to `task_row`
- [ ] Mute "Needs Sorting" header text color in `src/ui/task_list.rs`
- [ ] Apply `button::primary` / `button::text` in `src/ui/stop_prompt.rs`
- [ ] Apply `button::primary` / `button::text` in `src/ui/note_modal.rs`
- [ ] Apply `button::primary` / `button::text` / `button::danger` in `src/ui/task_list.rs`
- [ ] Apply `button::danger` / `button::primary` / `button::text` in `src/ui/review.rs`
- [ ] Apply `button::text` / `button::primary` in `src/ui/quick_add.rs`
- [ ] Add `NOTE_MODAL_ID` const and `.id()` to text_editor in `src/ui/note_modal.rs`
- [ ] Return `operation::focus(NOTE_MODAL_ID)` from `OpenNoteModal` in `src/app.rs`
- [ ] `cargo check` — zero errors
- [ ] Visual check of all 8 items listed in the verify section

---

## Technical Notes

_Fill in after implementation. Say "I finished Phase 13" to trigger this._

---

## Change History

_No changes yet._
