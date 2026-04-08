# Icons

- **Status:** Planned
- **Created:** 2026-04-08
- **Last updated:** 2026-04-08
- **Touches:** `Cargo.toml`, `src/main.rs`, `src/ui/style.rs`, `src/ui/timer_bar.rs`, `src/ui/task_list.rs`, `src/ui/review.rs`, `src/ui/stop_prompt.rs`, `src/ui/note_modal.rs`, `src/ui/status_bar.rs`, `src/ui/quick_add.rs`

---

## What It Does

Replaces text labels on compact action buttons with Bootstrap Icons via the `iced_fonts` crate. Font bytes are bundled in the crate — no separate `.ttf` file needed.

---

## Why It Was Built

Text-only buttons take more horizontal space and are visually noisy. Icon buttons make the interface denser and more scannable, especially in the review screen where multiple action buttons appear per row.

---

## Implementation Checklist

### Initial implementation — 2026-04-08
- [ ] Add `iced_fonts` to `Cargo.toml`
- [ ] Register `BOOTSTRAP_FONT_BYTES` in `main.rs`
- [ ] Add `icon()` helper to `src/ui/style.rs`
- [ ] Update `timer_bar.rs`
- [ ] Update `task_list.rs`
- [ ] Update `review.rs`
- [ ] Update `stop_prompt.rs`
- [ ] Update `note_modal.rs`
- [ ] Update `status_bar.rs`
- [ ] Update `quick_add.rs`
- [ ] Verify: no empty boxes, layout intact, pause/resume switch correctly

---

## Technical Notes

_To be filled in after implementation._

---

## Change History

_No changes yet._
