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

**Data model (Step 0)**

- [ ] Add `paused_at: Option<String>` to `TimeEntry` with `#[serde(default)]`
- [ ] Add `paused_minutes: i64` to `TimeEntry` with `#[serde(default)]`
- [ ] Add `is_paused()` helper method to `TimeEntry`
- [ ] Initialise both fields in `TimeEntry::new()`

**Stop timer with note (Step 1)**

- [ ] Add `note: Option<String>` parameter to `stop_active_timer()`
- [ ] Finalise current pause into `paused_minutes` in `stop_active_timer()` if entry is paused
- [ ] Compute `minutes = gross - paused_minutes` in `stop_active_timer()`
- [ ] Update both call sites: `StartTimer` (pass `None`) and `StopTimer` (pass `None`)

**Stop prompt state and messages (Steps 2–3)**
- [ ] Add `stop_prompt_open`, `stop_prompt_note`, `stop_prompt_status` to `App`
- [ ] Initialise prompt fields in `new()`
- [ ] Import `TaskStatus` in `app.rs`
- [ ] Add `OpenStopPrompt`, `StopPromptNoteChanged`, `StopPromptStatusChanged`, `ConfirmStop`, `CancelStop` to `Message`
- [ ] Add `PauseTimer`, `ResumeTimer` to `Message`

**update() arms (Steps 4 + 12)**

- [ ] Implement `OpenStopPrompt` arm — sets prompt state, calls `pause_active_timer()`
- [ ] Implement `StopPromptNoteChanged` arm
- [ ] Implement `StopPromptStatusChanged` arm
- [ ] Implement `ConfirmStop` arm — stops timer with note, updates task status, clears prompt
- [ ] Implement `CancelStop` arm — closes prompt, calls `resume_active_timer()`
- [ ] Implement `PauseTimer` arm — calls `pause_active_timer()`
- [ ] Implement `ResumeTimer` arm — calls `resume_active_timer()`

**Pause/resume helpers (Step 8)**

- [ ] Add `pause_active_timer()` private method
- [ ] Add `resume_active_timer()` private method

**elapsed_display (Step 9)**

- [ ] Update `elapsed_display()` signature: add `paused_at: Option<&str>`, `paused_minutes: i64`
- [ ] Freeze display when `paused_at` is `Some` (use pause-start as effective now)
- [ ] Subtract `paused_minutes * 60` from gross seconds before formatting
- [ ] Update all `elapsed_display` call sites in `timer_bar()`

**subscription (Step 10)**

- [ ] Update `subscription()` — only tick when `active_entry.is_some() && !is_paused()`

**View changes (Steps 5–7 + 13)**
- [ ] Change Stop button in `timer_bar()` to emit `OpenStopPrompt`
- [ ] Add Pause/Resume button to `timer_bar()` (conditional on `entry.is_paused()`)
- [ ] Add `stop_prompt_view()` method
- [ ] Update `view()` to conditionally use `stack!`
- [ ] Add `stack` to imports

**Verify (Step 14)**

- [ ] Pause/Resume button freezes and unfreezes elapsed display
- [ ] Stop opens prompt with timer paused; Cancel resumes timer
- [ ] Stop & Save: JSON has correct `started_at`, `ended_at`, `minutes`, `paused_minutes`, `notes`
- [ ] Auto-stop (starting a new timer) is silent — no prompt, `notes: null`

---

## Technical Notes

*To be filled in after implementation. Say "I finished Phase 4" to trigger this.*

---

## Change History

*No changes yet.*
