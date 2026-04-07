# Feature — Migrate All Note Fields to `text_editor`

**Date:** 2026-04-06

---

## What and why

`text_input` is a single-line widget. Four fields in the app accept content that needs
multiple lines:

| Location                | Field             | Current      | Target        |
|-------------------------|-------------------|--------------|---------------|
| `ui/note_modal.rs`      | Live session note | `text_input` | `text_editor` |
| `ui/stop_prompt.rs`     | Stop note         | `text_input` | `text_editor` |
| `ui/review.rs` (inline) | Entry note        | `text_input` | `text_editor` |
| `ui/task_list.rs`       | Task description  | `text_input` | `text_editor` |

`status_bar.rs` (add task name) stays as `text_input` — task names are always one line.

**Current state of `app.rs` and `message.rs`** — the migration is partially done but
inconsistent. Here is the exact status before this guide:

| Field                      | Struct type | message.rs | Arms                                              |
|----------------------------|-------------|------------|---------------------------------------------------|
| `stop_prompt_note`         | `Content` ✓ | `Action` ✓ | `.clear()` ✗ — no such method                     |
| `note_modal_content`       | `Content` ✓ | `Action` ✓ | `.clear()` ✗ — no such method                     |
| `editing_note_content`     | `Content` ✓ | `Action` ✓ | Arms still reference old `editing_note_text` name |
| `edit_description_content` | `Content` ✓ | `String` ✗ | Arms have typo + stale `edit_description` name    |

`Content` has no `.clear()` method — the correct reset is reassigning `Content::new()`.
Every `.clear()` call on a Content field is a compile error.

---

## Step 1 — Fix all `Content` issues in `src/app.rs`

### 1a — Change `editing_note_text` field

In the `App` struct, replace:

```rust
pub(crate) editing_note_text: String,
```

with:

```rust
pub(crate) editing_note_content: iced::widget::text_editor::Content,
```

In `App::new()`, replace:

```rust
editing_note_text: String::new(),
```

with:

```rust
editing_note_content: iced::widget::text_editor::Content::new(),
```

### 1b — Replace `.clear()` on Content fields

`Content` has no `.clear()` method. Every `.clear()` call on a Content field must become
a reassignment to `Content::new()`. There are several in `update()` — find them all by
searching for `.clear()` and checking which field it is called on.

**`stop_prompt_note.clear()`** — appears three times (in `StartTimer`'s auto-stop path,
in `ConfirmStop`, and in `CancelStop`). Replace each with:

```rust
self .stop_prompt_note = iced::widget::text_editor::Content::new();
```

**`note_modal_text.clear()`** — appears in `SaveNoteModal` and `CancelNoteModal`.
Replace each with:

```rust
self .note_modal_text = iced::widget::text_editor::Content::new();
```

### 1c — Update `OpenEditNote` arm

Replace:

```rust
self .editing_note_id = Some(entry_id);
self .editing_note_text = current_note;
```

with:

```rust
self .editing_note_id = Some(entry_id);
self .editing_note_content = iced::widget::text_editor::Content::with_text( & current_note);
```

### 1d — Update `EditNoteChanged` arm

Replace:

```rust
Message::EditNoteChanged(value) => {
self.editing_note_text = value;
}
```

with:

```rust
Message::EditNoteChanged(action) => {
self.editing_note_content.perform(action);
}
```

### 1e — Update `SaveEditNote` arm

Replace the note-reading logic:

```rust
let s = self .editing_note_text.trim().to_string();
if s.is_empty() { None } else { Some(s) }
```

with:

```rust
let s = self .editing_note_content.text();
let s = s.trim().to_string();
if s.is_empty() { None } else { Some(s) }
```

And replace `self.editing_note_text.clear()` with:

```rust
self .editing_note_content = iced::widget::text_editor::Content::new();
```

### 1f — Update `CancelEditNote` and `RequestDeleteEntry` arms

Both call `self.editing_note_text.clear()`. Replace each with:

```rust
self .editing_note_content = iced::widget::text_editor::Content::new();
```

### 1g — Update navigation arms

`OpenReview`, `CloseReview`, `ReviewPrevDay`, and `ReviewNextDay` all call
`self.editing_note_text.clear()`. Replace each with:

```rust
self .editing_note_content = iced::widget::text_editor::Content::new();
```

---

## Step 2 — Fix `src/ui/note_modal.rs`

Also note: the current file has a bug — both buttons say "Save". Fix that here too.

Replace the entire file:

```rust
use crate::app::{App, Message};
use iced::widget::{button, container, row, text, text_editor};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let panel = iced::widget::column![
        text("Session note"),
        text_editor(&app.note_modal_text)
            .on_action(Message::NoteModalChanged)
            .height(Length::Fixed(160.0)),
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

**Why `height(Length::Fixed(160.0))`?**
`text_editor` needs an explicit height — unlike `text_input` which is always one line tall,
a text editor has no natural height when placed inside a column. Without a fixed height it
collapses to zero. 160px gives roughly 5–7 lines of text, which is enough for a session note
without dominating the modal.

---

## Step 3 — Fix `src/ui/stop_prompt.rs`

Replace the entire file:

```rust
use crate::app::{App, Message};
use crate::data::task::TaskStatus;
use iced::widget::{button, container, pick_list, row, text, text_editor};
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
        text_editor(&app.stop_prompt_note)
            .on_action(Message::StopPromptNoteChange)
            .height(Length::Fixed(120.0)),
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

The stop prompt note is intentionally slightly shorter (120px vs 160px) — the stop prompt
has more UI elements and less vertical room to work with.

---

## Step 4 — Fix `src/ui/review.rs`

The inline note editor is different from the modals — it lives inside a table row alongside
other columns. Using `text_editor` here means the row expands vertically when editing, which
is fine. The Save/Cancel buttons move below the editor (instead of beside it) so the editor
has room to breathe.

**Change the import line:**

```rust
use iced::widget::{button, column, container, row, scrollable, text, text_editor};
```

Remove `text_input`, add `text_editor`. Keep `column` — you'll need it for the edit widget.

**Change the editing branch of `note_widget`:**

Replace:

```rust
let note_widget: Element<Message> = if app.editing_note_id == Some(entry.id) {
row ! [
text_input("Add a note...", & app.editing_note_text)
.on_input(Message::EditNoteChanged)
.on_submit(Message::SaveEditNote)
.width(Length::Fill),
button(text("Save")).on_press(Message::SaveEditNote),
button(text("Cancel")).on_press(Message::CancelEditNote),
]
.spacing(4)
.into()
```

with:

```rust
let note_widget: Element<Message> = if app.editing_note_id == Some(entry.id) {
column ! [
text_editor( & app.editing_note_content)
.on_action(Message::EditNoteChanged)
.height(Length::Fixed(80.0)),
row ! [
button(text("Save")).on_press(Message::SaveEditNote),
button(text("Cancel")).on_press(Message::CancelEditNote),
]
.spacing(4),
]
.spacing(4)
.into()
```

**Why 80px here?** The review is a compact list — 80px gives around 2–3 lines, enough to
read and edit a note without the row taking over the screen. The modals can afford more
space; inline editors should be compact.

**Why `column` instead of `row`?** With a `row`, the Save/Cancel buttons would sit at the
same height as a tiny editor, which looks cramped. Moving them below with a `column` makes
both the editor and the buttons usable.

---

## Step 5 — Fix the task description field

The description field is in a different part of the codebase from the note fields, and its
migration has more errors to untangle: a wrong field name used in several arms, a typo in
one arm, `.clear()` on a Content field, and the message still carrying `String` instead of
`Action`.

### 5a — Fix `message.rs`

Change:

```rust
EditDescriptionChanged(String),
```

to:

```rust
EditDescriptionChanged(iced::widget::text_editor::Action),
```

### 5b — Fix all broken arms in `src/app.rs`

There are six arms that reference the description field incorrectly. Go through each:

**`OpenEditTask` arm** — currently assigns a `String` to the wrong field name:

```rust
// Wrong — edit_description does not exist, and you can't assign a String to Content
self .edit_description = task.description.clone().unwrap_or_default();
```

Replace with:

```rust
self .edit_description_content = iced::widget::text_editor::Content::with_text(
& task.description.clone().unwrap_or_default()
);
```

**`CloseEditTask` arm** — has a typo (`conent` instead of `content`) and uses `.clear()`:

```rust
// Wrong — typo in field name, and Content has no .clear()
self .edit_description_conent = iced::widget::text_editor::Content::new();
```

Replace with:

```rust
self .edit_description_content = iced::widget::text_editor::Content::new();
```

**`SaveEditTask` arm** — reads from the wrong field name:

```rust
// Wrong — edit_description is a String that no longer exists
let desc = self .edit_description.trim().to_string();
```

Replace with:

```rust
let raw = self .edit_description_content.text();
let desc = raw.trim().to_string();
```

The `self.edit_description_content = Content::new()` line further down in the same arm is
already correct — leave it.

**`RequestDeleteTask` arm** — calls `.clear()` on the wrong field name:

```rust
// Wrong — edit_description does not exist
self .edit_description.clear();
```

Replace with:

```rust
self .edit_description_content = iced::widget::text_editor::Content::new();
```

**`ConfirmDeleteTask` arm** — calls `.clear()` on `stop_prompt_note` (a Content field):

```rust
// Wrong — Content has no .clear()
self .stop_prompt_note.clear();
```

Replace with:

```rust
self .stop_prompt_note = iced::widget::text_editor::Content::new();
```

**`CancelDeleteTask` arm** — assigns a String to the wrong field name:

```rust
// Wrong — edit_description does not exist
self .edit_description = task.description.clone().unwrap_or_default();
```

Replace with:

```rust
self .edit_description_content = iced::widget::text_editor::Content::with_text(
& task.description.clone().unwrap_or_default()
);
```

### 5c — Fix `src/ui/task_list.rs`

In `edit_row`, replace the `text_input` with `text_editor`. The description field is inside
the task edit panel — it sits below the urgency/importance checkboxes.

Change the import line:

```rust
// Before
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
// After
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_editor};
```

Replace the description field inside `edit_row`:

```rust
// Before
text_input("Description (optional)", & app.edit_description)
.on_input(Message::EditDescriptionChanged)
.on_submit(Message::SaveEditTask)
```

```rust
// After
text_editor( & app.edit_description_content)
.on_action(Message::EditDescriptionChanged)
.height(Length::Fixed(100.0))
.placeholder("Description (optional)")
```

**Why 100px?** The description is inside a compact inline panel, not a full modal. 100px
gives 3–4 lines — enough for a short code snippet or a few bullet points — without making
the task row dominate the list. Adjust to taste.

**Why no `on_submit`?** `text_editor` has no `on_submit`. The Save button in the row above
the editor submits the whole task edit. The keyboard shortcut in Step 6 handles Ctrl+Enter.

---

## Step 6 — Update `subscription()` for keyboard submit

The keyboard subscription handles Ctrl+Enter to submit. Now that there are four `text_editor`
fields — note modal, stop prompt, review inline editor, and task description — extend it to
cover all four. Each maps to a different save message.

In `src/app.rs`, replace the `submit_note` subscription block with:

```rust
let submit_note = {
let save_msg = if self.note_modal_open {
Some(Message::SaveNoteModal)
} else if self.stop_prompt_open {
Some(Message::ConfirmStop)
} else if self.editing_note_id.is_some() {
Some(Message::SaveEditNote)
} else if self.editing_task_id.is_some() {
Some(Message::SaveEditTask)
} else {
None
};

if let Some(msg) = save_msg {
keyboard::on_key_press( move | key, modifiers | {
if matches ! (key, keyboard::Key::Named(Named::Enter)) {
if modifiers.control() | | modifiers.shift() {
return Some(msg.clone());
}
}
None
})
} else {
iced::Subscription::none()
}
};
```

**Why exactly one subscription?** The four cases are mutually exclusive — you cannot have
two of these panels open at the same time. Computing one `Option<Message>` and registering
a single subscription only when something is open is both simpler and more efficient than
batching four separate keyboard listeners.

**`msg.clone()` in the closure:** The closure captures `msg` by value. `Message` derives
`Clone`, so this is fine. The closure is called once per key event, not in a hot loop.

**Order matters:** The `if/else if` chain resolves to the first true condition. The order
here — note modal, stop prompt, review editor, task editor — reflects how they are layered
in `view()`: modals first, inline editors last. If somehow two flags were set (a bug), the
modal takes priority.

---

## Step 7 — Verify

```
cargo check
cargo run
```

Test each editor in order:

1. **Note modal** — start a timer, click Note. Type a multi-line note using Enter. Ctrl+Enter
   saves and closes. Reopen — note is pre-filled.
2. **Stop prompt** — click Stop. The note field should be pre-filled from the live note.
   Type more, Enter adds lines, Ctrl+Enter saves and stops.
3. **Review inline editor** — open Review, click Edit on an entry. The editor opens inline.
   Type a multi-line note. Ctrl+Enter or Save button saves. The note is displayed back in
   the row.
4. **Task description** — click Edit on a task. The description field should be a text
   editor box below the checkboxes. Type a multi-line description. Ctrl+Enter or the Save
   button saves. Open Edit again — the description should be pre-filled.

Check `~/.tracker/data.json` after each — notes and descriptions with embedded newlines
are stored as JSON strings with `\n` characters, which is correct.
