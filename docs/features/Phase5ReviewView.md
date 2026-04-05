# Phase5ReviewView

- **Status:** Planned
- **Created:** 2026-04-05
- **Last updated:** 2026-04-05
- **Touches:** `src/app.rs`

---

## What It Does

Replaces the main tracker view when the user clicks Review. Shows all time entries for a
selected date in chronological order, with gaps between entries displayed as "Untracked"
blocks. Each entry shows the task name, start and end time, net minutes, pause minutes (if
any), and an editable note field. A summary bar at the bottom shows first start, last end,
total tracked, and total untracked time. Prev/Next buttons navigate between dates. Close
returns to the tracker.

---

## Why It Was Built

The app's entire purpose is to make end-of-day TANNSS reporting frictionless. Without a
review view, all the captured data is invisible — you cannot read your own session log. This
is the feature that turns raw data into a usable reporting surface.

---

## Implementation Checklist

### Initial implementation — 2026-04-05

**New types (Step 1)**

- [ ] Add `Screen` enum (`Tracker`, `Review`) to `app.rs`
- [ ] Add `ReviewRow` enum (`Entry { entry, task_name }`, `Gap { minutes }`) to `app.rs`

**App struct fields (Step 2)**

- [ ] Add `screen: Screen` to `App`
- [ ] Add `review_date: chrono::NaiveDate` to `App`
- [ ] Add `editing_note_id: Option<Uuid>` to `App`
- [ ] Add `editing_note_text: String` to `App`
- [ ] Initialise all four fields in `new()`

**Message variants (Step 3)**

- [ ] Add `CloseReview` to `Message`
- [ ] Add `ReviewPrevDay` to `Message`
- [ ] Add `ReviewNextDay` to `Message`
- [ ] Add `OpenEditNote(Uuid)` to `Message`
- [ ] Add `EditNoteChanged(String)` to `Message`
- [ ] Add `SaveEditNote` to `Message`
- [ ] Add `CancelEditNote` to `Message`

**update() arms (Step 4)**

- [ ] Implement `OpenReview` arm — set screen, reset date to today, clear note edit
- [ ] Implement `CloseReview` arm
- [ ] Implement `ReviewPrevDay` arm (uses `pred_opt()`)
- [ ] Implement `ReviewNextDay` arm — guard against future dates
- [ ] Implement `OpenEditNote` arm — copy-on-open into `editing_note_text`
- [ ] Implement `EditNoteChanged` arm
- [ ] Implement `SaveEditNote` arm — write back to entry, save
- [ ] Implement `CancelEditNote` arm — discard, clear state

**Helpers (Steps 5–6)**

- [ ] Add `format_hhmm(rfc3339: &str) -> String` static helper
- [ ] Add `build_review_rows(&self) -> Vec<ReviewRow>` method
    - [ ] Filter entries by `review_date` (string prefix match on `started_at`)
    - [ ] Include active entry if it started on `review_date`
    - [ ] Sort by `started_at`
    - [ ] Insert `Gap` rows for gaps ≥ 5 minutes between entries

**View (Steps 7–8)**

- [ ] Add `review_view()` method:
    - [ ] Date header with Prev / date label / Next (disabled on today) / Close
    - [ ] Entry rows: task name, times, duration, pause annotation, note or edit field
    - [ ] Gap rows: "⊘ Untracked — Xm"
    - [ ] Summary bar: first start, last end, total tracked, total untracked
- [ ] Update `view()` to route to `review_view()` when `screen == Screen::Review`

**Verify (Step 9)**

- [ ] Review opens on today's date
- [ ] Entries appear chronologically with correct data
- [ ] Gap rows appear between entries with ≥ 5 min gap
- [ ] Note edit: Save persists, Cancel discards
- [ ] Prev/Next navigate dates; Next is disabled on today
- [ ] Close returns to tracker without losing timer state
- [ ] Edited notes survive app restart

---

## Technical Notes

_To be filled in after implementation. Say "I finished Phase5ReviewView" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
