# Feature Plan — Phase 5: Review View

## What this phase builds

- Clicking **Review** switches the main window to a dedicated review view
- The review view shows all completed time entries for a chosen date in chronological order
- Gaps between entries are shown as "Untracked" rows
- Each entry row shows: task name, start → end times, net minutes, pause minutes (if any), and the note
- Notes can be edited inline (same copy-on-open/apply-on-save pattern as task editing)
- A summary bar shows: first start, last end, total tracked, total untracked
- Prev/Next buttons navigate between dates
- A **Close** button returns to the main tracker view

This is the reporting surface — the feature that makes the captured data usable for TANNSS.

---

## Architecture fit

iced is a single-window framework. There is no second window. The standard approach in Elm-style
architectures is a `Screen` enum on the App struct. `view()` checks the screen and delegates to
the appropriate render function.

This keeps all state in one place and avoids any cross-window communication complexity.

---

## Data already available

All data needed exists:

| Field                     | Where                              |
|---------------------------|------------------------------------|
| `started_at`, `ended_at`  | `TimeEntry`                        |
| `minutes` (net work time) | `TimeEntry`                        |
| `total_paused`            | `TimeEntry`                        |
| `notes`                   | `TimeEntry`                        |
| Task name                 | `App.tasks` — matched by `task_id` |

No new fields are needed on `TimeEntry` or `Task`.

---

## Changes overview

| Area                    | Change                                                                                                                                               |
|-------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| `app.rs` — `App` struct | Add `screen: Screen`, `review_date`, `editing_note_id`, `editing_note_text`                                                                          |
| `app.rs` — `Message`    | `OpenReview` arm filled in; add `CloseReview`, `ReviewPrevDay`, `ReviewNextDay`, `OpenEditNote`, `EditNoteChanged`, `SaveEditNote`, `CancelEditNote` |
| `app.rs` — `update()`   | Implement all new arms                                                                                                                               |
| `app.rs` — `view()`     | Delegate to `review_view()` when `screen == Screen::Review`                                                                                          |
| `app.rs` — new method   | `review_view()` — full review layout                                                                                                                 |
| `app.rs` — new helper   | `build_review_rows()` — computes interleaved entry + gap rows                                                                                        |
| `app.rs` — new helper   | `format_hhmm(rfc3339: &str) -> String` — formats timestamps for display                                                                              |

---

## Step 1 — Add `Screen` enum

Add this above or below the `Message` enum:

```rust
// Which top-level view is active.
// Tracker is the default; Review shows the daily log.
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Tracker,
    Review,
}
```

---

## Step 2 — Add review state fields to `App`

```rust
pub struct App {
    // ... existing fields ...

    // Which top-level screen is shown
    screen: Screen,

    // Review view state
    review_date: chrono::NaiveDate,       // which day is being displayed
    editing_note_id: Option<Uuid>,        // which entry's note is open for editing
    editing_note_text: String,            // the text in the active note edit field
}
```

Add the corresponding initialisations in `new()`:

```rust
screen: Screen::Tracker,
review_date: chrono::Utc::now().date_naive(),
editing_note_id: None,
editing_note_text: String::new(),
```

**Why `NaiveDate`?**

`chrono::NaiveDate` is a date with no timezone — it's the right type for "which calendar day
am I looking at?" We don't need a time-of-day or timezone for that concept. All entries are
stored in UTC RFC 3339, so when we filter by date we compare against the date portion of
`started_at` as a string (`entry.started_at.starts_with(&date_str)`).

---

## Step 3 — Add new `Message` variants

```rust
pub enum Message {
    // ... existing variants ...

    // Review navigation
    CloseReview,
    ReviewPrevDay,
    ReviewNextDay,

    // Inline note editing (review view)
    OpenEditNote(Uuid),
    EditNoteChanged(String),
    SaveEditNote,
    CancelEditNote,
}
```

---

## Step 4 — Implement new `update()` arms

### Screen navigation

```rust
Message::OpenReview => {
// Always open to today. Reset any in-progress note edit.
self.screen = Screen::Review;
self.review_date = chrono::Utc::now().date_naive();
self.editing_note_id = None;
self.editing_note_text.clear();
}

Message::CloseReview => {
// Discard any unsaved note edit and return to tracker.
self.screen = Screen::Tracker;
self.editing_note_id = None;
self.editing_note_text.clear();
}

Message::ReviewPrevDay => {
self.review_date = self.review_date.pred_opt().unwrap_or( self.review_date);
// Navigating days discards any open note edit — same reason as CloseReview.
self.editing_note_id = None;
self.editing_note_text.clear();
}

Message::ReviewNextDay => {
let today = chrono::Utc::now().date_naive();
if self.review_date < today {
self.review_date = self.review_date.succ_opt().unwrap_or( self.review_date);
self.editing_note_id = None;
self.editing_note_text.clear();
}
// If already on today, do nothing — no future dates.
}
```

**Why discard edits on navigation?** The note belongs to an entry on a specific date. If
the user navigates away before saving, silently discarding (not saving) is correct — the
same behaviour as `CloseEditTask` in the tracker view.

**Why `pred_opt()` / `succ_opt()`?** These are chrono's safe date arithmetic methods.
`pred_opt()` returns `None` on underflow (before year 0); `succ_opt()` returns `None` on
overflow. Using `unwrap_or(self.review_date)` means the date simply doesn't change at the
extremes — no crash.

### Inline note editing

```rust
Message::OpenEditNote(entry_id) => {
// Copy the current note into the edit field (copy-on-open pattern).
let current_note = self.entries.iter()
.find( | e | e.id == entry_id)
.and_then( | e | e.notes.clone())
.unwrap_or_default();

self.editing_note_id = Some(entry_id);
self.editing_note_text = current_note;
}

Message::EditNoteChanged(value) => {
self.editing_note_text = value;
}

Message::SaveEditNote => {
if let Some(id) = self.editing_note_id {
let note = {
let s = self.editing_note_text.trim().to_string();
if s.is_empty() { None } else { Some(s) }
};
if let Some(entry) = self.entries.iter_mut().find( | e| e.id == id) {
entry.notes = note;
}
self.editing_note_id = None;
self.editing_note_text.clear();
self.save();
}
}

Message::CancelEditNote => {
// Discard — do not write anything.
self.editing_note_id = None;
self.editing_note_text.clear();
}
```

---

## Step 5 — Add `format_hhmm()` helper

```rust
fn format_hhmm(rfc3339: &str) -> String {
    use chrono::{DateTime, Utc};
    DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.with_timezone(&Utc).format("%H:%M").to_string())
        .unwrap_or_else(|_| "??:??".to_string())
}
```

This is used in entry rows to display `10:00 → 11:35`.

---

## Step 6 — Add `build_review_rows()` helper

```rust
// A single displayable row in the review list.
// Defined at module level (inside app.rs is fine, above or below App impl).
enum ReviewRow {
    Entry {
        entry: TimeEntry,
        task_name: String,
    },
    Gap {
        minutes: i64,  // how long this untracked block lasted
    },
}
```

```rust
fn build_review_rows(&self) -> Vec<ReviewRow> {
    use chrono::{DateTime, Utc};

    let date_str = self.review_date.format("%Y-%m-%d").to_string();

    // Collect all entries for this date. Include completed entries and,
    // if today is selected and a timer is active, include the active entry too.
    let mut day_entries: Vec<TimeEntry> = self.entries.iter()
        .filter(|e| {
            e.started_at.starts_with(&date_str)
                && (e.ended_at.is_some() || e.is_active())
        })
        .cloned()
        .collect();

    // Also include active_entry if it's for today and not already in entries
    // (it is mirrored in self.entries, so this is typically a no-op — defensive)
    if let Some(active) = &self.active_entry {
        if active.started_at.starts_with(&date_str) {
            if !day_entries.iter().any(|e| e.id == active.id) {
                day_entries.push(active.clone());
            }
        }
    }

    // Sort chronologically by started_at.
    day_entries.sort_by(|a, b| a.started_at.cmp(&b.started_at));

    // Build interleaved entry + gap rows.
    let mut rows: Vec<ReviewRow> = Vec::new();
    let mut prev_end: Option<DateTime<Utc>> = None;

    for entry in day_entries {
        // Check for a gap before this entry.
        if let Some(prev) = prev_end {
            if let Ok(this_start) = DateTime::parse_from_rfc3339(&entry.started_at) {
                let gap_minutes = (this_start.with_timezone(&Utc) - prev).num_minutes();
                if gap_minutes >= 5 {
                    // Only show gaps of 5 minutes or more — sub-5-minute gaps are noise.
                    rows.push(ReviewRow::Gap { minutes: gap_minutes });
                }
            }
        }

        // Advance prev_end to the end of this entry.
        if let Some(ended) = &entry.ended_at {
            if let Ok(dt) = DateTime::parse_from_rfc3339(ended) {
                prev_end = Some(dt.with_timezone(&Utc));
            }
        } else {
            // Active entry — use now as the effective end.
            prev_end = Some(Utc::now());
        }

        let task_name = self.tasks.iter()
            .find(|t| t.id == entry.task_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "Unknown task".to_string());

        rows.push(ReviewRow::Entry { entry, task_name });
    }

    rows
}
```

**Why 5-minute threshold?** Gaps shorter than 5 minutes are typically context switches, not
real breaks — showing them as "Untracked" would create noise. You can adjust this value later.

---

## Step 7 — Add `review_view()` method

```rust
fn review_view(&self) -> Element<'_, Message> {
    let today = chrono::Utc::now().date_naive();
    let date_label = if self.review_date == today {
        format!("Today — {}", self.review_date.format("%A, %-d %B %Y"))
    } else {
        self.review_date.format("%A, %-d %B %Y").to_string()
    };

    // --- Date navigation header ---
    let header = row![
        button(text("← Prev")).on_press(Message::ReviewPrevDay),
        text(date_label).width(Length::Fill),
        button(text("Next →"))
            .on_press_maybe(
                (self.review_date < today).then_some(Message::ReviewNextDay)
            ),
        button(text("Close")).on_press(Message::CloseReview),
    ]
        .padding(12)
        .spacing(8);

    // --- Build row list ---
    let rows = self.build_review_rows();

    let mut total_tracked: i64 = 0;
    let mut total_gap: i64 = 0;
    let mut first_start: Option<String> = None;
    let mut last_end: Option<String> = None;

    let mut items: Vec<Element<Message>> = Vec::new();

    if rows.is_empty() {
        items.push(
            container(text("No entries for this day."))
                .padding(20)
                .into()
        );
    }

    for row_item in &rows {
        match row_item {
            ReviewRow::Entry { entry, task_name } => {
                let start_str = Self::format_hhmm(&entry.started_at);
                let end_str = entry.ended_at.as_deref()
                    .map(Self::format_hhmm)
                    .unwrap_or_else(|| "running".to_string());

                let net_min = entry.minutes.unwrap_or_else(|| {
                    // Active entry — compute live elapsed.
                    use chrono::{DateTime, Utc};
                    DateTime::parse_from_rfc3339(&entry.started_at)
                        .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_minutes().max(0))
                        .unwrap_or(0)
                });
                let h = net_min / 60;
                let m = net_min % 60;
                let duration_label = if h > 0 {
                    format!("{h}h {m}m")
                } else {
                    format!("{m}m")
                };

                // Pause annotation — shown only if the entry had pauses.
                let pause_label = if entry.total_paused > 0 {
                    format!("  ({} min paused)", entry.total_paused)
                } else {
                    String::new()
                };

                // Track summary values.
                total_tracked += net_min;
                if first_start.is_none() {
                    first_start = Some(start_str.clone());
                }
                if entry.ended_at.is_some() {
                    last_end = Some(end_str.clone());
                }

                // Note: edit or display.
                let note_widget: Element<Message> = if self.editing_note_id == Some(entry.id) {
                    // Active edit field for this entry.
                    row![
                        text_input("Add a note...", &self.editing_note_text)
                            .on_input(Message::EditNoteChanged)
                            .on_submit(Message::SaveEditNote)
                            .width(Length::Fill),
                        button(text("Save")).on_press(Message::SaveEditNote),
                        button(text("Cancel")).on_press(Message::CancelEditNote),
                    ]
                        .spacing(4)
                        .into()
                } else {
                    let note_text = entry.notes.as_deref().unwrap_or("—");
                    row![
                        text(note_text).width(Length::Fill),
                        button(text("Edit")).on_press(Message::OpenEditNote(entry.id)),
                    ]
                        .spacing(4)
                        .into()
                };

                let entry_row = row![
                    text(task_name).width(Length::FillPortion(3)),
                    text(format!("{start_str} → {end_str}")).width(Length::FillPortion(2)),
                    text(format!("{duration_label}{pause_label}")).width(Length::FillPortion(2)),
                    note_widget,
                ]
                    .padding(8)
                    .spacing(8);

                items.push(entry_row.into());
            }

            ReviewRow::Gap { minutes } => {
                let h = minutes / 60;
                let m = minutes % 60;
                let gap_label = if h > 0 {
                    format!("⊘  Untracked — {h}h {m}m")
                } else {
                    format!("⊘  Untracked — {m}m")
                };
                total_gap += minutes;
                items.push(
                    container(text(gap_label))
                        .padding(iced::Padding::new(4.0).left(16.0))
                        .into()
                );
            }
        }
    }

    // --- Summary bar ---
    let first_str = first_start.as_deref().unwrap_or("—");
    let last_str = last_end.as_deref().unwrap_or("—");
    let tracked_h = total_tracked / 60;
    let tracked_m = total_tracked % 60;
    let gap_h = total_gap / 60;
    let gap_m = total_gap % 60;

    let summary = row![
        text(format!("First: {first_str}")),
        text(format!("Last: {last_str}")),
        text(format!("Tracked: {tracked_h}h {tracked_m}m")).width(Length::Fill),
        text(format!("Untracked: {gap_h}h {gap_m}m")),
    ]
        .padding(8)
        .spacing(16);

    // --- Compose full layout ---
    column![
        header,
        scrollable(column(items).spacing(2)).height(Length::Fill),
        summary,
    ]
        .into()
}
```

**Why `on_press_maybe`?** iced's `button()` has `on_press_maybe(Option<Message>)` — if the
option is `None`, the button renders but is not clickable (no hover effect, no press). This
is the correct way to disable a button without extra state. Here we disable "Next →" when
already on today.

---

## Step 8 — Update `view()`

```rust
pub fn view(&self) -> Element<'_, Message> {
    match self.screen {
        Screen::Review => return self.review_view(),
        Screen::Tracker => {}
    }

    let base = container(
        column![self.timer_bar(), self.task_list(), self.status_bar()].spacing(0)
    )
        .width(Length::Fill)
        .height(Length::Fill);

    if self.stop_prompt_open {
        stack![base, self.stop_prompt_view()].into()
    } else {
        base.into()
    }
}
```

The `match` at the top of `view()` acts as a screen router. Tracker is the fallthrough.

---

## Step 9 — Verify

```
cargo run
```

Test in this order:

1. Add two tasks, start and stop both timers with notes
2. Click **Review** — the date header shows today with a formatted date
3. Both entries appear in chronological order with correct task names, times, and minutes
4. If there was a gap between the entries, a "⊘ Untracked" row appears between them
5. Click **Edit** on an entry note — the text field appears pre-filled with the current note
6. Edit the text and click **Save** — note updates in the row
7. Click **Edit** again, change text, click **Cancel** — note is unchanged
8. Click **← Prev** — previous day shown (likely empty)
9. Click **Next →** — back to today
10. Click **Close** — back to the tracker view, timer still running if it was
11. Restart the app, go to Review — edited notes are still there (persisted in JSON)

---

## What this phase does not do

- **No delete** — entries and tasks cannot be deleted from the review. That is Phase 6.
- **No entry note editing for the active timer** — the active entry row shows "running" as
  its end time. If you want to add a note to the active session, use the stop prompt.
- **No timezone conversion** — times display in UTC. If the system timezone matters for
  TANNSS, that is a future concern.
- **No print or export** — the review is read-only reporting. Copy-paste to TANNSS manually.

---

## Commit message (draft)

```
feat(app): phase 5 — daily review view with inline note editing

Clicking Review switches to a date-navigable log of the day's time entries.
Entries show task name, start/end times, net minutes, and pause annotation.
Gaps of 5+ minutes between entries appear as Untracked rows. Notes can be
edited inline with the same copy-on-open/apply-on-save pattern used for
task editing. A summary bar shows first start, last end, and totals.
```
