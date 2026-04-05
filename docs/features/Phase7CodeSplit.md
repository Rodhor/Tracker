# Phase7CodeSplit

- **Status:** Planned
- **Created:** 2026-04-05
- **Last updated:** 2026-04-05
- **Touches:** `src/main.rs`, `src/app.rs`, `src/ui/` (new module)

---

## What It Does

Splits `src/app.rs` into focused modules under `src/ui/`. No behaviour changes. Each view
function moves to its own file. `app.rs` becomes a coordinator only.

---

## Why It Was Built

`app.rs` was approaching 1000 lines with every view function alongside state and update
logic. Every new feature adds more. The split makes each concern independently navigable
and keeps future additions from compounding the mess.

---

## Implementation Checklist

### Initial implementation — 2026-04-05

- [ ] Add `mod ui;` to `src/main.rs`
- [ ] Create `src/ui/mod.rs` with `pub mod` declarations for all 6 ui files (including `style`)
- [ ] Create the 5 empty ui files + `src/ui/style.rs` (empty — used in Phase 13)
- [ ] Change all `App` fields to `pub(crate)` in `src/app.rs`
- [ ] Move `review_screen()`, `ReviewRow`, `build_review_rows()`, `format_hhmm()` → `src/ui/review.rs`
- [ ] Move `stop_prompt_view()` → `src/ui/stop_prompt.rs`
- [ ] Move `status_bar()`, `today_total_minutes()` → `src/ui/status_bar.rs`
- [ ] Move `task_list()`, `task_row_or_edit()`, `task_row()`, `edit_row()`, `delete_task_confirm_row()` →
  `src/ui/task_list.rs`
- [ ] Move `timer_bar()`, `elapsed_display()` → `src/ui/timer_bar.rs`
- [ ] Update `view()` in `app.rs` to call `crate::ui::*::view(self)`
- [ ] Remove unused imports from `app.rs`
- [ ] `cargo check` — zero errors, zero warnings

---

## Technical Notes

_To be filled in after implementation. Say "I finished Phase7CodeSplit" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
