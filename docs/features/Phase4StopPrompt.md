# Phase 4 — Stop Prompt and Entry Notes

- **Status:** Planned
- **Created:** 2026-04-05
- **Last updated:** 2026-04-05
- **Touches:** `src/app.rs`

---

## What It Does

Replaces the immediate stop behaviour with a modal prompt. When the user clicks Stop,
a panel appears over the main content asking for a work note and a new task status.
Confirming writes the note to the TimeEntry and updates the task. Cancelling leaves
the timer running. Auto-stop (on timer switch) bypasses the prompt entirely.

---

## Why It Was Built

The review view (Phase 5) shows time entries with their notes. Without this prompt,
every entry has a null note and the review is less useful. This phase captures the
context of each work block at the natural moment: when the user stops the timer.

---

## Implementation Checklist

### Initial implementation — 2026-04-05

- [ ] Add `note: Option<String>` parameter to `stop_active_timer()` — update all callers
- [ ] Add `stop_prompt_open`, `stop_prompt_note`, `stop_prompt_status` to `App`
- [ ] Initialise prompt fields in `new()`
- [ ] Import `TaskStatus` in `app.rs`
- [ ] Add `OpenStopPrompt`, `StopPromptNoteChanged`, `StopPromptStatusChanged`, `ConfirmStop`, `CancelStop` to `Message`
- [ ] Implement all 5 new `update()` arms
- [ ] Change Stop button in `timer_bar()` to emit `OpenStopPrompt`
- [ ] Add `stop_prompt_view()` method
- [ ] Update `view()` to conditionally use `stack!`
- [ ] Add `stack` to imports
- [ ] Verify: prompt opens/closes, note and status persist, auto-stop is silent

---

## Technical Notes

*To be filled in after implementation. Say "I finished Phase 4" to trigger this.*

---

## Change History

*No changes yet.*
