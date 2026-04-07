# Feature Plan — Phase 10: Quick-Add Filter Modal

**Date:** 2026-04-06
**Permanent record:** `docs/features/Phase10QuickAdd.md`

---

## What this phase builds

A modal overlay that opens from a keyboard shortcut (Phase 11) or a button. A single text
input filters the task list in real time. Arrow keys move a selection cursor through the
results. Enter starts the timer for the selected task. If the typed text matches nothing,
Enter creates a new task and starts its timer immediately. Escape closes without action.

This is the primary daily workflow surface — one shortcut, filter or type a name, Enter.
No mouse required.

---

## What already exists

- `tasks: Vec<AppTask>` in `App` — the full task list, already rendered in `task_list.rs`
- `StartTimer(Uuid)` message — already handles auto-stopping the previous timer
- `match (stop_prompt_open, note_modal_open)` in `view()` — the layering pattern for modals
- `iced::event::listen_with` subscription — already in `subscription()` for Ctrl+Enter

---

## New files

```
src/ui/quick_add.rs    — the filter modal view
```

---

## Changes overview

| File                  | Change                                                                                        |
|-----------------------|-----------------------------------------------------------------------------------------------|
| `src/message.rs`      | Add 5 new messages for quick-add                                                              |
| `src/app.rs`          | Add 3 state fields; implement 5 new `update()` arms; update `view()`; update `subscription()` |
| `src/ui/quick_add.rs` | New file — the filter modal view                                                              |
| `src/ui/mod.rs`       | Add `pub mod quick_add;`                                                                      |

---

## Step 1 — Add new messages (`src/message.rs`)

Add these five variants after the live notes section:

```rust
// Quick-add modal
OpenQuickAdd,
CloseQuickAdd,
QuickAddInputChanged(String),
QuickAddMoveUp,
QuickAddMoveDown,
QuickAddConfirm,
```

**Why `String` not `Action` for `QuickAddInputChanged`?**

Task names are always one line. `text_input` (single-line) is the right widget here — the
same widget used for `new_task_input`. Single-line fields carry `String` in their messages;
multi-line `text_editor` fields carry `Action`. The distinction maps directly to widget type.

**Why no `QuickAddSelect(usize)` message for mouse clicks?**

Clicking a task row in the modal immediately fires `StartTimer(task_id)` — the same message
that clicking Start in the task list fires. `StartTimer` already closes the active entry and
starts a new one. We just need `StartTimer` to also close the quick-add modal (see Step 3).
A separate Select message would add a two-step "click to select, then confirm" flow that
contradicts the modal's purpose: fast.

---

## Step 2 — Add state fields to `App` (`src/app.rs`)

Add three fields to the `App` struct, grouped with the other modal states:

```rust
// Quick-add modal
pub(crate) quick_add_open: bool,
pub(crate) quick_add_input: String,
pub(crate) quick_add_selected: usize,
```

Initialise in `App::new()`:

```rust
quick_add_open: false,
quick_add_input: String::new(),
quick_add_selected: 0,
```

**What `quick_add_selected` indexes:**

The filtered task list is computed in `view()` — it is not stored in `App`. `quick_add_selected`
is a cursor into that list. Index `0` through `filtered.len() - 1` points to existing tasks.
When `quick_add_input` is non-empty, index `filtered.len()` points to the "Create new" option
at the bottom of the list.

MoveUp/MoveDown must clamp to the valid range (see Step 3).

---

## Step 3 — Implement `update()` arms (`src/app.rs`)

Add these arms. Place them with the stop prompt and note modal arms.

```rust
Message::OpenQuickAdd => {
    self.quick_add_open = true;
    self.quick_add_input.clear();
    self.quick_add_selected = 0;
    // Return a Task that focuses the text input so the user can type immediately.
    // The input must have a matching Id in the view (see Step 5).
    use iced::widget::text_input;
    return text_input::focus(text_input::Id::new("quick_add_input"));
}
Message::CloseQuickAdd => {
    self.quick_add_open = false;
    self.quick_add_input.clear();
    self.quick_add_selected = 0;
}
Message::QuickAddInputChanged(value) => {
    self.quick_add_input = value;
    self.quick_add_selected = 0; // reset to top on every keystroke
}
Message::QuickAddMoveUp => {
    if self.quick_add_selected > 0 {
        self.quick_add_selected -= 1;
    }
}
Message::QuickAddMoveDown => {
    // The upper bound depends on filtered count — computed in the view.
    // We cannot access that here, so we just increment and let view clamp the highlight.
    // The view renders the cursor only up to max_index, so over-incrementing is harmless.
    self.quick_add_selected = self.quick_add_selected.saturating_add(1);
}
Message::QuickAddConfirm => {
    if !self.quick_add_open {
        // Guard against spurious fires when modal is closed
    } else {
        let input = self.quick_add_input.trim().to_string();
        let filtered: Vec<_> = self
            .tasks
            .iter()
            .filter(|t| t.name.to_lowercase().contains(&input.to_lowercase()))
            .collect();

        let task_id = if self.quick_add_selected < filtered.len() {
            // Start existing task
            filtered[self.quick_add_selected].id
        } else if !input.is_empty() {
            // Create new task with the typed name
            let task = crate::data::task::Task::new(input.clone());
            let id = task.id;
            self.tasks.push(task);
            id
        } else {
            // Empty input, nothing selected — do nothing
            return Task::none();
        };

        self.quick_add_open = false;
        self.quick_add_input.clear();
        self.quick_add_selected = 0;

        // Re-use existing StartTimer logic — it handles auto-stop automatically
        return Task::done(Message::StartTimer(task_id));
    }
}
```

**Also: close quick-add when `StartTimer` fires** (handles mouse clicks on rows):

In the existing `StartTimer` arm, add at the top:

```rust
Message::StartTimer(task_id) => {
    // Close quick-add if it is open (mouse click on a result row)
    self.quick_add_open = false;
    self.quick_add_input.clear();
    self.quick_add_selected = 0;
    // ... rest of the existing StartTimer arm unchanged
```

**Why does `QuickAddMoveDown` not clamp here?**

The filtered list is a view-time computation — `App` does not store it. Clamping in `update()`
would require duplicating the filter logic. Instead, the view just does not render a highlight
beyond `filtered.len()`, making the over-increment invisible. The cursor resets to 0 on every
input change anyway, so it cannot accumulate unboundedly.

**Why does `QuickAddConfirm` re-derive the filtered list?**

`update()` runs once per message. It cannot read the filtered list that `view()` computed
because `view()` runs separately. Duplicating the filter here is the only option — it is
a short, pure computation with no side effects.

---

## Step 4 — Update `view()` to layer the quick-add modal (`src/app.rs`)

Replace the current `match (self.stop_prompt_open, self.note_modal_open)` block with a
three-arm tuple match:

```rust
match (self.stop_prompt_open, self.note_modal_open, self.quick_add_open) {
    (true, _, _) => stack![base, ui::stop_prompt::view(self)].into(),
    (_, true, _) => stack![base, ui::note_modal::view(self)].into(),
    (_, _, true) => stack![base, ui::quick_add::view(self)].into(),
    _ => base.into(),
}
```

**Why keep the tuple match instead of if-else?**

Each arm is mutually exclusive by design — at most one modal is open at a time. A tuple match
makes the mutual exclusion explicit and visible: you can see all combinations in one place.
When a fourth modal is added, the compiler will not complain (the `_` wildcard absorbs it),
but the pattern stays readable. If-else chains hide the structure inside control flow.

**Priority order:** stop prompt first (ends a timer — most critical), then note modal, then
quick-add (least destructive — it is safe to show last).

---

## Step 5 — Create `src/ui/quick_add.rs`

```rust
use crate::app::{App, Message};
use crate::data::task::Task;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

// The Id used by OpenQuickAdd to focus the input on open.
// Must match the .id() call on the text_input widget below.
fn input_id() -> iced::widget::text_input::Id {
    iced::widget::text_input::Id::new("quick_add_input")
}

pub fn view(app: &App) -> Element<'_, Message> {
    let input_text = app.quick_add_input.as_str();

    // Filter tasks whose names contain the input (case-insensitive).
    // Empty input shows all tasks.
    let filtered: Vec<&Task> = app
        .tasks
        .iter()
        .filter(|t| t.name.to_lowercase().contains(&input_text.to_lowercase()))
        .collect();

    // Clamp the selection cursor to the valid range for rendering.
    // filtered.len() is the "Create new" slot — only valid when input is non-empty.
    let max_idx = if input_text.is_empty() {
        filtered.len().saturating_sub(1)
    } else {
        filtered.len() // includes "Create new" at this index
    };
    let selected = app.quick_add_selected.min(max_idx);

    // Build the result rows
    let mut rows: Vec<Element<Message>> = filtered
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let label = if i == selected {
                format!("▶  {}", task.name)
            } else {
                format!("   {}", task.name)
            };
            // Clicking a row fires StartTimer directly — fast path for mouse users.
            // StartTimer will also close the quick-add modal (see Step 3).
            button(text(label))
                .width(Length::Fill)
                .on_press(Message::StartTimer(task.id))
                .into()
        })
        .collect();

    // "Create new" row — only shown when the input is non-empty
    if !input_text.is_empty() {
        let create_label = if selected == filtered.len() {
            format!("▶  Create \"{}\"", input_text)
        } else {
            format!("   Create \"{}\"", input_text)
        };
        rows.push(
            button(text(create_label))
                .width(Length::Fill)
                .on_press(Message::QuickAddConfirm)
                .into(),
        );
    }

    let list = scrollable(
        column(rows).spacing(2),
    )
    .height(Length::Fixed(240.0));

    let panel = column![
        text("Start a task"),
        text_input("Filter or create new...", &app.quick_add_input)
            .id(input_id())
            .on_input(Message::QuickAddInputChanged)
            .on_submit(Message::QuickAddConfirm),
        list,
        row![
            button(text("Cancel")).on_press(Message::CloseQuickAdd),
        ]
        .spacing(8),
    ]
    .spacing(12)
    .padding(24)
    .width(Length::Fixed(420.0));

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
```

**Why `scrollable` with a fixed height?**

The task list can grow long. Without a fixed height the list expands until it overflows the
window. 240px fits roughly 8–10 rows — enough to see useful results without the modal
dominating the screen. The fixed width (420px) gives the modal a well-defined footprint that
will sit cleanly in the centre of any reasonable window size.

**Why `▶` instead of a real highlight style?**

Phase 13 handles styling. For now, a prefix character communicates "selected" without
requiring any changes to `style.rs`. It can be replaced in one place when that pass happens.

**Why does clicking a task row fire `StartTimer` directly instead of `QuickAddConfirm`?**

`StartTimer` is already the correct message to start a timer for an existing task. It handles
auto-stopping the previous timer, creating the entry, and saving. Routing through a separate
confirm message would add a pointless hop. The only thing clicking a row needs to do beyond
`StartTimer` is close the modal — and that is added to the `StartTimer` arm in Step 3.

---

## Step 6 — Declare the module (`src/ui/mod.rs`)

Add one line:

```rust
pub mod quick_add;
```

---

## Step 7 — Update `subscription()` for arrow key navigation (`src/app.rs`)

The existing `event::listen_with` handles Ctrl+Enter. Extend it to also handle arrow keys
and Escape when the quick-add modal is open.

**The constraint:** `event::listen_with` takes a function pointer — it cannot capture `self`
or `app`. All state-based routing happens in `update()`. The function only maps keys to messages.

Replace the `submit` subscription block with:

```rust
let keys = iced::event::listen_with(|event, _status, _window| {
    use iced::keyboard::{self, key::Named};

    if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
        match key {
            // Ctrl+Enter or Shift+Enter — submit whichever editor is active
            keyboard::Key::Named(Named::Enter)
                if modifiers.control() || modifiers.shift() =>
            {
                Some(Message::SubmitActiveEditor)
            }
            // Arrow keys — used for quick-add list navigation.
            // Emitted even when a text input is focused (status may be Captured).
            // update() is a no-op for these when quick_add is not open.
            keyboard::Key::Named(Named::ArrowUp) => Some(Message::QuickAddMoveUp),
            keyboard::Key::Named(Named::ArrowDown) => Some(Message::QuickAddMoveDown),
            // Escape — close whichever modal is open (handled in update())
            keyboard::Key::Named(Named::Escape) => Some(Message::CloseActiveModal),
            _ => None,
        }
    } else {
        None
    }
});
```

And add a `CloseActiveModal` message and arm (see Phase 11 plan — it is introduced there and
handles Escape across all modals).

**Why always register this subscription instead of conditionally?**

Arrow keys and Escape should do nothing when no modal is open. Rather than conditionally
registering the subscription, the `update()` arms for `QuickAddMoveUp`, `QuickAddMoveDown`,
and `CloseActiveModal` are no-ops when the relevant flags are false. The cost of receiving
an ignored keyboard event is negligible compared to the complexity of computing a condition
that tracks all three modal states.

**Why intercept arrow keys even when `status == Captured`?**

When the text input is focused, iced marks arrow key events as `Captured` (for cursor
movement). If we filtered by `status == Ignored`, arrow keys would move the text cursor but
not the selection. We want both: the text cursor moves AND the list selection moves. Emitting
`QuickAddMoveUp`/`QuickAddMoveDown` unconditionally achieves this. Since those messages are
no-ops when `quick_add_open` is false, there is no unintended side effect elsewhere.

---

## Step 8 — Verify

```
cargo check
cargo run
```

Test in this order:

1. Open the app. No timer running. Press Q (Phase 11 adds this; for now, add a temporary
   "Quick Add" button to `status_bar.rs` that fires `Message::OpenQuickAdd` for testing).
2. The modal opens and the text input is focused.
3. Type part of a task name. The list filters in real time.
4. Arrow Down moves the `▶` cursor down. Arrow Up moves it back.
5. Press Enter — the timer starts for the selected task. The modal closes.
6. Reopen with no text. All tasks appear. Arrow keys navigate. Enter starts.
7. Type a name that matches nothing. A "Create" row appears. Arrow Down selects it. Enter
   creates the task and starts the timer. Open Review — the new task has an entry.
8. Reopen. Type something. Press Escape. Modal closes. Nothing changes.
9. Click a task row directly — timer starts immediately (mouse path).

---

## What this phase does not do

- **No keyboard shortcut to open quick-add** — that is Phase 11 (Q key)
- **No Eisenhower toggles in the create flow** — quick-add creates with defaults; use the
  task edit panel to set urgent/important after
- **No filtering by status** — Done tasks appear in the list; Phase 13 can add visual
  distinction or a toggle to hide them
- **No fuzzy matching** — substring match only; good enough for daily use

---

## Commit message (draft)

```
feat(ui): quick-add filter modal

Opens a modal that filters existing tasks or creates a new one.
Arrow keys navigate, Enter starts the timer. Mouse clicks on rows
start immediately. StartTimer closes the modal in all code paths.
```
