# Feature: Task Rename

- **Mode:** FEATURE
- **Date:** 2026-04-08

---

## What it does

Allows the user to rename a task from within the existing edit panel. Currently the task name is shown as static text in `edit_row` — this adds a `text_input` in its place so the name can be changed and saved.

---

## Where it fits

Follows the same pattern as all other editable task fields (`edit_urgent`, `edit_important`, `edit_description_content`): a temporary state field is populated when the edit panel opens, modified by the user, then written back to the task on save.

No new files. No new screens. The change is contained to the edit panel path.

---

## New state

In `src/app.rs` — add one field to the `App` struct:

```rust
pub(crate) edit_task_name: String,
```

---

## New message

In `src/message.rs` — add to the "Edit task panel" group:

```rust
EditTaskNameChanged(String),
```

---

## Changes to `src/app.rs`

**`new()` initialiser** — initialise alongside the other edit fields:
```rust
edit_task_name: String::new(),
```

**`OpenEditTask` arm** — populate when the panel opens:
```rust
self.edit_task_name = task.name.clone();
```

**`CloseEditTask` arm** — clear on close:
```rust
self.edit_task_name.clear();
```

**New `EditTaskNameChanged` arm** — simple update:
```rust
Message::EditTaskNameChanged(value) => {
    self.edit_task_name = value;
}
```

**`SaveEditTask` arm** — write the name back, guard against empty:
```rust
let name = self.edit_task_name.trim().to_string();
if !name.is_empty() {
    task.name = name;
}
// ... existing urgent, important, description saves unchanged
```
Also clear the field after the save block:
```rust
self.edit_task_name.clear();
```

**`CancelDeleteTask` arm** — this re-opens the edit panel for the task that was about to be deleted, so it must also restore the name:
```rust
self.edit_task_name = task.name.clone();
```

---

## Changes to `src/ui/task_list.rs`

In `edit_row`, replace the static name display with an editable input.

```rust
// Remove:
text(&task.name).width(Length::Fill),

// Add:
text_input("Task name", &app.edit_task_name)
    .on_input(Message::EditTaskNameChanged)
    .on_submit(Message::SaveEditTask)
    .width(Length::Fill),
```

`text_input` is already imported in this file — no import changes needed.

---

## Implementation order

1. Add `edit_task_name: String` to the `App` struct and initialise in `new()`
2. Add `EditTaskNameChanged(String)` to `Message`
3. Update `OpenEditTask` to populate `edit_task_name`
4. Update `CloseEditTask` to clear it
5. Update `CancelDeleteTask` to restore it
6. Add the `EditTaskNameChanged` arm in `update()`
7. Update `SaveEditTask` to write the name back and clear the field
8. Replace `text(&task.name)` with `text_input` in `edit_row`

---

## Testing

- Open edit panel → name field should be pre-filled with the current name
- Change the name and save → task list updates immediately
- Save with an empty name → name should remain unchanged (guard prevents overwrite)
- Cancel without saving → original name is preserved
- Delete confirm → cancel returns to edit panel with name still pre-filled
