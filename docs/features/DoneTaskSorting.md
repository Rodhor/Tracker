# Done Task Sorting

- **Status:** Planned
- **Created:** 2026-04-08
- **Last updated:** 2026-04-08
- **Touches:** `src/data/task.rs`, `src/app.rs`, `src/ui/task_list.rs`

---

## What It Does

Separates Done tasks into a "Done today" section at the bottom of the task list. Tasks marked Done on a previous day are hidden entirely, keeping the list focused on current work.

---

## Why It Was Built

Done tasks cluttered the active task list with no way to distinguish work finished today from tasks completed days ago.

---

## Implementation Checklist

### Initial implementation — 2026-04-08
- [ ] Add `completed_at: Option<String>` to `Task` struct with `#[serde(default)]`
- [ ] Initialise `completed_at` to `None` in `Task::new()`
- [ ] Set/clear `completed_at` in `CycleStatus` arm
- [ ] Set/clear `completed_at` in `ConfirmStop` arm
- [ ] Exclude `Done` tasks from `sorted` and `unsorted` groups in `task_list.rs`
- [ ] Build `done_today` group (Done + completed today)
- [ ] Render `done_today` section below Needs Sorting
- [ ] Verify: cycle to Done → moves to Done today; cycle back → returns to original section
- [ ] Verify: Done tasks from previous days are hidden on restart

---

## Technical Notes

_To be filled in after implementation._

---

## Change History

_No changes yet._
