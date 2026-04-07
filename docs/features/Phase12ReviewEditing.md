---
name: Phase 12 — Entry Time Editing + Copy Button
description: Copy note to clipboard; edit start/end times of completed review entries with HH:MM inputs
type: project
---

# Phase 12 — Entry Time Editing + Copy Button

- **Status:** Planned
- **Created:** 2026-04-07
- **Last updated:** 2026-04-07
- **Touches:** `src/message.rs`, `src/app.rs`, `src/ui/review.rs`

---

## What It Does

Two additions to the review screen. A Copy button on each entry puts the note text on the
system clipboard — one click to get the note into TANNSS. An Edit time button on each
completed entry opens HH:MM inputs for start and end time; saving reconstructs the stored
timestamps and recomputes the net duration.

---

## Why It Was Built

Copy completes the TANNSS reporting loop — the note is already there, the friction was
getting it to the clipboard. Time editing fixes the "forgot to stop the timer" problem:
previously the only option was to delete the entry and lose the note.

---

## Implementation Checklist

### Initial implementation — 2026-04-07

- [ ] Add 6 messages to `src/message.rs`: `CopyEntryNote(String)`, `OpenEditTime(Uuid)`,
  `EditTimeStartChanged(String)`, `EditTimeEndChanged(String)`, `SaveEditTime`, `CancelEditTime`
- [ ] Add 3 state fields to `App`: `editing_time_id`, `edit_time_start`, `edit_time_end`
- [ ] Initialise all three in `App::new()`
- [ ] Implement `CopyEntryNote` arm — returns `iced::clipboard::write(text)`
- [ ] Implement `OpenEditTime` arm — closes note editor, pre-fills HH:MM from stored timestamps
- [ ] Implement `EditTimeStartChanged` / `EditTimeEndChanged` arms
- [ ] Implement `SaveEditTime` arm — parse, validate, reconstruct UTC timestamps, recompute minutes
- [ ] Implement `CancelEditTime` arm
- [ ] Add time-edit close to top of `OpenEditNote` arm
- [ ] Add time-editing row to `review.rs` (with `continue`, before note_widget build)
- [ ] Add Copy button and Edit time button to normal note row in `review.rs`
- [ ] `cargo check` — zero errors
- [ ] Manual test: copy, edit times, validation, mutual exclusion, active entry guard

---

## Technical Notes

_Fill in after implementation. Say "I finished Phase 12" to trigger this._

---

## Change History

_No changes yet._
