# Feature: Done Task Sorting

- **Mode:** FEATURE
- **Date:** 2026-04-08

---

## What it does

Adds a "Done today" section at the bottom of the task list. Tasks marked Done on a previous day disappear from the list entirely, keeping it focused on today's work. Reactivating a Done task (cycling its status back) makes it reappear immediately.

---

## Data change — `src/data/task.rs`

Add a `completed_at` field to the `Task` struct:

- Type: `Option<String>` (RFC3339 timestamp, same format used everywhere else)
- Decoration: `#[serde(default)]` so existing saved tasks load without error (same pattern as `paused_at` on `TimeEntry`)
- Initialise to `None` in `Task::new()`

---

## Logic changes — `src/app.rs`

Two places in `update()` set a task's status. Both need the same follow-up: after writing the new status, check its value and update `completed_at` accordingly.

**Rule:**
- New status is `Done` → set `completed_at` to the current UTC time as an RFC3339 string (`Utc::now().to_rfc3339()`)
- New status is anything else → set `completed_at = None`

**`CycleStatus` arm**
After `task.status = task.status.next()`, apply the rule above to `task.completed_at`.

**`ConfirmStop` arm**
After `task.status = new_status`, apply the same rule.

---

## View changes — `src/ui/task_list.rs`

Currently `view()` builds two groups: `sorted` (has priority) and `unsorted` (no priority). A third group — `done_today` — needs to be added.

**Step 1 — Exclude Done tasks from existing groups**

Add a status filter to the existing `filter()` calls so that tasks with `status == TaskStatus::Done` are not included in `sorted` or `unsorted`. Without this, a Done task with priority flags set would appear in both TODOS and Done today.

**Step 2 — Build the `done_today` group**

Filter `app.tasks` for tasks where:
- `status == TaskStatus::Done`, **and**
- `completed_at` starts with today's date string (format: `"%Y-%m-%d"`)

Use the same `starts_with` pattern already used in `status_bar.rs` and `review.rs` for filtering entries by date.

**Step 3 — Render the Done today section**

After the Needs Sorting section, render `done_today` with a header label (e.g. `"Done today"`). Each task inside is a standard `task_row` — no special styling needed unless you want to add it later.

---

## Implementation order

1. Add `completed_at: Option<String>` to `Task` struct with `#[serde(default)]`
2. Initialise it to `None` in `Task::new()`
3. Update `CycleStatus` in `app.rs` to set/clear `completed_at`
4. Update `ConfirmStop` in `app.rs` to set/clear `completed_at`
5. Add the `status != Done` filter to the `sorted` and `unsorted` groups in `task_list.rs`
6. Build and render the `done_today` group below `unsorted`

---

## Testing

- Mark a task Done via the cycle button → it moves to Done today immediately
- Mark a task Done via the stop prompt → same
- Cycle a Done task back to Todo → it leaves Done today and reappears in its original section
- Restart the app with Done tasks from a previous day → they are absent from all sections
- A task with priority flags set, marked Done → does not appear in TODOS, only in Done today
