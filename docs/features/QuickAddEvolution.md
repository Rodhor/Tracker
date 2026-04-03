---
name: QuickAddEvolution
description: Quick-add with task filter/create, auto-stop on start, stop prompt with notes and status selection, Urgent/Important toggles
type: feature
---

# QuickAddEvolution

- **Status:** Planned
- **Created:** 2026-04-02
- **Last updated:** 2026-04-02
- **Touches:** `internal/timer/store.go`, `internal/ui/quick_add.go`, `internal/ui/time_bar.go`, `internal/ui/app.go`

---

## What It Does

Transforms the quick-add window into the primary daily workflow surface. A single text field filters existing tasks or creates a new one. Starting a timer silently auto-stops the previous one. Stopping a timer manually brings up a prompt for a note and a status decision. Urgent/Important toggles are available when creating new tasks.

---

## Why It Was Built

The core daily loop — hotkey → quick-add → new task + timer, repeat — was too slow and required too many decisions. The auto-stop makes starting a new task a single action. The filter/create field removes the task switching friction. The stop prompt captures the note at the natural moment (when you stop), preserving focus during work.

---

## Implementation Checklist

### Initial implementation — 2026-04-02

- [ ] Modify `timer.Store.Start()` — auto-stop previous timer if one is running
- [ ] Add `timer.Store.StopWithNote(note string) error` — stops and saves note in one call
- [ ] Replace stop button `OnTapped` in `time_bar.go` with modal: note field + status selector
- [ ] Pass `win fyne.Window` into `newTimerBar` for dialog anchoring
- [ ] Replace mode select in quick-add with suggestion list (filter/create)
- [ ] Load tasks on quick-add open; filter on `nameEntry.OnChanged`
- [ ] Track `selectedTaskID *int64` — nil = create new, non-nil = use existing
- [ ] On submit: create or select task, call `timerStore.Start()`, hide window, refresh
- [ ] Add Urgent/Important checkboxes; call `UpdateMetadata` after `Add()` for new tasks
- [ ] Reset all fields (text, checkboxes, selectedTaskID) after each submission

---

## Technical Notes

_To be filled in after implementation. Say "I finished QuickAddEvolution" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
