---
name: BugFixes
description: Seven defects in the existing FyneUI — timer bar name, elapsed reset, status cycle, layout placeholder, quick-add mode/enter/clear
type: feature
---

# BugFixes

- **Status:** Planned
- **Created:** 2026-04-02
- **Last updated:** 2026-04-02
- **Touches:** `internal/ui/time_bar.go`, `internal/ui/task_list.go`, `internal/ui/app.go`, `internal/ui/quick_add.go`, `internal/tasks/store.go`

---

## What It Does

Corrects seven defects in the FyneUI feature that make the app incorrect or confusing in daily use. Prerequisite for all further feature work.

---

## Why It Was Built

Found during a full codebase review in the brainstorm session. These are not cosmetic — the timer bar shows the wrong information, the status cycle silently fails on Done tasks, and the layout has a visible placeholder widget.

---

## Implementation Checklist

### Initial implementation — 2026-04-02

- [ ] Add `tasks.Store.GetByID(id int64)` store method
- [ ] Fix timer bar to show task name using `GetByID`
- [ ] Fix elapsed binding reset to `00:00:00` when timer stops
- [ ] Fix status cycle — add `Done → TODO` case
- [ ] Set default mode in quick-add to `CaptureAndStart`
- [ ] Wire `nameEntry.OnSubmitted` in quick-add to call `submitBTN.OnTapped()`
- [ ] Clear `nameEntry` and hide window after quick-add submission
- [ ] Fix three-zone layout — timer bar at top, status bar at bottom, remove `bottomWidget` placeholder

---

## Technical Notes

_To be filled in after implementation. Say "I finished BugFixes" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
