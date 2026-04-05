# Feature — Phase 4: Stop Prompt and Entry Notes

**Date:** 2026-04-05
**Mode:** FEATURE
**Architecture reference:** `docs/Brainstorm-RustIcedRewrite.md`
**Permanent record:** `docs/features/Phase4StopPrompt.md`

---

## What this phase builds

- Clicking Stop no longer stops the timer immediately — it opens a modal prompt
- The prompt shows the task name, a note field ("what did you accomplish?"), and
  three status buttons (To Do / In Progress / Done)
- The status defaults to the task's natural next status when the prompt opens
- Confirming stops the timer, writes the note to the `TimeEntry`, and updates the task status
- Cancelling dismisses the prompt and leaves the timer running
- Auto-stop (when starting a new timer via the Start button) is silent — no prompt

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

| Area                  | Change                                                                         |
|-----------------------|--------------------------------------------------------------------------------|
| `stop_active_timer()` | Add `note: Option<String>` parameter — all callers updated                     |
| `App` struct          | Add 3 stop-prompt state fields                                                 |
| `Message` enum        | Replace Stop button's `StopTimer` with `OpenStopPrompt`; add 4 prompt messages |
| `update()`            | Implement 5 new/changed arms                                                   |
| `timer_bar()`         | Stop button now emits `OpenStopPrompt`                                         |
| `stop_prompt_view()`  | New method — the modal panel                                                   |
| `view()`              | Conditionally wrap with `stack!` when prompt is open                           |
| imports               | Add `stack`                                                                    |

---

## Step 1 — Modify `stop_active_timer()` to accept a note

The current signature takes no arguments. Add `note: Option<String>` so `ConfirmStop`
can pass the user's text when stopping:

```rust
fn stop_active_timer(&mut self, note: Option<String>) {
    if let Some(mut entry) = self.active_entry.take() {
        use chrono::{DateTime, Utc};
        let now = Utc::now();
        entry.ended_at = Some(now.to_rfc3339());

        let started = DateTime::parse_from_rfc3339(&entry.started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        entry.minutes = Some((now - started).num_minutes().max(0));
        entry.notes = note;   // ← the only new line

        if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *existing = entry;
        }
    }
}
```

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

## Step 8 — Verify

```
cargo run
```

Test in this order:

1. Add a task, start its timer
2. Click Stop — the prompt appears over the main content
3. Click Cancel — prompt closes, timer is still running
4. Click Stop again — prompt appears
5. Type a note, change the status to Done, click Stop & Save
6. Timer clears; task status shows Done in the list
7. Restart the app — load the JSON file at `~/.tracker/data.json` in a text editor
   and confirm the stopped entry has `"notes": "your note"` and `"minutes"` set
8. Start a second task while a first is running — confirm the first stops silently
   (no prompt) and its entry in the JSON has `"notes": null`

---

## What this phase does not do

- **No visual distinction** for the selected status button — all three look the same
  until a styling pass is done
- **No background dimming** behind the modal — `stack` draws the base layout fully
  visible underneath. A semi-transparent overlay requires a custom widget or styling
- **No keyboard shortcut** to dismiss the prompt — Cancel requires a button click

---

## Commit message (draft)

```
feat(app): stop prompt with note and status selection

Stopping a timer now opens a modal prompt for capturing a work note
and setting the task's new status. Auto-stop (on timer switch) remains
silent. Notes are persisted on TimeEntry; task status is updated on confirm.
```
