# Feature — Phase 2: Core Interactions

**Date:** 2026-04-03
**Mode:** FEATURE
**Architecture reference:** `docs/Brainstorm-RustIcedRewrite.md`
**Permanent record:** `docs/features/Phase2CoreInteractions.md`

---

## What this phase builds

By the end of Phase 2, the app is genuinely usable:

- Type a task name and press Enter to add it
- Click Start on a task row to begin timing it
- The timer bar shows the task name and live elapsed time (ticking every second)
- Click Stop to stop the active timer
- Starting a new timer auto-stops the previous one
- Click the status badge on a task row to cycle through To Do → In Progress → Done
- The status bar shows today's real total time, including the currently running timer
- All changes are saved to disk immediately

---

## What already exists

From Phase 1:
- `App` struct with `tasks`, `entries`, `active_entry`
- `Message` enum with placeholder `Tick`, `AddTask`, `OpenReview`
- `task_list()` and `task_row()` render tasks (read-only, no buttons)
- `timer_bar()` shows task UUID (not name) with no Stop button
- `status_bar()` shows hardcoded "Today: 0h 0m"
- `subscription()` always returns `Subscription::none()`
- `TimeEntry::new()` and `Task::new()` ready to use
- `store::save_data()` ready to use

---

## Changes overview

| Area | Change |
|------|--------|
| `App` struct | Add `new_task_input: String` field |
| `Message` enum | Replace `AddTask` with `TaskNameChanged` + `SubmitNewTask`; add `StartTimer`, `StopTimer`, `CycleStatus` |
| `update()` | Implement all 6 new arms |
| `timer_bar()` | Show task name + elapsed; add Stop button |
| `task_row()` | Add Start button; make status label clickable |
| `status_bar()` | Add text input + Add button; compute real total |
| `task_list()` | Wrap in `scrollable` |
| `subscription()` | Return tick subscription when timer is running |
| `app.rs` imports | Add `button`, `scrollable`, `text_input`, `chrono` |

---

## Step 1 — Expand the Message enum

Replace the three placeholder variants with the real ones. The result:

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // Fired every second when a timer is active. Carries no data —
    // elapsed time is computed from active_entry.started_at in view().
    Tick,

    // Task input field in the status bar
    TaskNameChanged(String),  // text changed — update new_task_input
    SubmitNewTask,            // Enter pressed or Add clicked — create the task

    // Task row interactions
    StartTimer(Uuid),         // Start button clicked — begin timing this task
    CycleStatus(Uuid),        // Status badge clicked — advance to next status

    // Timer bar
    StopTimer,                // Stop button clicked — stop the active timer

    // Placeholder — wired in Phase 5
    OpenReview,
}
```

`StartTimer` and `CycleStatus` carry `Uuid` — the ID of the task they act on. `Uuid` is
`Copy`, so passing it into a message costs nothing and avoids any lifetime issues.

**Add `Uuid` to the imports** at the top of `app.rs`:

```rust
use uuid::Uuid;
```

---

## Step 2 — Add the input field to App

Add one field to the `App` struct:

```rust
pub struct App {
    tasks: Vec<AppTask>,
    entries: Vec<TimeEntry>,
    active_entry: Option<TimeEntry>,
    new_task_input: String,   // ← add this
}
```

Initialise it in `App::new()`:

```rust
let app = Self {
    tasks: data.tasks,
    entries: data.entries,
    active_entry,
    new_task_input: String::new(),  // ← add this
};
```

---

## Step 3 — Add a save helper

Every mutation needs to persist. Rather than repeating the serialisation call in every
`update()` arm, add a private helper:

```rust
// Saves the current state to disk. Logs errors without panicking.
fn save(&self) {
    let data = store::AppData {
        tasks: self.tasks.clone(),
        entries: self.entries.clone(),
    };
    if let Err(e) = store::save_data(&data) {
        eprintln!("Failed to save: {e}");
    }
}
```

**Why clone?** `save_data` takes `&AppData`, which needs to own its vecs. At this data
size, cloning the whole collection on every save is cheaper than the file write itself.

---

## Step 4 — Add a stop helper

Both `StopTimer` and `StartTimer` (which auto-stops first) need to stop the active timer.
Extract the logic into a private method so it is not duplicated:

```rust
// Stops the active timer if one is running.
// Updates ended_at and minutes on the entry, then syncs self.entries.
// Does nothing if no timer is active.
fn stop_active_timer(&mut self) {
    // .take() removes the value from self.active_entry and returns it,
    // leaving None behind — a clean way to "claim" an Option value.
    if let Some(mut entry) = self.active_entry.take() {
        use chrono::{DateTime, Utc};
        let now = Utc::now();
        entry.ended_at = Some(now.to_rfc3339());

        // Parse started_at so we can compute the duration.
        // If parsing fails (should not happen with our RFC 3339 strings),
        // fall back to now — resulting in 0 minutes rather than a crash.
        let started = DateTime::parse_from_rfc3339(&entry.started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        // num_minutes() returns i64. .max(0) guards against a negative result
        // if the clock is adjusted while the timer runs.
        entry.minutes = Some((now - started).num_minutes().max(0));

        // Sync back into self.entries. The entry is already in the vec from
        // when it was created in StartTimer — we just update it in place.
        if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *existing = entry;
        }
        // Note: no save() here — the caller is responsible for saving after
        // any additional changes (e.g. StartTimer stops then starts).
    }
}
```

---

## Step 5 — Implement update()

Handle each message. The match arms below replace the three empty placeholders from Phase 1:

```rust
pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            // No state change — Tick just causes view() to re-run,
            // which recomputes elapsed time from active_entry.started_at.
        }

        Message::TaskNameChanged(value) => {
            self.new_task_input = value;
        }

        Message::SubmitNewTask => {
            // trim() removes leading/trailing whitespace.
            // to_string() allocates a new owned String.
            let name = self.new_task_input.trim().to_string();
            if !name.is_empty() {
                self.tasks.push(AppTask::new(name));
                self.new_task_input.clear();
                self.save();
            }
        }

        Message::StartTimer(task_id) => {
            // Stop any running timer before starting a new one.
            self.stop_active_timer();

            // Create a new entry and store it in both places:
            //   self.entries — the persistent list (saved to disk)
            //   self.active_entry — the fast-access current timer
            let entry = TimeEntry::new(task_id);
            self.active_entry = Some(entry.clone());
            self.entries.push(entry);
            self.save();
        }

        Message::StopTimer => {
            self.stop_active_timer();
            self.save();
        }

        Message::CycleStatus(task_id) => {
            // iter_mut() gives mutable references — we can modify the task in place.
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                task.status = task.status.next();
            }
            self.save();
        }

        Message::OpenReview => {
            // Placeholder — Phase 5
        }
    }

    Task::none()
}
```

---

## Step 6 — Wire up the subscription

Replace `Subscription::none()` with the conditional tick:

```rust
pub fn subscription(&self) -> iced::Subscription<Message> {
    if self.active_entry.is_some() {
        // every() produces Instant values — discard with |_| since we don't use them.
        // The elapsed time is computed from active_entry.started_at, not from the Instant.
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    } else {
        iced::Subscription::none()
    }
}
```

Add `iced::time` to the imports. It is part of the `iced` crate — no new dependency needed.

---

## Step 7 — Add elapsed and total helpers

These are pure computations used by view functions. Add them as private methods on `App`:

```rust
// Formats the elapsed time since a given RFC 3339 timestamp as "Xm Ys" or "Xh Ym Zs".
fn elapsed_display(started_at: &str) -> String {
    use chrono::{DateTime, Utc};
    let started = DateTime::parse_from_rfc3339(started_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let secs = (Utc::now() - started).num_seconds().max(0);
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}h {m}m {s}s")
    } else {
        format!("{m}m {s}s")
    }
}

// Returns today's total tracked minutes (completed entries + active entry elapsed).
fn today_total_minutes(&self) -> i64 {
    use chrono::{DateTime, Utc};
    let today = Utc::now().format("%Y-%m-%d").to_string();

    // Sum minutes from entries that started today and have been stopped.
    let completed: i64 = self.entries.iter()
        .filter(|e| e.started_at.starts_with(&today) && e.minutes.is_some())
        .map(|e| e.minutes.unwrap_or(0))
        .sum();

    // Add the currently running timer's elapsed minutes (if today).
    let active: i64 = self.active_entry.as_ref()
        .filter(|e| e.started_at.starts_with(&today))
        .and_then(|e| {
            DateTime::parse_from_rfc3339(&e.started_at).ok().map(|dt| {
                (Utc::now() - dt.with_timezone(&Utc)).num_minutes().max(0)
            })
        })
        .unwrap_or(0);

    completed + active
}
```

`elapsed_display` is a static method (no `&self`) — it only needs the timestamp string.
`today_total_minutes` needs `&self` to access the entries and active_entry.

---

## Step 8 — Update timer_bar()

Show the task name + elapsed time, with a Stop button on the right:

```rust
fn timer_bar(&self) -> Element<'_, Message> {
    match &self.active_entry {
        Some(entry) => {
            // Look up the task name by matching task_id.
            // Falls back to "Unknown task" if the ID is not found (should not happen).
            let task_name = self.tasks.iter()
                .find(|t| t.id == entry.task_id)
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown task");

            let elapsed = Self::elapsed_display(&entry.started_at);

            row![
                text(format!("{task_name}  —  {elapsed}")).width(Length::Fill),
                button(text("Stop")).on_press(Message::StopTimer),
            ]
            .padding(12)
            .spacing(8)
            .into()
        }
        None => container(text("No timer running"))
            .width(Length::Fill)
            .padding(12)
            .into(),
    }
}
```

---

## Step 9 — Update task_row()

Add a Start button on the left and make the status label a clickable button:

```rust
fn task_row<'a>(&self, task: &'a AppTask) -> Element<'a, Message> {
    row![
        // Start button — emits StartTimer with this task's UUID
        button(text("▶")).on_press(Message::StartTimer(task.id)),

        // Task name — expands to fill available space
        text(&task.name).width(Length::Fill),

        // Status badge — clicking it cycles to the next status
        button(text(task.status.label())).on_press(Message::CycleStatus(task.id)),
    ]
    .padding(8)
    .spacing(8)
    .into()
}
```

The `▶` and status label are both buttons — they look different from the Stop button only
because of styling, which comes in a later phase. For now, they are plain buttons.

---

## Step 10 — Update task_list()

Wrap the column in a scrollable so it does not overflow when many tasks are added:

```rust
fn task_list(&self) -> Element<'_, Message> {
    if self.tasks.is_empty() {
        return container(text("No tasks yet. Add one below."))
            .width(Length::Fill)
            .padding(20)
            .into();
    }

    let rows: Vec<Element<Message>> = self.tasks.iter()
        .map(|task| self.task_row(task))
        .collect();

    // scrollable() wraps a widget and makes it vertically scrollable.
    // .height(Length::Fill) makes the list take the remaining vertical space
    // between the timer bar and status bar.
    scrollable(column(rows).spacing(4))
        .height(Length::Fill)
        .into()
}
```

---

## Step 11 — Update status_bar()

Add the task name input and an Add button, and display the real total:

```rust
fn status_bar(&self) -> Element<'_, Message> {
    let total = self.today_total_minutes();
    let h = total / 60;
    let m = total % 60;
    let total_label = text(format!("Today: {h}h {m}m"));

    row![
        total_label,

        // Length::Fill pushes the input to the right side
        // (or use a Spacer if you prefer a fixed-width input)
        text_input("Add task...", &self.new_task_input)
            .on_input(Message::TaskNameChanged)  // Message::TaskNameChanged as fn pointer
            .on_submit(Message::SubmitNewTask)   // fires when Enter is pressed
            .width(Length::Fill),

        button(text("Add")).on_press(Message::SubmitNewTask),
    ]
    .padding(8)
    .spacing(8)
    .into()
}
```

`.on_input(Message::TaskNameChanged)` works because `Message::TaskNameChanged` is a tuple
variant constructor — Rust treats it as a function `fn(String) -> Message`, which is exactly
what `.on_input()` expects.

---

## Step 12 — Update imports in app.rs

Add the new widget types at the top:

```rust
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task};
use uuid::Uuid;
```

`iced::time` is used inline in `subscription()` without a `use` statement — it reads clearly
as `iced::time::every(...)`. Add a `use` if you prefer shorter syntax.

---

## Step 13 — Verify

```
cargo run
```

Test in this order:
1. Type a task name in the bottom bar, press Enter — task appears in the list
2. Click Add button for a second task — appears in the list
3. Click ▶ on a task — timer bar shows task name and a ticking elapsed counter
4. Click ▶ on a different task — previous timer stops, new one starts
5. Click Stop in the timer bar — timer clears, "No timer running"
6. Click the status badge on a task — cycles To Do → In Progress → Done → To Do
7. Restart the app — tasks, entries, and task statuses are still there
8. Start a timer, restart the app — timer bar shows the task as active (loaded from disk)

---

## What this phase does not do

- **No Eisenhower sorting** — task list is in insertion order. Phase 3.
- **No stop-with-note prompt** — Stop ends the timer immediately. Phase 4.
- **No visual styling** — buttons are plain iced defaults. Phase 3/styling pass.
- **No elapsed shown for the active entry in today's total** — actually this IS
  included via `today_total_minutes()`. The total updates every second with the timer.

---

## Commit message (draft)

```
feat(app): wire up core interactions — add task, start/stop timer, status cycling

Implements the first usable loop: tasks can be added, timed, and their status
advanced. The timer bar shows live elapsed time via a 1-second subscription.
All state is persisted to ~/.tracker/data.json on every mutation.
```
