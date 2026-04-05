# Feature — Phase 3: Eisenhower Sorting and Task Editing

**Date:** 2026-04-04
**Mode:** FEATURE
**Architecture reference:** `docs/Brainstorm-RustIcedRewrite.md`
**Permanent record:** `docs/features/Phase3Eisenhower.md`

---

## What this phase builds

- The task list splits into two sections: **sorted tasks** (Q1 → Q4) and **Needs sorting**
- A task with neither flag set (`urgent=false, important=false`) stays in "Needs sorting"
  until the user explicitly categorises it
- Each task row gains an Edit button
- Clicking Edit expands an inline edit panel below that row showing urgent/important
  checkboxes and a description field
- Saving writes the changes; cancelling discards them

The data model already has `urgent`, `important`, and `description` from Phase 1.
No new fields, no new files — this phase is entirely in `app.rs`.

---

## What already exists

- `Task::quadrant()` — returns 1–4 based on urgent/important flags
- `Task::has_priority()` — returns `true` if either flag is set; used to split sections
- `urgent: bool`, `important: bool`, `description: Option<String>` on every task
- All of `app.rs` from Phase 2

---

## Changes overview

| Area | Change |
|------|--------|
| `App` struct | Add 4 edit-state fields |
| `Message` enum | Add 6 edit messages |
| `update()` | Add 6 new arms |
| `task_list()` | Sort + split into two sections, route to edit row when editing |
| `task_row()` | Add Edit button |
| `edit_row()` | New method — the inline edit panel |
| imports | Add `checkbox` |

---

## Step 1 — Add edit state to App

The edit panel is inline — it replaces the task row for the task being edited. The App
needs to know which task is open for editing and hold the temporary values the user is
typing, so changes can be discarded on Cancel without touching the real task.

```rust
pub struct App {
    tasks: Vec<AppTask>,
    entries: Vec<TimeEntry>,
    active_entry: Option<TimeEntry>,
    new_task_input: String,

    // Edit panel state — all None/empty when no task is being edited
    editing_task_id: Option<Uuid>,   // which task has the edit panel open
    edit_urgent: bool,               // temporary value while editing
    edit_important: bool,            // temporary value while editing
    edit_description: String,        // temporary value while editing
}
```

Initialise them in `App::new()`:

```rust
editing_task_id: None,
edit_urgent: false,
edit_important: false,
edit_description: String::new(),
```

**Why temporary fields instead of editing the task directly?**

If you mutated the task in place while the user typed, clicking Cancel would have no
effect — the changes are already in the model. By copying the current values into
`edit_*` fields when the panel opens, you can discard them on Cancel by just clearing
these fields and leaving the task untouched.

---

## Step 2 — Add new Message variants

```rust
pub enum Message {
    // ... existing variants ...

    // Edit panel
    OpenEditTask(Uuid),           // Edit button clicked — open panel for this task
    CloseEditTask,                // Cancel clicked — discard and close
    EditUrgentChanged(bool),      // urgent checkbox toggled
    EditImportantChanged(bool),   // important checkbox toggled
    EditDescriptionChanged(String), // description text changed
    SaveEditTask,                 // Save clicked — write changes and close
}
```

Add `checkbox` to the iced widget imports:

```rust
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
```

---

## Step 3 — Implement update() arms

Add these six arms to the match in `update()`:

```rust
Message::OpenEditTask(task_id) => {
    // Copy the task's current values into the temporary edit fields.
    // If the task is not found (should not happen), do nothing.
    if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
        self.editing_task_id = Some(task_id);
        self.edit_urgent = task.urgent;
        self.edit_important = task.important;
        // Option<String> → String: use an empty string if description is None
        self.edit_description = task.description.clone().unwrap_or_default();
    }
}

Message::CloseEditTask => {
    // Discard — just clear the edit state without touching self.tasks
    self.editing_task_id = None;
    self.edit_urgent = false;
    self.edit_important = false;
    self.edit_description.clear();
}

Message::EditUrgentChanged(value) => {
    self.edit_urgent = value;
}

Message::EditImportantChanged(value) => {
    self.edit_important = value;
}

Message::EditDescriptionChanged(value) => {
    self.edit_description = value;
}

Message::SaveEditTask => {
    // Apply the temporary values to the real task, then clear edit state.
    if let Some(id) = self.editing_task_id {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.urgent = self.edit_urgent;
            task.important = self.edit_important;
            // Store None if description is blank, Some(...) if non-empty.
            let desc = self.edit_description.trim().to_string();
            task.description = if desc.is_empty() { None } else { Some(desc) };
        }
        self.editing_task_id = None;
        self.edit_urgent = false;
        self.edit_important = false;
        self.edit_description.clear();
        self.save();
    }
}
```

---

## Step 4 — Add edit_row()

This renders the inline edit panel. It replaces the normal task row when a task is being
edited. Like `task_row`, it borrows from both `&self` and `task`, so it needs the `'a`
lifetime annotation.

```rust
fn edit_row<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
    column![
        // First line: task name (read-only) + Save + Cancel buttons
        row![
            text(&task.name).width(Length::Fill),
            button(text("Save")).on_press(Message::SaveEditTask),
            button(text("Cancel")).on_press(Message::CloseEditTask),
        ]
        .spacing(8),

        // Second line: the two priority checkboxes
        // checkbox(label, is_checked) — on_toggle sends the new bool value
        row![
            checkbox("Urgent", self.edit_urgent)
                .on_toggle(Message::EditUrgentChanged),
            checkbox("Important", self.edit_important)
                .on_toggle(Message::EditImportantChanged),
        ]
        .spacing(16),

        // Third line: the description input
        text_input("Description (optional)", &self.edit_description)
            .on_input(Message::EditDescriptionChanged)
            .on_submit(Message::SaveEditTask),
    ]
    .padding(8)
    .spacing(4)
    .into()
}
```

**`checkbox(label, value).on_toggle(Message::EditUrgentChanged)`**

`on_toggle` takes a function `fn(bool) -> Message`. `Message::EditUrgentChanged` is a
tuple variant that takes a `bool`, so it satisfies that signature exactly — no closure needed.
The same applies to `EditImportantChanged`.

---

## Step 5 — Update task_row()

Add an Edit button on the right:

```rust
fn task_row<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
    row![
        button(text("▶")).on_press(Message::StartTimer(task.id)),
        text(&task.name).width(Length::Fill),
        button(text(task.status.label())).on_press(Message::CycleStatus(task.id)),
        button(text("Edit")).on_press(Message::OpenEditTask(task.id)),
    ]
    .padding(8)
    .spacing(8)
    .into()
}
```

---

## Step 6 — Update task_list()

This is the most significant change in this phase. The list now:
1. Splits tasks into "sorted" (has_priority) and "unsorted" (!has_priority)
2. Sorts the priority tasks by quadrant (Q1 first, Q4 last)
3. Renders each task as either its normal row or the edit panel

```rust
fn task_list(&self) -> Element<'_, Message> {
    if self.tasks.is_empty() {
        return container(text("No tasks yet. Add one below."))
            .width(Length::Fill)
            .padding(20)
            .into();
    }

    let mut items: Vec<Element<Message>> = Vec::new();

    // --- Sorted section ---
    // Collect tasks that have been prioritised, then sort by quadrant.
    // sort_by_key() is a stable sort — tasks within the same quadrant keep
    // their original insertion order.
    let mut sorted: Vec<&AppTask> = self.tasks.iter()
        .filter(|t| t.has_priority())
        .collect();
    sorted.sort_by_key(|t| t.quadrant());

    for task in sorted {
        items.push(self.task_row_or_edit(task));
    }

    // --- Needs sorting section ---
    let unsorted: Vec<&AppTask> = self.tasks.iter()
        .filter(|t| !t.has_priority())
        .collect();

    if !unsorted.is_empty() {
        // Section header — only shown when there are unsorted tasks
        // Padding::new(8) sets all sides to 8, then .bottom(4) reduces bottom spacing
        // so the header sits closer to its tasks than to the section above it.
        // Note: .padding([top, right, bottom, left]) does NOT exist in iced 0.14.
        // Use Padding::new(n).side(value) builder syntax instead.
        items.push(
            container(text("Needs sorting"))
                .padding(iced::Padding::new(8).bottom(4))
                .into()
        );
        for task in unsorted {
            items.push(self.task_row_or_edit(task));
        }
    }

    scrollable(column(items).spacing(4))
        .height(Length::Fill)
        .into()
}

// Routes to edit_row or task_row depending on whether this task is open for editing.
// Extracted so task_list() stays readable.
fn task_row_or_edit<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
    if self.editing_task_id == Some(task.id) {
        self.edit_row(task)
    } else {
        self.task_row(task)
    }
}
```

**Why `task_row_or_edit` as a separate method?**

The `task_list()` loop calls this for every task. If the routing logic were inline, the loop
body would grow large and obscure the sorting intent. A named helper makes the loop read as:
"for each task in this group, render it (in whatever state it is in)."

---

## Step 7 — Verify

```
cargo run
```

Test in this order:
1. Add several tasks — they all appear in "Needs sorting"
2. Click Edit on a task — the edit panel expands in place
3. Check Urgent — click Save — the task moves to the sorted section (Q3: urgent, not important)
4. Check both Urgent and Important — Save — task moves to Q1
5. Add a description — confirm it saves (open Edit again to check)
6. Click Cancel — confirm no changes applied
7. Add tasks with different priority combinations — confirm Q1 → Q2 → Q3 → Q4 order
8. Restart the app — sorting and priorities are preserved

---

## What this phase does not do

- **No visual quadrant labels** — tasks are sorted but not labelled "Do First", "Schedule",
  etc. Add headers per quadrant in a later styling pass if desired.
- **No priority on new tasks** — adding a task from the status bar always creates it with
  both flags false (into "Needs sorting"). The Quick-add panel in Phase 4 adds priority
  toggles to the creation flow.
- **No description display** — the description is stored and editable but not shown on the
  task row. Add it as a subtitle in a later styling pass.

---

## Commit message (draft)

```
feat(ui): Eisenhower sorting and inline task editing

Tasks are now split into sorted (Q1–Q4) and "Needs sorting" sections.
Editing urgent/important/description is done via an inline edit panel
that opens in place of the task row. Changes are staged in temporary
fields and only applied on Save.
```
