---
name: Phase 11 — Keyboard Shortcuts
description: Q/N/P/S/R/Escape shortcuts for primary app actions; extends the event::listen_with subscription
status: Planned
created: 2026-04-06
last_updated: 2026-04-06
touches: src/message.rs, src/app.rs
---

# Phase 11 — Keyboard Shortcuts

- **Status:** Planned
- **Created:** 2026-04-06
- **Last updated:** 2026-04-06
- **Touches:** `src/message.rs`, `src/app.rs`

---

## What It Does

Six keyboard shortcuts: Q (quick-add), N (note modal), P (pause/resume), S (stop prompt),
R (review toggle), Escape (close current modal). Character shortcuts only fire when no text
field is focused (`Status::Ignored`). Arrow keys and Escape are intercepted regardless of
focus. State-conditional routing (P → pause or resume, R → open or close) is handled by
three new "dispatcher" messages that resolve in `update()`.

---

## Why It Was Built

The app is used in a flow-state context — switching hands to the mouse breaks focus. The
shortcuts allow the full daily loop (start, note, stop, review) without leaving the keyboard.

---

## Implementation Checklist

### Initial implementation — 2026-04-06

- [ ] Add 3 messages to `src/message.rs`: `TogglePauseTimer`, `ToggleReview`, `CloseActiveModal`
- [ ] Implement 3 new `update()` arms for those messages
- [ ] Add guards to `OpenNoteModal` and `OpenStopPrompt` arms (no-op when no active timer)
- [ ] Replace the conditional `submit` subscription with an unconditional `keys` subscription
- [ ] Extend `event::listen_with` fn to match `Status::Ignored` character keys (Q/N/P/S/R)
- [ ] Extend `event::listen_with` fn to match Escape → `CloseActiveModal`
- [ ] Update `Subscription::batch([tick, keys])`
- [ ] `cargo check` — zero errors
- [ ] Manual test all 6 shortcuts in all relevant app states (see Step 5 in planning doc)

---

## Technical Notes

_Fill in after implementation. Say "I finished Phase 11" to trigger this._

---

## Change History

_No changes yet._
