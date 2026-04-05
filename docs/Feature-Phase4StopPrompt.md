# Feature — Phase 4: Stop Prompt and Entry Notes

**Date:** 2026-04-05
**Mode:** FEATURE
**Architecture reference:** `docs/Brainstorm-RustIcedRewrite.md`
**Permanent record:** `docs/features/Phase4StopPrompt.md`

---

## What this phase builds

- Clicking Stop opens a modal prompt — it no longer stops immediately
- The prompt shows the task name, a note field, and three status buttons (To Do / In Progress / Done)
- The status defaults to the task's natural next status when the prompt opens
- The timer is **paused** while the prompt is visible — it stops counting
- Confirming stops the timer, writes the note to the `TimeEntry`, and updates the task status
- Cancelling dismisses the prompt and **resumes** the timer from where it paused
- A **Pause** button sits in the timer bar alongside Stop — for manual breaks
- When paused, the button becomes **Resume**; the elapsed display freezes
- Auto-stop (when starting a new timer via the Start button) is silent — no pause, no prompt

This phase introduces `iced::widget::stack`, which layers widgets on top of each other.
The prompt panel is drawn over the main content, not in a separate window.

---

## What already exists

- `stop_active_timer()` private method — handles the stop logic
- `TimeEntry.notes: Option<String>` — field exists, never written until now
- `TaskStatus::next()` — used to compute the default status in the prompt
- `StopTimer` message — currently wired to the Stop button; will be replaced

---

## Changes overview

| Area                    | Change                                                                          |
|-------------------------|---------------------------------------------------------------------------------|
| `src/data/entry.rs`     | Add `paused_at: Option<String>` field (with `#[serde(default)]`)                |
| `stop_active_timer()`   | Add `note` parameter; use `paused_at` as effective end time if paused           |
| `pause_active_timer()`  | New private helper — sets `paused_at` and syncs to entries vec                  |
| `resume_active_timer()` | New private helper — shifts `started_at` forward to exclude pause, clears field |
| `elapsed_display()`     | Add `end_at: Option<&str>` — pass `paused_at` when paused to freeze display     |
| `App` struct            | Add 3 stop-prompt state fields                                                  |
| `Message` enum          | Add `OpenStopPrompt`, `PauseTimer`, `ResumeTimer`, + 4 prompt messages          |
| `update()`              | Implement 7 new arms; modify `OpenStopPrompt` and `CancelStop` for pause        |
| `subscription()`        | Only tick when timer is running (not paused)                                    |
| `timer_bar()`           | Pause/Resume button + Stop button; elapsed freezes when paused                  |
| `stop_prompt_view()`    | New method — the modal panel                                                    |
| `view()`                | Conditionally wrap with `stack!` when prompt is open                            |
| imports                 | Add `stack`                                                                     |

---

## Step 0 — Add `paused_at` and `paused_minutes` to TimeEntry (`src/data/entry.rs`)

Two new fields — both optional/defaulted so existing JSON files load without error:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: Uuid,
    pub task_id: Uuid,
    pub started_at: String,         // original start — NEVER modified after creation
    pub ended_at: Option<String>,
    pub minutes: Option<i64>,       // net work minutes (total minus all pauses)
    pub notes: Option<String>,
    #[serde(default)]
    pub paused_at: Option<String>,  // Some(timestamp) while currently paused, else None
    #[serde(default)]
    pub paused_minutes: i64,        // total accumulated pause time in minutes
}
```

Add both fields to `TimeEntry::new()`:

```rust
paused_at: None,
paused_minutes: 0,
```

Add a helper method:

```rust
pub fn is_paused(&self) -> bool {
    self.paused_at.is_some()
}
```

**Why two fields?**

`paused_at` is transient state — it is set when a pause begins and cleared when it ends or
the entry is stopped. `paused_minutes` accumulates the total pause time across all
pause/resume cycles during a work block.

**Why `started_at` must not change:**

When you report work into TANNSS, you need the original start and end times plus the
pause duration to subtract. For example: started 10:00, ended 11:50, paused 15 min →
bill 1h 35m. Shifting `started_at` would destroy the original start, making accurate
reporting impossible.

**Why `#[serde(default)]`?**

Existing JSON files have no `paused_at` or `paused_minutes` key. Without this annotation,
serde errors on load. With it, missing keys default to `None` / `0`.

---

## Step 1 — Modify `stop_active_timer()` to accept a note and handle pause

```rust
fn stop_active_timer(&mut self, note: Option<String>) {
    if let Some(mut entry) = self.active_entry.take() {
        use chrono::{DateTime, Utc};
        let now = Utc::now();

        // If currently paused, add the current pause duration to paused_minutes
        // before computing net time. Work stops now regardless.
        if let Some(paused_at_str) = &entry.paused_at {
            if let Ok(paused_at) = DateTime::parse_from_rfc3339(paused_at_str) {
                let this_pause = (now - paused_at.with_timezone(&Utc))
                    .num_minutes().max(0);
                entry.paused_minutes += this_pause;
            }
        }

        let started = DateTime::parse_from_rfc3339(&entry.started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        let gross_minutes = (now - started).num_minutes().max(0);

        entry.ended_at = Some(now.to_rfc3339());
        entry.minutes = Some((gross_minutes - entry.paused_minutes).max(0));
        entry.notes = note;
        entry.paused_at = None;
        // paused_minutes is preserved on the entry — the review will show it

        if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *existing = entry;
        }
    }
}
```

`minutes` = gross duration − pauses = net billable time.
`paused_minutes` stays on the entry so the review view can show both figures.
`ended_at` = now regardless of pause state — the block ended when the user clicked Stop.

**Update all existing call sites** — there are two:

```rust
// In StartTimer:
self.stop_active_timer(None);   // auto-stop: no note, no prompt

// In StopTimer (keep the arm but it now passes None):
self.stop_active_timer(None);
```

---

## Step 2 — Add stop-prompt state to App

```rust
pub struct App {
    // ... existing fields ...

    // Stop prompt modal state
    stop_prompt_open: bool,
    stop_prompt_note: String,
    stop_prompt_status: TaskStatus,
}
```

Initialise in `App::new()`:

```rust
stop_prompt_open: false,
stop_prompt_note: String::new(),
stop_prompt_status: TaskStatus::Todo,   // overwritten when the prompt opens
```

`TaskStatus` needs to be imported in `app.rs` — add it to the existing `use` line:

```rust
use crate::data::task::{Task as AppTask, TaskStatus};
```

---

## Step 3 — Add new Message variants

```rust
pub enum Message {
    // ... existing variants ...

    // Stop prompt modal
    OpenStopPrompt,                          // Stop button clicked — open the prompt
    StopPromptNoteChanged(String),           // note field changed
    StopPromptStatusChanged(TaskStatus),     // user picked a status button
    ConfirmStop,                             // Stop & Save clicked
    CancelStop,                             // Cancel clicked — leave timer running
}
```

`StopTimer` can stay in the enum — it is used by `StartTimer`'s auto-stop path if you
ever want to trigger a stop programmatically. It is no longer emitted by the UI.

`TaskStatus` must derive `Clone` to be carried in a `Message` — it already does.

---

## Step 4 — Implement update() arms

```rust
Message::OpenStopPrompt => {
    // Default the status to the task's natural next status.
    // This gives a sensible starting point: a Todo task defaults to In Progress,
    // an In Progress task defaults to Done, etc.
    let next_status = self.active_entry.as_ref()
        .and_then(|e| self.tasks.iter().find(|t| t.id == e.task_id))
        .map(|t| t.status.next())
        .unwrap_or(TaskStatus::Todo);

    self.stop_prompt_open = true;
    self.stop_prompt_note.clear();
    self.stop_prompt_status = next_status;
}

Message::StopPromptNoteChanged(value) => {
    self.stop_prompt_note = value;
}

Message::StopPromptStatusChanged(status) => {
    self.stop_prompt_status = status;
}

Message::ConfirmStop => {
    // 1. Collect what we need before mutating
    let note = {
        let s = self.stop_prompt_note.trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    };
    let new_status = self.stop_prompt_status.clone();

    // 2. Stop the timer with the note
    self.stop_active_timer(note);

    // 3. Update the task status
    //    active_entry is now None (taken by stop_active_timer), so we need
    //    to find the task a different way. We already have new_status;
    //    find the task by looking at the entry we just stopped.
    //    Because stop_active_timer updates self.entries, we can find the
    //    most recently stopped entry (ended_at is Some, was just set).
    if let Some(entry) = self.entries.iter().rev().find(|e| e.ended_at.is_some()) {
        let task_id = entry.task_id;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = new_status;
        }
    }

    // 4. Clear prompt state and save
    self.stop_prompt_open = false;
    self.stop_prompt_note.clear();
    self.save();
}

Message::CancelStop => {
    // Close the prompt — timer keeps running
    self.stop_prompt_open = false;
    self.stop_prompt_note.clear();
}
```

**Why find by `rev().find(ended_at.is_some())`?**

After `stop_active_timer()`, `self.active_entry` is `None` — we can no longer read the
task ID from it. We saved the note and ended_at onto the entry in `self.entries`. The
most recently stopped entry is the last one in the vec with `ended_at.is_some()`.
Iterating in reverse (`rev()`) finds it efficiently without scanning the whole list.

---

## Step 5 — Update timer_bar()

Change the Stop button to emit `OpenStopPrompt` instead of `StopTimer`:

```rust
button(text("Stop")).on_press(Message::OpenStopPrompt),
//                                      ↑ was Message::StopTimer
```

---

## Step 6 — Add stop_prompt_view()

This renders the modal panel. It only ever renders when `stop_prompt_open` is true — the
call site in `view()` guards it. The container fills the full screen and centers the panel.

```rust
fn stop_prompt_view(&self) -> Element<'_, Message> {
    let task_name = self.active_entry.as_ref()
        .and_then(|e| self.tasks.iter().find(|t| t.id == e.task_id))
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown task");

    // Three buttons — one per status. The selected one is shown in the label below.
    // Without styling, all three look the same; the current selection is shown as text.
    let status_row = row![
        button(text("To Do"))
            .on_press(Message::StopPromptStatusChanged(TaskStatus::Todo)),
        button(text("In Progress"))
            .on_press(Message::StopPromptStatusChanged(TaskStatus::InProgress)),
        button(text("Done"))
            .on_press(Message::StopPromptStatusChanged(TaskStatus::Done)),
    ]
    .spacing(8);

    let panel = column![
        text(format!("Stopping: {task_name}")),
        text_input("What did you accomplish?", &self.stop_prompt_note)
            .on_input(Message::StopPromptNoteChanged)
            .on_submit(Message::ConfirmStop),
        status_row,
        text(format!("Mark as: {}", self.stop_prompt_status.label())),
        row![
            button(text("Stop & Save")).on_press(Message::ConfirmStop),
            button(text("Cancel")).on_press(Message::CancelStop),
        ]
        .spacing(8),
    ]
    .spacing(12)
    .padding(24);

    // container fills the full window and centers the panel within it.
    // This is what makes it feel like a modal — it sits on top of the base layout
    // via widget::stack (wired in view()).
    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
```

**Note on `center_x` / `center_y`:** In iced 0.14 these methods take a `Length` argument.
If the compiler reports that the method does not exist or the signature is wrong, use the
alignment API instead:

```rust
use iced::alignment::{Horizontal, Vertical};

container(panel)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
```

Both achieve the same result. Try `center_x` first; fall back to `align_x` if it errors.

---

## Step 7 — Update view() to use stack

```rust
pub fn view(&self) -> Element<'_, Message> {
    let base = container(
        column![self.timer_bar(), self.task_list(), self.status_bar()].spacing(0)
    )
    .width(Length::Fill)
    .height(Length::Fill);

    if self.stop_prompt_open {
        // stack![] layers widgets. Later children are drawn on top.
        // The base layout is behind; the prompt panel is in front.
        stack![
            base,
            self.stop_prompt_view(),
        ]
        .into()
    } else {
        base.into()
    }
}
```

**How `stack` works:**

`stack![]` draws its children in order, with each one occupying the same space. The first
child is the base; every subsequent child is drawn on top. Unlike `column` and `row`, which
lay children out in sequence, `stack` layers them at the same position. Each child is sized
independently — which is why the modal container needs `Length::Fill` to cover the whole
window and block interaction with the base layout underneath.

Add `stack` to the iced widget imports:

```rust
use iced::widget::{button, checkbox, column, container, row, scrollable, stack, text, text_input};
```

---

## Step 8 — Add pause/resume helpers

Two private methods on `App`. Neither touches `started_at` — that field is immutable
after creation.

```rust
fn pause_active_timer(&mut self) {
    if let Some(entry) = self.active_entry.as_mut() {
        if entry.paused_at.is_none() {
            // Record the moment the pause began.
            // paused_minutes is NOT incremented here — that happens on resume or stop,
            // once we know how long this pause lasted.
            entry.paused_at = Some(Utc::now().to_rfc3339());

            // Keep self.entries in sync so the data is consistent if the app crashes.
            if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry.clone();
            }
        }
    }
}

fn resume_active_timer(&mut self) {
    if let Some(entry) = self.active_entry.as_mut() {
        if let Some(paused_at_str) = entry.paused_at.take() {
            // Compute how long this pause lasted and add it to the running total.
            use chrono::{DateTime, Utc};
            if let Ok(paused_at) = DateTime::parse_from_rfc3339(&paused_at_str) {
                let this_pause = (Utc::now() - paused_at.with_timezone(&Utc))
                    .num_minutes()
                    .max(0);
                entry.paused_minutes += this_pause;
            }
            // paused_at is already cleared by .take() above.

            if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry.clone();
            }
        }
    }
}
```

**Why accumulate on resume, not on pause?**

At pause time we do not yet know how long the pause will be. We only know once the user
resumes (or stops). Recording the pause start time and adding the duration on resume is
the only approach that works without background tasks.

**Why `entry.paused_at.take()`?**

`.take()` on an `Option<T>` moves the value out and leaves `None` in its place — a single
operation that both reads the old value and clears the field. It is idiomatic Rust for
"consume and clear".

---

## Step 9 — Update `elapsed_display()`

The display needs to:

- Subtract accumulated pause time from the gross duration
- Freeze when currently paused (show the net time at the moment of pause, not keep advancing)

```rust
fn elapsed_display(started_at: &str, paused_at: Option<&str>, paused_minutes: i64) -> String {
    use chrono::{DateTime, Utc};

    let started = match DateTime::parse_from_rfc3339(started_at) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => return "0:00:00".to_string(),
    };

    // If currently paused, use the pause-start time as the "now" anchor.
    // This freezes the display at the net duration when the pause began.
    let effective_now = if let Some(p) = paused_at {
        DateTime::parse_from_rfc3339(p)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    let gross_secs = (effective_now - started).num_seconds().max(0);
    let pause_secs = paused_minutes * 60;
    let net_secs = (gross_secs - pause_secs).max(0);

    let h = net_secs / 3600;
    let m = (net_secs % 3600) / 60;
    let s = net_secs % 60;
    format!("{h}:{m:02}:{s:02}")
}
```

**Update all call sites in `timer_bar()`:**

Before, the call was:

```rust
let elapsed = Self::elapsed_display( & entry.started_at);
```

After:

```rust
let elapsed = Self::elapsed_display(
& entry.started_at,
entry.paused_at.as_deref(),
entry.paused_minutes,
);
```

`as_deref()` converts `Option<String>` to `Option<&str>` — the deref coercion needed to
pass a string reference without cloning.

---

## Step 10 — Update `subscription()`

The timer should only tick when the timer is actively running — not when it is paused.
A paused timer's display does not change, so there is no reason to trigger a redraw every
second.

```rust
fn subscription(&self) -> Subscription<Message> {
    // Tick only when a timer is active AND not paused.
    // is_paused() returns true when paused_at is Some.
    let should_tick = self
        .active_entry
        .as_ref()
        .map(|e| !e.is_paused())
        .unwrap_or(false);

    if should_tick {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    } else {
        Subscription::none()
    }
}
```

---

## Step 11 — Add `PauseTimer` and `ResumeTimer` to `Message`

```rust
pub enum Message {
    // ... existing variants ...

    // Pause / resume
    PauseTimer,    // Pause button clicked — freeze the timer
    ResumeTimer,   // Resume button clicked — unfreeze the timer
}
```

---

## Step 12 — Implement `PauseTimer` and `ResumeTimer` update arms; modify `OpenStopPrompt` and `CancelStop`

```rust
Message::PauseTimer => {
self.pause_active_timer();
}

Message::ResumeTimer => {
self.resume_active_timer();
}
```

Modify `OpenStopPrompt` to pause the timer when the prompt opens:

```rust
Message::OpenStopPrompt => {
let next_status = self.active_entry.as_ref()
.and_then( | e | self.tasks.iter().find( | t | t.id == e.task_id))
.map( | t | t.status.next())
.unwrap_or(TaskStatus::Todo);

self.stop_prompt_open = true;
self.stop_prompt_note.clear();
self.stop_prompt_status = next_status;

// Pause the timer while the prompt is visible.
// This means the user is not billed for the time spent typing the note.
self.pause_active_timer();
}
```

Modify `CancelStop` to resume the timer when the prompt is dismissed:

```rust
Message::CancelStop => {
self.stop_prompt_open = false;
self.stop_prompt_note.clear();

// Resume the timer — the user cancelled, so the work block continues.
self.resume_active_timer();
}
```

**Why pause on OpenStopPrompt?**

The user is filling in a note, not working. If the prompt stays open for two minutes, those
two minutes should not appear in the net work time or in the gross time reported to TANNSS.
Pausing on open and resuming on cancel means the timer accurately reflects only active work.

**Why resume on CancelStop but not ConfirmStop?**

`ConfirmStop` calls `stop_active_timer()`, which finalises any open pause into
`paused_minutes` before computing `minutes`. The timer is stopped entirely — no resume
needed. `CancelStop` leaves the timer running, so the pause must be reversed.

---

## Step 13 — Update `timer_bar()` with Pause/Resume button

The timer bar now needs a third button — Pause when running, Resume when paused.
The Stop button always emits `OpenStopPrompt` (which pauses the timer and opens the modal).

```rust
fn timer_bar(&self) -> Element<'_, Message> {
    if let Some(entry) = &self.active_entry {
        let task_name = self.tasks.iter()
            .find(|t| t.id == entry.task_id)
            .map(|t| t.name.as_str())
            .unwrap_or("Unknown");

        let elapsed = Self::elapsed_display(
            &entry.started_at,
            entry.paused_at.as_deref(),
            entry.paused_minutes,
        );

        // Show Pause when running, Resume when paused.
        let pause_resume_btn = if entry.is_paused() {
            button(text("Resume")).on_press(Message::ResumeTimer)
        } else {
            button(text("Pause")).on_press(Message::PauseTimer)
        };

        row![
            text(format!("{task_name} — {elapsed}")),
            pause_resume_btn,
            button(text("Stop")).on_press(Message::OpenStopPrompt),
        ]
            .spacing(8)
            .padding(8)
            .into()
    } else {
        container(text("No active timer"))
            .padding(8)
            .into()
    }
}
```

---

## Step 14 — Verify

```
cargo run
```

Test in this order:

1. Add a task, start its timer — elapsed increments each second
2. Click **Pause** — elapsed freezes, button shows "Resume"
3. Wait 10 seconds — elapsed does not change
4. Click **Resume** — elapsed starts from where it paused (10s of pause time excluded)
5. Click **Stop** — timer pauses and the prompt appears; elapsed is frozen in the prompt
6. Click **Cancel** — prompt closes, timer resumes from the paused point
7. Click **Stop** again — type a note, click **Stop & Save**
8. Check `~/.tracker/data.json` — the entry should have:
    - `"started_at"`: original start time (unchanged)
    - `"ended_at"`: time you clicked Stop & Save
    - `"minutes"`: net work time (gross minus pauses)
    - `"paused_minutes"`: total pause time in minutes
    - `"notes"`: your note
9. Start a second task while a first is running — first task stops silently (auto-stop).
   Its entry should have `"notes": null` and correct `"minutes"`.

---

## What this phase does not do

- **No visual distinction** for the selected status button — all three look the same
  until a styling pass is done
- **No background dimming** behind the modal — `stack` draws the base layout fully
  visible underneath. A semi-transparent overlay requires a custom widget or styling
- **No keyboard shortcut** to dismiss the prompt or toggle pause — buttons only
- **Pause sub-minute precision** — pause duration is stored in whole minutes. A pause
  shorter than 60 seconds contributes 0 to `paused_minutes`. This is intentional: TANNSS
  accepts whole-minute adjustments, and sub-minute pauses (e.g., a brief phone glance)
  are not worth logging.

---

## Commit message (draft)

```
feat(app): stop prompt with note, status selection, and pause/resume

Stopping a timer opens a modal prompt for a work note and new task status.
The timer pauses while the prompt is open; Cancel resumes it. A Pause/Resume
button is always visible in the timer bar for manual breaks. Net work time
(gross minus all pauses) is stored as minutes; original start time is
preserved alongside paused_minutes for accurate TANNSS reporting.
```
