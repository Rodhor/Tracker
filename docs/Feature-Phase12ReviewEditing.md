# Feature Plan — Phase 12: Entry Time Editing + Copy Button

**Date:** 2026-04-07
**Permanent record:** `docs/features/Phase12ReviewEditing.md`

---

## What this phase builds

Two additions to the review screen:

1. **Copy button** — puts an entry's note text on the system clipboard. One click, then switch to TANNSS and paste.
2. **Time editing** — each completed entry gets an "Edit time" mode showing two HH:MM inputs for start and end. Saving recomputes the stored duration. Fixes entries where you forgot to stop the timer.

---

## What already exists

- `src/ui/review.rs` — the full review view; entry rows already have Edit note / Delete buttons
- `TimeEntry` fields: `started_at` (UTC RFC3339), `ended_at` (UTC RFC3339), `minutes` (net duration), `total_paused`
- `editing_note_id: Option<Uuid>` pattern — the existing note edit state; time editing follows the same pattern
- `format_hhmm()` in `review.rs` — parses RFC3339 and formats as HH:MM in local time; we need the reverse for saving

**No new crate needed.** iced 0.14 has a built-in clipboard Task: `iced::clipboard::write(text)`. Adding `arboard` would be redundant.

---

## Changes overview

| File               | Change                                                                          |
|--------------------|---------------------------------------------------------------------------------|
| `src/message.rs`   | Add 6 new messages                                                              |
| `src/app.rs`       | Add 3 state fields; implement 6 new `update()` arms                            |
| `src/ui/review.rs` | Add Copy button; add Edit time button; add time-editing row state; add a helper |

---

## Step 1 — Add new messages (`src/message.rs`)

```rust
// Copy button
CopyEntryNote(String),     // carries the note text directly — no Uuid lookup needed at call site

// Time editing
OpenEditTime(Uuid),
EditTimeStartChanged(String),
EditTimeEndChanged(String),
SaveEditTime,
CancelEditTime,
```

**Why `CopyEntryNote(String)` carries text instead of a `Uuid`?**

The note text is already in scope when the button is rendered — we are iterating over entries in
the view. Passing the text directly avoids a redundant `Uuid` → entry → notes lookup in `update()`.
The clipboard write is a fire-and-forget operation; no state needs to be updated after it.

**Why `EditTimeStartChanged(String)` instead of an Action?**

HH:MM is a short, single-line field. `text_input` (not `text_editor`) is the right widget —
the same widget used for `new_task_input` and `quick_add_input`. Single-line inputs carry
`String` in their messages.

---

## Step 2 — Add state fields to `App` (`src/app.rs`)

```rust
// Time editing (review screen)
pub(crate) editing_time_id: Option<Uuid>,
pub(crate) edit_time_start: String,   // "HH:MM" string while editing
pub(crate) edit_time_end: String,     // "HH:MM" string while editing
```

Initialise in `App::new()`:

```rust
editing_time_id: None,
edit_time_start: String::new(),
edit_time_end: String::new(),
```

---

## Step 3 — Implement `update()` arms (`src/app.rs`)

### CopyEntryNote

```rust
Message::CopyEntryNote(text) => {
    return iced::clipboard::write(text);
}
```

`iced::clipboard::write` returns a `Task<T>` that hands the text to the OS clipboard. No
state changes. No save needed. The return type matches `Task<Message>` because iced infers
`T = Message` from the surrounding context.

### OpenEditTime

```rust
Message::OpenEditTime(entry_id) => {
    // Close any open note edit to avoid two editors at once
    self.editing_note_id = None;
    self.editing_note_content = text_editor::Content::new();

    if let Some(entry) = self.entries.iter().find(|e| e.id == entry_id) {
        use chrono::{DateTime, Local};
        let start = DateTime::parse_from_rfc3339(&entry.started_at)
            .map(|dt| dt.with_timezone(&Local).format("%H:%M").to_string())
            .unwrap_or_default();
        let end = entry.ended_at.as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Local).format("%H:%M").to_string())
            .unwrap_or_default();

        self.editing_time_id = Some(entry_id);
        self.edit_time_start = start;
        self.edit_time_end = end;
    }
}
```

**Why pre-fill with the current times?**

The user is correcting a mistake, not building from scratch. Pre-filling shows what is
stored and makes small adjustments (e.g. changing 17:45 to 17:30) fast.

### EditTimeStartChanged / EditTimeEndChanged

```rust
Message::EditTimeStartChanged(value) => {
    self.edit_time_start = value;
}
Message::EditTimeEndChanged(value) => {
    self.edit_time_end = value;
}
```

### SaveEditTime

```rust
Message::SaveEditTime => {
    use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};

    let Some(id) = self.editing_time_id else {
        return Task::none();
    };
    let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) else {
        return Task::none();
    };

    // Parse the entry's date in local time (times are displayed in local)
    let entry_date = DateTime::parse_from_rfc3339(&entry.started_at)
        .map(|dt| dt.with_timezone(&Local).date_naive());
    let Ok(entry_date) = entry_date else {
        return Task::none(); // malformed stored timestamp — give up
    };

    // Parse both HH:MM inputs
    let Ok(new_start_time) = NaiveTime::parse_from_str(&self.edit_time_start, "%H:%M") else {
        return Task::none(); // invalid format — give up silently for now
    };
    let Ok(new_end_time) = NaiveTime::parse_from_str(&self.edit_time_end, "%H:%M") else {
        return Task::none();
    };

    // Reconstruct local datetimes, then convert to UTC RFC3339 for storage
    let Some(start_local) = Local.from_local_datetime(&entry_date.and_time(new_start_time)).single() else {
        return Task::none(); // ambiguous local time (DST) — give up
    };
    let Some(end_local) = Local.from_local_datetime(&entry_date.and_time(new_end_time)).single() else {
        return Task::none();
    };

    // Validate: end must be after start
    if end_local <= start_local {
        return Task::none(); // invalid range — give up silently for now
    }

    // Recompute net minutes
    let gross_minutes = (end_local - start_local).num_minutes();
    let net_minutes = (gross_minutes - entry.total_paused).max(0);

    entry.started_at = start_local.with_timezone(&Utc).to_rfc3339();
    entry.ended_at = Some(end_local.with_timezone(&Utc).to_rfc3339());
    entry.minutes = Some(net_minutes);

    self.editing_time_id = None;
    self.edit_time_start = String::new();
    self.edit_time_end = String::new();
    self.save();
}
```

**Why give up silently instead of showing an error?**

Error display requires new state (an error string field) and new UI. Phase 13 is the right
time to add visual feedback. For now, silent failure keeps the scope contained — the inputs
retain their values so the user can correct them.

**Why `NaiveTime::parse_from_str` and not just split on `:`?**

`NaiveTime::parse_from_str` validates the range (hours 0–23, minutes 0–59) and handles edge
cases like leading zeros. Manual splitting would require the same checks by hand.

**Why `Local.from_local_datetime(...).single()`?**

DST transitions create ambiguous local times (e.g. 02:30 can occur twice). `.single()` returns
`None` for ambiguous times rather than guessing. This is a correctness safeguard that will
almost never trigger in practice.

**Why recompute `minutes` instead of leaving it as-is?**

`minutes` is the stored net duration. It is what the review uses for display and what TANNSS
gets. If the times change, the duration must follow. Leaving the old value would show a
contradiction: the times say 1h but the duration says 3h.

### CancelEditTime

```rust
Message::CancelEditTime => {
    self.editing_time_id = None;
    self.edit_time_start = String::new();
    self.edit_time_end = String::new();
}
```

---

## Step 4 — Update `OpenEditNote` to close any time edit (`src/app.rs`)

In the existing `OpenEditNote` arm, add at the top:

```rust
Message::OpenEditNote(entry_id) => {
    // Close any open time edit to avoid two editors at once
    self.editing_time_id = None;
    self.edit_time_start = String::new();
    self.edit_time_end = String::new();
    // ... rest of existing arm unchanged
```

---

## Step 5 — Update the review view (`src/ui/review.rs`)

The entry row currently has two states: normal and note-editing. After this phase it has three:
normal, note-editing (unchanged), and time-editing.

### Add a helper to parse HH:MM for display in inputs

Add this at the bottom of `review.rs`, next to `format_hhmm`:

```rust
// Parses "HH:MM" — used to validate what the user typed before rendering.
// Returns the input unchanged; here purely to document the expected format.
// Actual validation happens in SaveEditTime in update().
```

No code needed — just use `text_input` with the string as-is. The helper note is for clarity.

### Modify the entry row build loop

Currently the note widget is built as:

```rust
let note_widget: Element<Message> = if app.editing_note_id == Some(entry.id) {
    // note editor
} else {
    // normal note + Edit/Delete buttons
};
```

Extend this with a third branch for time editing. Restructure as follows:

```rust
// --- Time editing row (replaces entire entry_row when active) ---
if app.editing_time_id == Some(entry.id) {
    let time_edit = row![
        text(task_name).width(Length::FillPortion(2)),
        text_input("HH:MM", &app.edit_time_start)
            .on_input(Message::EditTimeStartChanged)
            .width(Length::Fixed(60.0)),
        text(" -> "),
        text_input("HH:MM", &app.edit_time_end)
            .on_input(Message::EditTimeEndChanged)
            .width(Length::Fixed(60.0)),
        button(text("Save")).on_press(Message::SaveEditTime),
        button(text("Cancel")).on_press(Message::CancelEditTime),
    ]
    .padding(8)
    .spacing(8);
    items.push(time_edit.into());
    continue; // skip the normal row build
}
```

Place this block right after the delete-confirm block (which also uses `continue`), before the
`note_widget` build.

**Why `continue` instead of an if-else chain?**

The delete confirm row also uses `continue` for the same reason: when a special row state is
active, the entire normal row layout is replaced. Using `continue` keeps the three special
cases (delete confirm, time edit) visually symmetrical — each one short-circuits early with
a `continue` and the reader never has to trace deeply nested if-else to understand what renders.

### Add Copy and Edit time buttons to the normal note row

Currently the normal note row is:

```rust
row![
    text(note_text).width(Length::Fill),
    button(text("Edit")).on_press(Message::OpenEditNote(entry.id)),
    button(text("Delete")).on_press_maybe(...),
]
```

Add two buttons:

```rust
let note_text = entry.notes.clone().unwrap_or_else(|| "-".to_string());
let copy_btn = entry.notes.as_ref().map(|note| {
    button(text("Copy")).on_press(Message::CopyEntryNote(note.clone()))
});

row![
    text(note_text).width(Length::Fill),
    button(text("Edit")).on_press(Message::OpenEditNote(entry.id)),
    // Copy only shown when a note exists
    if let Some(btn) = copy_btn { btn } else { button(text("Copy")) },
    // Edit time only for completed entries
    button(text("Edit time")).on_press_maybe(
        entry.ended_at.as_ref().map(|_| Message::OpenEditTime(entry.id))
    ),
    button(text("Delete")).on_press_maybe(
        entry.ended_at.as_ref().map(|_| Message::RequestDeleteEntry(entry.id))
    ),
]
.spacing(4)
.into()
```

**Why disable Copy when there is no note (show greyed button) rather than hiding it?**

Hiding a button causes the row layout to shift — neighbouring widgets reflow to fill the space.
That is visually distracting when scanning multiple rows. A disabled (no `on_press`) button
holds its space and communicates "this action is unavailable here" without moving things around.

**Why disable Edit time for the active entry?**

The active entry has no `ended_at`. Its end time is "now", which changes every second. Editing
start time while the timer runs could produce a negative duration. The `on_press_maybe` guard
on `ended_at.as_ref()` handles this automatically — same pattern already used for Delete.

---

## Step 6 — Verify

```
cargo check
cargo run
```

Test in this order:

1. **Copy:** Run a task for a few minutes, stop it with a note. Open review. Press Copy on
   that entry. Switch to any text editor and paste — the note text should appear.
2. **Copy disabled:** An entry with no note shows a greyed Copy button. Clicking does nothing.
3. **Edit time — normal:** Open review on a completed entry. Press Edit time. The inputs are
   pre-filled with the entry's current start and end times. Change end time forward by 15
   minutes. Save. The row refreshes with the new end time and updated duration.
4. **Edit time — validation:** Change end time to before start time. Press Save. Nothing
   happens, inputs retain their values.
5. **Edit time — invalid format:** Type "9:5" in an input. Press Save. Nothing happens.
6. **Mutual exclusion:** With a note editor open, press Edit time on the same entry. The note
   editor closes and the time editor opens. And vice versa.
7. **Active entry:** The currently running entry shows Edit time greyed out.
8. **Cancel:** Open Edit time, change values, press Cancel. The row returns to normal with
   the original times unchanged.

---

## What this phase does not do

- **No error messages** — invalid input silently does nothing; Phase 13 can add inline
  validation feedback
- **No cross-midnight entries** — if start is 23:50 and end is 00:10 the next day, the
  reconstruction would produce end < start and reject it. An edge case not worth the
  complexity for a personal tool.
- **No editing of active entries** — start time editing while the timer runs is excluded

---

## Commit message (draft)

```
feat(review): entry time editing and copy button

Copy puts the entry note on the clipboard via iced's built-in
clipboard Task. Edit time shows HH:MM inputs pre-filled from
the stored timestamps, validates end > start, and recomputes
net minutes on save. Both controls are disabled for active entries.
```
