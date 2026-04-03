---
name: ReviewWindow
description: Daily log window — chronological time entries with gaps, inline note editing, delete, summary bar, date navigation
type: feature
---

# ReviewWindow

- **Status:** Planned
- **Created:** 2026-04-02
- **Last updated:** 2026-04-02
- **Touches:** `internal/ui/review.go`, `internal/timer/store.go`, `internal/tasks/store.go`, `internal/db/db.go`, `internal/timer/model.go`, `internal/ui/app.go`, `internal/ui/status_bar.go`

---

## What It Does

A dedicated second window for end-of-day reporting. Shows today's time entries in chronological order with start time, end time, duration, and an editable notes field per entry. Gaps between entries are shown as "Untracked" blocks (breaks). A summary bar shows the first start, last end, total tracked, and total untracked time. Date navigation lets you review previous days. Entries and tasks can be deleted from here.

---

## Why It Was Built

The app's entire purpose is to let you fill in TANNSS at the end of the day without relying on memory. This window is the reporting surface — you read your entries with their notes and copy the data into TANNSS. Without it, the captured data is stuck in the database with no way to review it.

---

## Implementation Checklist

### Initial implementation — 2026-04-02

- [ ] Migration 4: `ALTER TABLE time_entries ADD COLUMN notes TEXT` with `columnExists` guard
- [ ] Update `timer/model.go` — add `Notes *string` field
- [ ] Add `timer.Store.ListByDate(date string) ([]TimeEntry, error)`
- [ ] Add `timer.Store.UpdateNotes(id int64, notes string) error`
- [ ] Add `timer.Store.Delete(id int64) error`
- [ ] Add `tasks.Store.Delete(id int64) error` with cascade delete of time entries
- [ ] Create `internal/ui/review.go` with window skeleton and date navigation
- [ ] Build `reviewRow` type to unify entry rows and gap rows
- [ ] Implement `buildRows()` — interleave entry rows and gap rows from `[]TimeEntry`
- [ ] Implement entry row layout: task name, start→end, duration, notes entry
- [ ] Implement gap row layout: muted "⊘ Untracked — Xm" label
- [ ] Wire `notesEntry.OnSubmitted` to call `timerStore.UpdateNotes()`
- [ ] Add delete button per entry row with confirmation dialog
- [ ] Implement `buildSummary()` — first start, last end, tracked total, untracked total
- [ ] Render summary in the bottom bar; update on reload
- [ ] Wire review window into `app.go`; add "Review" button to status bar and tray menu
- [ ] Add delete task option to task list edit dialog; call `taskStore.Delete()`

---

## Technical Notes

_To be filled in after implementation. Say "I finished ReviewWindow" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
