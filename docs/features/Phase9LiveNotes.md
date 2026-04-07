---
name: Phase 9 — Live Notes Modal
description: Note button in timer bar opens a modal to write session notes without stopping the timer
status: Planned
created: 2026-04-06
last_updated: 2026-04-06
touches: src/message.rs, src/app.rs, src/ui/timer_bar.rs, src/ui/note_modal.rs (new), src/ui/mod.rs
---

# Phase 9 — Live Notes Modal

- **Status:** Planned
- **Created:** 2026-04-06
- **Last updated:** 2026-04-06
- **Touches:** `src/message.rs`, `src/app.rs`, `src/ui/timer_bar.rs`, `src/ui/note_modal.rs` (new), `src/ui/mod.rs`

---

## What It Does

Adds a "Note" button to the timer bar. Clicking it opens a modal overlay with a text input
pre-filled from any previously written note for this session. Saving writes the note to the
active entry immediately — the timer keeps running. The stop prompt note field is pre-filled
from this note when the session ends.

---

## Why It Was Built

Notes could previously only be written at stop time. Mid-session observations ("blocked by
missing API key", "found the edge case") had to be held in memory until the timer stopped.
This is the most natural place in the workflow to capture context while it is fresh.

---

## Implementation Checklist

### Initial implementation — 2026-04-06

- [ ] Add 4 messages to `src/message.rs`: `OpenNoteModal`, `NoteModalChanged(String)`, `SaveNoteModal`,
  `CancelNoteModal`
- [ ] Add 2 state fields to `App` struct: `note_modal_open: bool`, `note_modal_text: String`
- [ ] Initialise both fields in `App::new()`
- [ ] Implement 4 new `update()` arms
- [ ] Add Note button to `src/ui/timer_bar.rs` with note-exists indicator
- [ ] Create `src/ui/note_modal.rs`
- [ ] Declare `pub mod note_modal;` in `src/ui/mod.rs`
- [ ] Update `view()` in `src/app.rs` to layer note modal using `match` on both modal flags
- [ ] `cargo check` — zero errors
- [ ] Manual test: write live note, stop timer, confirm pre-fill works

---

## Technical Notes

_Fill in after implementation. Say "I finished Phase 9" to trigger this._

---

## Change History

_No changes yet._
