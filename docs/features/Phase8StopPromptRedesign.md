---
name: Phase 8 — Stop Prompt Redesign
description: Replace three identical status buttons with a pick_list dropdown; pre-fill note from live notes
status: Planned
created: 2026-04-06
last_updated: 2026-04-06
touches: src/data/task.rs, src/ui/stop_prompt.rs, src/app.rs
---

# Phase 8 — Stop Prompt Redesign

- **Status:** Planned
- **Created:** 2026-04-06
- **Last updated:** 2026-04-06
- **Touches:** `src/data/task.rs`, `src/ui/stop_prompt.rs`, `src/app.rs`

---

## What It Does

Replaces the three identical status buttons in the stop prompt with a `pick_list` dropdown.
The note field is pre-filled from any live note written during the session (Phase 9).

---

## Why It Was Built

Three buttons with no visual selection state forced the user to read a "Mark as: X" label
to know what was selected. A pick_list communicates the current selection directly and
reduces the prompt from six rows to four.

---

## Implementation Checklist

### Initial implementation — 2026-04-06

- [ ] Add `Display` impl to `TaskStatus` in `src/data/task.rs`
- [ ] Replace status buttons with `pick_list` in `src/ui/stop_prompt.rs`
- [ ] Add `pick_list` to imports in `stop_prompt.rs`; remove `column` if unused
- [ ] Modify `OpenStopPrompt` arm to pre-fill note from `active_entry.notes`
- [ ] `cargo check` — zero errors

---

## Technical Notes

_Fill in after implementation. Say "I finished Phase 8" to trigger this._

---

## Change History

_No changes yet._
