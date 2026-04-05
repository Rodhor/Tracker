# Feature Plan — Phase 6: Delete

## What this phase builds

- A **Delete** button on each completed entry row in the review view
- An inline confirmation replaces the row before the delete actually happens
- A **Delete task** button in the task edit panel
- Deleting a task cascades — all of its time entries are also removed
- If the active timer belongs to a task being deleted, the timer is discarded silently

No system dialogs. All confirmation is inline, using the same pattern already in use for
note editing and task editing: the row transforms in place, then restores on cancel.

---

## Why delete matters

The review is the TANNSS reporting surface. Mistakes happen — accidentally tracked time on
the wrong task, a timer that ran overnight, a test task that was never real. Without delete,
those errors are permanent and pollute the review every day.

---

## Interaction design: inline confirmation

iced has no native confirmation dialog. The correct pattern for this app is inline
confirmation — the same approach used for task editing and note editing:

1. User clicks **Delete** — the row transforms to a "confirm or cancel" state
2. User clicks **Confirm delete** — the item is removed and the view refreshes
3. User clicks **Cancel** — the row restores to its normal state

Only one confirmation can be open at a time. Opening a new delete confirmation clears any
other open edit state (note editing, task editing) and vice versa.

---

## Changes overview

| Area                 | Change                                                                     |
|----------------------|----------------------------------------------------------------------------|
| `App` struct         | Add `deleting_entry_id: Option<Uuid>`, `deleting_task_id: Option<Uuid>`    |
| `Message`            | Add 6 new variants for entry and task delete                               |
| `update()`           | Implement 6 new arms                                                       |
| `review_screen()`    | Entry rows get a Delete button; confirmation row replaces them when active |
| `task_row_or_edit()` | Route to `delete_task_confirm_row()` when `deleting_task_id` matches       |
| `edit_row()`         | Add Delete button to the first row                                         |
| New method           | `delete_task_confirm_row()`                                                |

---

## Step 1 — Add state fields to `App`

```rust
pub struct App {
    // ... existing fields ...

    // Delete confirmation state
    deleting_entry_id: Option<Uuid>,  // review: which entry has confirmation showing
    deleting_task_id: Option<Uuid>,   // tracker: which task has confirmation showing
}
```

Initialise both to `None` in `new()`.

---

## Step 2 — Add new `Message` variants

```rust
pub enum Message {
    // ... existing variants ...

    // Entry delete (review view)
    RequestDeleteEntry(Uuid),
    ConfirmDeleteEntry,
    CancelDeleteEntry,

    // Task delete (tracker view — edit panel)
    RequestDeleteTask(Uuid),
    ConfirmDeleteTask,
    CancelDeleteTask,
}
```

---

## Step 3 — Implement new `update()` arms

### Entry delete

```rust
Message::RequestDeleteEntry(entry_id) => {
    // Clear any open note edit first — only one action per row at a time.
    self.editing_note_id = None;
    self.editing_note_text.clear();
    self.deleting_entry_id = Some(entry_id);
}

Message::ConfirmDeleteEntry => {
    if let Some(id) = self.deleting_entry_id {
        self.entries.retain(|e| e.id != id);
        self.deleting_entry_id = None;
        self.save();
    }
}

Message::CancelDeleteEntry => {
    self.deleting_entry_id = None;
}
```

**Why `retain`?** `Vec::retain(|x| predicate)` removes all elements where the predicate
is false — a single-pass in-place filter. It is the idiomatic Rust way to delete by
predicate without allocating a new vec.

### Task delete

```rust
Message::RequestDeleteTask(task_id) => {
    // Close the edit panel — we're switching to delete confirmation for this task.
    self.editing_task_id = None;
    self.edit_description.clear();
    self.deleting_task_id = Some(task_id);
}

Message::ConfirmDeleteTask => {
    if let Some(id) = self.deleting_task_id {
        // If the active timer is for this task, discard it silently.
        if self.active_entry.as_ref().is_some_and(|e| e.task_id == id) {
            self.active_entry = None;
            // Also close the stop prompt if it was open.
            self.stop_prompt_open = false;
            self.stop_prompt_note.clear();
        }

        // Cascade: remove all entries that belonged to this task.
        self.entries.retain(|e| e.task_id != id);

        // Remove the task itself.
        self.tasks.retain(|t| t.id != id);

        self.deleting_task_id = None;
        self.save();
    }
}

Message::CancelDeleteTask => {
    self.deleting_task_id = None;
}
```

**Why discard the active timer silently?** The task no longer exists — there is nowhere to
attribute the time. A prompt would be confusing. Discarding is the only coherent outcome.

**Why cascade entries?** Entries without a task are meaningless for TANNSS reporting. The
task name is the attribution. Keeping orphaned entries would silently corrupt the review.

---

## Step 4 — Add Delete button to `edit_row()`

The first row of the edit panel currently has: task name | Save | Cancel

Add a Delete button at the end of that row:

```rust
fn edit_row<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
    column![
        row![
            text(&task.name).width(Length::Fill),
            button(text("Save")).on_press(Message::SaveEditTask),
            button(text("Cancel")).on_press(Message::CloseEditTask),
            button(text("Delete")).on_press(Message::RequestDeleteTask(task.id)),
        ]
        .spacing(8),
        // ... rest of edit_row unchanged ...
    ]
    // ...
}
```

---

## Step 5 — Add `delete_task_confirm_row()`

This replaces the task row when `deleting_task_id == Some(task.id)`.

```rust
fn delete_task_confirm_row<'a>(&self, task: &'a AppTask) -> Element<'a, Message> {
    // Count how many entries belong to this task so the warning is informative.
    let entry_count = self.entries.iter().filter(|e| e.task_id == task.id).count();
    let warning = if entry_count == 1 {
        format!("Delete '{}' and 1 time entry?", task.name)
    } else if entry_count > 1 {
        format!("Delete '{}' and {} time entries?", task.name, entry_count)
    } else {
        format!("Delete '{}'?", task.name)
    };

    row![
        text(warning).width(Length::Fill),
        button(text("Confirm delete")).on_press(Message::ConfirmDeleteTask),
        button(text("Cancel")).on_press(Message::CancelDeleteTask),
    ]
    .padding(8)
    .spacing(8)
    .into()
}
```

---

## Step 6 — Update `task_row_or_edit()`

Add a third branch for the delete confirmation state:

```rust
fn task_row_or_edit<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
    if self.deleting_task_id == Some(task.id) {
        self.delete_task_confirm_row(task)
    } else if self.editing_task_id == Some(task.id) {
        self.edit_row(task)
    } else {
        self.task_row(task)
    }
}
```

The delete confirmation takes priority over the edit panel. `RequestDeleteTask` already
clears `editing_task_id`, so in practice both can't be true at the same time — but ordering
the check this way makes the priority explicit and safe.

---

## Step 7 — Add Delete button to entry rows in `review_screen()`

Inside the `ReviewRow::Entry` match arm, update the note widget branch for the normal
(non-editing) state, and add a separate confirmation row when the entry is being deleted.

The current structure of the entry match arm is:

```rust
ReviewRow::Entry { entry, task_name } => {
    // ... build start_str, end_str, duration_label, pause_label ...

    let note_widget = if self.editing_note_id == Some(entry.id) {
        // ... edit field ...
    } else {
        // ... note display + Edit button ...
    };

    let entry_row = row![ ... note_widget ... ].into();
    items.push(entry_row);
}
```

Replace this block with a check that intercepts if the delete confirmation is active:

```rust
ReviewRow::Entry { entry, task_name } => {
    // ... build start_str, end_str, net_min, duration_label, pause_label ...
    // ... update total_tracked, first_start, last_end as before ...

    // If this entry has a pending delete confirmation, show that instead.
    if self.deleting_entry_id == Some(entry.id) {
        let confirm_row = row![
            text(format!("Delete '{task_name}' ({start_str} → {end_str})?"))
                .width(Length::Fill),
            button(text("Confirm delete")).on_press(Message::ConfirmDeleteEntry),
            button(text("Cancel")).on_press(Message::CancelDeleteEntry),
        ]
        .padding(8)
        .spacing(8);
        items.push(confirm_row.into());
        continue;  // skip the normal entry row build
    }

    // Normal row — note widget and Delete button.
    let note_widget: Element<Message> = if self.editing_note_id == Some(entry.id) {
        // ... edit field unchanged ...
    } else {
        let note_text = entry.notes.clone().unwrap_or_else(|| "-".to_string());
        row![
            text(note_text).width(Length::Fill),
            button(text("Edit")).on_press(Message::OpenEditNote(entry.id)),
            // Only show Delete for completed entries — not the active one.
            button(text("Delete"))
                .on_press_maybe(
                    entry.ended_at.as_ref().map(|_| Message::RequestDeleteEntry(entry.id))
                ),
        ]
        .spacing(4)
        .into()
    };

    // ... rest of entry_row build unchanged ...
}
```

**Why `on_press_maybe` for Delete?** The active entry (no `ended_at`) should not be
deletable from the review — use the Stop flow for that. `on_press_maybe` renders the button
visually but makes it non-interactive when the entry is still running.

**Why `continue`?** The loop is a `for` loop over owned `rows`. `continue` skips the rest
of the loop body for this iteration — cleanly avoids deeply nested if/else for the two
mutually exclusive display states.

---

## Step 8 — Clear conflicting state when opening edits

Two small additions to existing arms to keep the UI consistent:

In `Message::OpenEditNote(entry_id)`: clear `deleting_entry_id` at the start.

```rust
Message::OpenEditNote(entry_id) => {
    self.deleting_entry_id = None;  // ← add this line
    let current_note = ...
}
```

In `Message::OpenEditTask(task_id)`: clear `deleting_task_id` at the start.

```rust
Message::OpenEditTask(task_id) => {
    self.deleting_task_id = None;  // ← add this line
    if let Some(task) = ...
}
```

This ensures that if a delete confirmation is open and the user somehow triggers a different
edit, the confirmation closes cleanly rather than both states being true simultaneously.

---

## Step 9 — Verify

```
cargo run
```

**Entry delete (review view):**

1. Add tasks, run timers, stop them, go to Review
2. Click **Delete** on a completed entry — row transforms to confirmation
3. Click **Cancel** — row restores normally
4. Click **Delete** again, then **Confirm delete** — entry disappears, summary updates
5. Restart — deleted entry does not reappear
6. The active entry (if any) shows a greyed-out / non-interactive Delete button

**Task delete (tracker view):**

1. Add a task with several completed time entries
2. Click **Edit** on the task → **Delete** in the edit panel
3. The row shows: "Delete 'task name' and X time entries?" with Confirm / Cancel
4. Click **Cancel** — returns to the edit panel
5. Click **Delete** → **Confirm delete** — task and all its entries disappear
6. Go to Review — the deleted entries are gone
7. Restart — nothing reappears

**Active timer + task delete:**

1. Start a timer on task A
2. Edit task A → Delete → Confirm
3. Task disappears, timer bar shows "No timer running"
4. The stop prompt (if it was open) is also closed

---

## What this phase does not do

- **No edit of entry times** — if a timer ran too long, you can delete the entry and
  re-add it manually by starting/stopping again. Inline time editing is a later concern.
- **No undo** — delete is permanent. A confirmation step is the only safeguard.
- **No bulk delete** — entries are deleted one at a time.

---

## Commit message (draft)

```
feat(app): phase 6 — inline delete for entries and tasks

Time entries can be deleted from the review view via an inline
confirmation row. Tasks can be deleted from the edit panel; deletion
cascades to all associated time entries and discards any active timer
for that task. Active entries are protected from review deletion.
```
