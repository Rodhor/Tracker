# Phase 2 — Core Interactions

- **Status:** Planned
- **Created:** 2026-04-03
- **Last updated:** 2026-04-03
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
- [ ] Expand Message enum — `TaskNameChanged`, `SubmitNewTask`, `StartTimer`, `StopTimer`, `CycleStatus`
- [ ] Add `new_task_input: String` to App struct and initialise in `new()`
- [ ] Add `save()` private helper
- [ ] Add `stop_active_timer()` private helper
- [ ] Implement all update() arms
- [ ] Wire up subscription() — tick when active timer, none otherwise
- [ ] Add `elapsed_display()` static helper
- [ ] Add `today_total_minutes()` helper
- [ ] Update `timer_bar()` — task name + elapsed + Stop button
- [ ] Update `task_row()` — Start button + clickable status badge
- [ ] Update `task_list()` — wrap in scrollable
- [ ] Update `status_bar()` — text_input + Add button + real total
- [ ] Update imports — button, scrollable, text_input, Uuid
- [ ] Verify: full interaction loop works and survives app restart

---

## Technical Notes

*To be filled in after implementation. Say "I finished Phase 2" to trigger this.*

---

## Change History

*No changes yet.*
