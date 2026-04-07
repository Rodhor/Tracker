# Feature Plan — Phase 8 + 9: Stop Prompt Redesign and Live Notes

**Date:** 2026-04-06
**Permanent records:** `docs/features/Phase8StopPromptRedesign.md`, `docs/features/Phase9LiveNotes.md`

---

## What these phases build

**Phase 8 — Stop prompt redesign:**

Replace the three identical status buttons (To Do / In Progress / Done) with a `pick_list`
dropdown. This is a compact widget that clearly shows the selected status and opens a list on
click. The note field is also pre-filled from any live notes already written for the session.

**Phase 9 — Live notes modal:**

Add a "Note" button to the timer bar. Clicking it opens a modal overlay where you can type
notes while the timer keeps running. Those notes are saved to the active entry immediately
and pre-fill the stop prompt note field when you stop.

**Why these two phases together:**

Phase 9's output (notes on the active entry) feeds directly into Phase 8's input (the stop
prompt note field). If you build Phase 8 first without the pre-fill, you have to go back and
change `OpenStopPrompt` anyway. Building them together means touching `OpenStopPrompt` only
once.

---

## What already exists

- `stop_prompt.rs` — the stop prompt view, currently with three status buttons
- `timer_bar.rs` — timer bar view, currently with Pause/Resume and Stop
- `TaskStatus` in `data/task.rs` — has `Clone`, `PartialEq`, `label()`; needs `Display`
- `active_entry.notes: Option<String>` — the field exists and is written on stop; it is
  never written during a session today

---

## New files

```
src/ui/note_modal.rs    — new modal view for live notes
```

---

## Changes overview

| File                    | Change                                                                                          |
|-------------------------|-------------------------------------------------------------------------------------------------|
| `src/data/task.rs`      | Add `Display` impl for `TaskStatus`                                                             |
| `src/message.rs`        | Add 4 new messages for the note modal                                                           |
| `src/app.rs`            | Add 2 new state fields; implement 4 new update() arms; modify `OpenStopPrompt`; update `view()` |
| `src/ui/stop_prompt.rs` | Replace status buttons with `pick_list`; remove now-unused import                               |
| `src/ui/timer_bar.rs`   | Add Note button with note-exists indicator                                                      |
| `src/ui/note_modal.rs`  | New file — the live notes modal view                                                            |
| `src/ui/mod.rs`         | Declare `pub mod note_modal;`                                                                   |

---

## Step 1 — Add `Display` to `TaskStatus` (`src/data/task.rs`)

`pick_list` renders its items as text. iced calls `.to_string()` on each item, which
requires `std::fmt::Display`. Without it, the code will not compile.

Add this `impl` block below the existing `impl TaskStatus`:

```rust
impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
```

**Why use `label()` rather than deriving `Debug` or writing the string directly?**

`label()` is already the canonical display text for the UI ("To Do", "In Progress", "Done").
Using it here means there is exactly one place where the display text is defined — if you
ever change a label, both the task list and the pick_list update together.

**Why not `#[derive(Display)]`?**

There is no built-in derive for `Display` in Rust's standard library. `derive(Debug)` gives
`"Todo"` / `"InProgress"` / `"Done"` — the raw variant names, not the user-readable labels.
Writing the impl explicitly is two lines and stays in your control.

---

## Step 2 — Add new messages (`src/message.rs`)

Add four variants to the `Message` enum, in the stop prompt section:

```rust
// Note modal (live notes while a timer is running)
OpenNoteModal,
NoteModalChanged(String),
SaveNoteModal,
CancelNoteModal,
```

These follow the same naming pattern as the stop prompt messages: one to open, one for
input changes, one to confirm, one to cancel. Consistency in naming makes `update()` easier
to scan.

---

## Step 3 — Add state fields to `App` (`src/app.rs`)

Add two fields to the `App` struct, alongside the existing stop prompt fields:

```rust
// Note modal state
pub(crate) note_modal_open: bool,
pub(crate) note_modal_text: String,
```

Initialise them in `App::new()`:

```rust
note_modal_open: false,
note_modal_text: String::new(),
```

**Why separate state rather than reusing `stop_prompt_note`?**

Both modals can theoretically exist in sequence within the same session (you write a live
note, then stop). They have distinct semantics: `stop_prompt_note` is the final end-of-session
note; `note_modal_text` is a scratchpad that feeds into it. Sharing state would create
invisible coupling between two independent operations.

---

## Step 4 — Implement the new `update()` arms (`src/app.rs`)

Add these four arms, keeping them in the same section as the stop prompt arms:

```rust
Message::OpenNoteModal => {
// Pre-fill with whatever notes already exist on the active entry
self.note_modal_text = self
.active_entry
.as_ref()
.and_then( | e | e.notes.clone())
.unwrap_or_default();
self.note_modal_open = true;
}
Message::NoteModalChanged(value) => {
self.note_modal_text = value;
}
Message::SaveNoteModal => {
if let Some(entry) = self.active_entry.as_mut() {
let note = self.note_modal_text.trim().to_string();
entry.notes = if note.is_empty() { None } else { Some(note) };
// Sync to self.entries so the data is consistent if the app is quit
if let Some(existing) = self.entries.iter_mut().find( | e| e.id == entry.id) {
* existing = entry.clone();
}
}
self.note_modal_open = false;
self.note_modal_text.clear();
self.save();
}
Message::CancelNoteModal => {
self.note_modal_open = false;
self.note_modal_text.clear();
}
```

**Why does `SaveNoteModal` sync to `self.entries` manually?**

`active_entry` is a clone of the entry kept separately for fast access (it is the "live"
copy). `self.entries` is the authoritative Vec that gets serialised to disk. They can drift
if you update one without updating the other. Every write to `active_entry` must be mirrored
to `self.entries` — the same pattern used in `pause_active_timer()` and `resume_active_timer()`.

**Why does `CancelNoteModal` not need to restore the previous note?**

The modal pre-fills from `active_entry.notes` when it opens and writes only on Save.
Cancelling simply clears the scratch text and closes — `active_entry.notes` was never
touched, so there is nothing to restore.

---

## Step 5 — Modify `OpenStopPrompt` to pre-fill the note (`src/app.rs`)

The existing `OpenStopPrompt` arm clears the note field:

```rust
self .stop_prompt_note.clear();
```

Replace that line with:

```rust
// Pre-fill from any live note already written for this session.
// If no live note exists, this is an empty string — same behaviour as before.
self .stop_prompt_note = self
.active_entry
.as_ref()
.and_then( | e| e.notes.clone())
.unwrap_or_default();
```

**Why pre-fill rather than always starting empty?**

Live notes and stop notes serve the same purpose: capturing what happened in the session.
If you wrote a note mid-session, the stop prompt should start from it, not throw it away.
You can extend it, shorten it, or replace it — but you should not have to re-type it.

---

## Step 6 — Update `view()` to layer the note modal (`src/app.rs`)

The current `view()` stacks the stop prompt conditionally. Extend it to also handle the
note modal. The two modals are mutually exclusive — you cannot open both at once.

Replace the current conditional at the end of `view()`:

```rust
if self .stop_prompt_open {
stack![base, ui::stop_prompt::view(self)].into()
} else {
base.into()
}
```

With:

```rust
match ( self .stop_prompt_open, self .note_modal_open) {
(true, _) => stack ! [base, ui::stop_prompt::view( self )].into(),
(_, true) => stack ! [base, ui::note_modal::view( self )].into(),
_ => base.into(),
}
```

**Why `match` instead of nested `if`?**

Two booleans, three outcomes. A `match` on a tuple makes all three cases explicit and
exhaustive — the compiler will tell you if you add a third modal and forget to handle it.
Nested `if/else` would express the same logic but obscures the mutual exclusion.

**Why stop prompt takes priority (`(true, _)`)?**

If somehow both flags were set (a bug), showing the stop prompt is the safer choice —
it is the operation that ends the timer. The note modal is non-destructive.

---

## Step 7 — Replace status buttons with `pick_list` (`src/ui/stop_prompt.rs`)

**How `pick_list` works in iced:**

```rust
// Why: pick_list needs a slice of all options, the current selection, and a callback.
// The callback is called with the newly selected value when the user picks one.
pick_list(
& [TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done][..],
Some(app.stop_prompt_status.clone()),   // current selection — None shows a placeholder
Message::StopPromptStatusChanged,       // fn(TaskStatus) -> Message
)
```

`pick_list` calls `.to_string()` on each item to render the label — which is why Step 1
(adding `Display`) must come first.

The second argument is `Option<T>`. Passing `Some(value)` shows the current selection.
Passing `None` would show a blank/placeholder — not what you want here, since a status is
always selected when the prompt opens.

Replace the entire `view` function in `src/ui/stop_prompt.rs` with:

```rust
use crate::app::{App, Message};
use crate::data::task::TaskStatus;
use iced::widget::{button, container, pick_list, row, text, text_input};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let task_name = app
        .active_entry
        .as_ref()
        .and_then(|e| app.tasks.iter().find(|t| t.id == e.task_id))
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown task");

    let panel = iced::widget::column![
        text(format!("Stopping: {task_name}")),
        text_input("What did you accomplish?", &app.stop_prompt_note)
            .on_input(Message::StopPromptNoteChange)
            .on_submit(Message::ConfirmStop),
        row![
            text("Status:"),
            pick_list(
                &[TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done][..],
                Some(app.stop_prompt_status.clone()),
                Message::StopPromptStatusChanged,
            ),
        ]
        .spacing(8),
        row![
            button(text("Stop and save")).on_press(Message::ConfirmStop),
            button(text("Cancel")).on_press(Message::CancelStop),
        ]
        .spacing(7),
    ]
        .spacing(11)
        .padding(23);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
```

The "Mark as: X" label row is gone — the pick_list communicates the selection directly.
Six rows become four. No `column` import needed — `iced::widget::column!` is called with
the full path as before.

**If the compiler says `pick_list` requires `PartialEq` or `Clone`:** `TaskStatus` already
derives both. If it complains about `Display`, Step 1 was not applied yet.

---

## Step 8 — Add the Note button to the timer bar (`src/ui/timer_bar.rs`)

Add a Note button between the task name/elapsed display and the Pause/Resume button.
Show a small indicator if notes already exist on the active entry:

```rust
let note_label = if entry.notes.is_some() { "Note ●" } else { "Note" };

row![
    text(format!("{task_name} - {elapsed}")).width(Length::Fill),
    button(text(note_label)).on_press(Message::OpenNoteModal),
    pause_resume_btn,
    button(text("Stop")).on_press(Message::OpenStopPrompt),
]
```

**Why a bullet (●) rather than a count or full label?**

It communicates "something is there" without taking up space. The Phase 13 styling pass can
turn this into something more polished. For now, a single character is enough to answer the
question "did I write anything yet?"

Also add `Message` to the import if it is not already there (it is — the file uses it for
other buttons).

---

## Step 9 — Create `src/ui/note_modal.rs`

```rust
use crate::app::{App, Message};
use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let panel = iced::widget::column![
        text("Session note"),
        text_input("What are you working on?", &app.note_modal_text)
            .on_input(Message::NoteModalChanged)
            .on_submit(Message::SaveNoteModal),
        row![
            button(text("Save")).on_press(Message::SaveNoteModal),
            button(text("Cancel")).on_press(Message::CancelNoteModal),
        ]
        .spacing(8),
    ]
        .spacing(12)
        .padding(24);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
```

**Why does the note modal not pause the timer?**

The stop prompt pauses the timer because stopping a timer is a deliberate, end-of-session
action — the time spent filling in the note is not billable work. Writing a live note is
different: you are still working, just capturing context. The timer should keep running.

The note modal is also designed to be fast: open, type a phrase, save, close, done. Pausing
and resuming for that operation adds friction without benefit.

---

## Step 10 — Declare the new module (`src/ui/mod.rs`)

Add one line:

```rust
pub mod note_modal;
```

---

## Step 11 — Verify

```
cargo check
cargo run
```

Test in this order:

1. Start a timer. The timer bar should show a "Note" button.
2. Click "Note" — the note modal opens. The timer should still be counting.
3. Type a note. Click "Save". The modal closes. The timer bar button now shows "Note ●".
4. Click "Note" again — the modal pre-fills with your saved text.
5. Click "Cancel" — the modal closes. No change.
6. Click "Stop" — the stop prompt opens. The note field should be pre-filled with your live note.
7. Check the status pick_list shows the correct default status.
8. Change the status in the pick_list, confirm. Check the task status updated.
9. Start a timer, click Stop without writing any live notes — stop prompt note field should be empty.
10. Check `~/.tracker/data.json` — entries should have `"notes"` set correctly.

---

## What this phase does not do

- **No keyboard shortcut** to open the note modal — `N` key is planned for Phase 11
- **No rich text or multi-line input** — `text_input` is single-line; Phase 13 can revisit
- **No visual distinction** between the selected status in the pick_list and other statuses
  beyond what the widget itself provides — Phase 13 handles styling
- **No note history** — the live note replaces itself each time you save; there is no log
  of intermediate notes within a session

---

## Commit message (draft)

```
feat(app): live notes modal and stop prompt redesign

Live notes: a Note button in the timer bar opens a modal overlay where
notes can be written without stopping the timer. Notes persist to the
active entry immediately. The stop prompt note field is pre-filled from
any live note when stopping.

Stop prompt: three identical status buttons replaced with a pick_list
dropdown. Requires Display on TaskStatus. Layout reduced from six rows
to four.
```
