# Phase 2 — Core Interactions

- **Status:** Complete
- **Created:** 2026-04-03
- **Last updated:** 2026-04-04
- **Touches:** `src/app.rs`

---

## What It Does

Makes the app genuinely usable. The user can add tasks by name, start and stop
timers against them, cycle task status, and see a live elapsed display and daily
total. All changes are persisted to disk immediately.

---

## Why It Was Built

Phase 1 established the skeleton — window opens, data structs exist, layout renders.
Phase 2 wires up every interaction in that skeleton so the app does real work.

---

## Implementation Checklist

### Initial implementation — 2026-04-03
- [x] Expand Message enum — `TaskNameChanged`, `SubmitNewTask`, `StartTimer`, `StopTimer`, `CycleStatus`
- [x] Add `new_task_input: String` to App struct and initialise in `new()`
- [x] Add `save()` private helper
- [x] Add `stop_active_timer()` private helper
- [x] Implement all update() arms
- [x] Wire up subscription() — tick when active timer, none otherwise
- [x] Add `elapsed_display()` static helper
- [x] Add `today_total_minutes()` helper
- [x] Update `timer_bar()` — task name + elapsed + Stop button
- [x] Update `task_row()` — Start button + clickable status badge
- [x] Update `task_list()` — wrap in scrollable
- [x] Update `status_bar()` — text_input + Add button + real total
- [x] Update imports — button, scrollable, text_input, Uuid
- [x] Verify: full interaction loop works and survives app restart

---

## Technical Notes

- **`stop_active_timer()` helper:** Extracted as a private method so both `StopTimer` and
  `StartTimer` (which auto-stops first) share the same stop logic without duplication.

- **`save()` helper:** Clones both vecs into a fresh `AppData` on every mutation. At this
  data size the clone is cheaper than the file write. Called at the end of every arm that
  changes state.

- **`elapsed_display()` as a static method:** Takes only a `&str` timestamp — no `&self`
  needed. Called from `timer_bar()` as `Self::elapsed_display(...)`.

- **`today_total_minutes()` includes active entry:** The running timer's current elapsed
  minutes are added to the completed total so the status bar reflects real-time progress.

- **`StopTimer` is a unit variant (no `()`):** An early implementation used `StopTimer()`
  (tuple variant, zero fields). Changed to a unit variant so it can be passed directly as
  a value to `.on_press()` without needing a closure.

- **`task_row` lifetime:** Uses `fn task_row<'a>(&'a self, task: &'a AppTask)` — both `self`
  and `task` tied to the same `'a`. The plan only tied `task`, but tying `self` as well
  compiles and has no practical impact at this call site.

- **Pattern followed:** iced Elm architecture — all mutations go through `update()`, view
  functions are pure reads of `&self`.

---

## Change History

*No changes yet.*
