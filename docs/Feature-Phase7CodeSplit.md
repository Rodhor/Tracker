# Feature Plan — Phase 7: Code Structure Split

## What this phase does

Splits `src/app.rs` (~1000 lines) into focused modules. No behaviour changes — the app
works identically before and after. Every view function moves to its own file under a new
`src/ui/` module. `app.rs` becomes a coordinator: App struct, Message, update(),
view(), subscription(), and private helper methods only.

---

## Target structure

```
src/
├── main.rs                  # entry point — declares mod app, mod data, mod ui
├── app.rs                   # App, Screen, Message, new(), update(), view(), subscription()
│                            # private helpers: save(), stop_active_timer(),
│                            #   pause_active_timer(), resume_active_timer()
├── data/
│   ├── mod.rs               # unchanged
│   ├── task.rs              # unchanged
│   ├── entry.rs             # unchanged
│   └── store.rs             # unchanged
└── ui/
    ├── mod.rs               # declares pub mod for each ui file
    ├── timer_bar.rs         # view() + elapsed_display()
    ├── task_list.rs         # view() + task_row_or_edit() + task_row()
    │                        #   + edit_row() + delete_task_confirm_row()
    ├── status_bar.rs        # view() + today_total_minutes()
    ├── stop_prompt.rs       # view()
    └── review.rs            # view() + ReviewRow + build_review_rows() + format_hhmm()
```

---

## Architecture rule

`app.rs` owns all state and all mutation. `ui/` files are **pure view functions** — they
receive `&App` and return `Element<Message>`. They read App state; they never mutate it.
All `Message` values are still produced in view functions (via `on_press` etc.) and handled
exclusively in `update()`.

---

## Key design decisions

**Free functions, not impl blocks.**
Each ui file exports a `pub fn view(app: &App) -> Element<'_, Message>`. This makes the
dependency explicit in the function signature and keeps `impl App` entirely in `app.rs`.

**`pub(crate)` fields.**
The view functions need to read App fields. Rust's privacy rules prevent cross-module field
access unless fields are `pub(crate)`. This change is safe — the crate is a single binary,
nothing is exported to external consumers.

**One file at a time, compile after each.**
Do steps 3–7 in order. After moving each file's functions and removing them from `app.rs`,
run `cargo check` before moving to the next. This keeps errors small and localised.

---

## Step 1 — Add `mod ui;` to `main.rs`

```rust
mod app;
mod data;
mod ui;       // ← add this line

use app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Tasktracker")
        .subscription(App::subscription)
        .run()
}
```

---

## Step 2 — Create `src/ui/mod.rs`

```rust
pub mod review;
pub mod status_bar;
pub mod stop_prompt;
pub mod style;        // empty for now — style functions live here in Phase 13
pub mod task_list;
pub mod timer_bar;
```

Create six empty files:

```
src/ui/timer_bar.rs     — create empty
src/ui/task_list.rs     — create empty
src/ui/status_bar.rs    — create empty
src/ui/stop_prompt.rs   — create empty
src/ui/review.rs        — create empty
src/ui/style.rs         — create empty (used in Phase 13: UI Polish)
```

`style.rs` is intentionally empty now. Its purpose: when Phase 13 arrives, all button and
widget style functions go here, and every ui file calls `use crate::ui::style` to apply
them. Establishing the module now means Phase 13 is "fill in style.rs and add .style()
calls" rather than "hunt through 5 files consolidating scattered inline styles."

**Rule for all ui files:** never write inline colours, hardcoded padding magic values, or
style closures directly on widgets. Leave them as default for now. Phase 13 fills in
`style.rs` and wires it up.

Run `cargo check` — should compile with warnings about empty modules.

---

## Step 3 — Make `App` fields `pub(crate)`

In `src/app.rs`, add `pub(crate)` to every field in the `App` struct:

```rust
pub struct App {
    pub(crate) tasks: Vec<AppTask>,
    pub(crate) entries: Vec<TimeEntry>,
    pub(crate) active_entry: Option<TimeEntry>,
    pub(crate) new_task_input: String,

    pub(crate) editing_task_id: Option<Uuid>,
    pub(crate) edit_urgent: bool,
    pub(crate) edit_important: bool,
    pub(crate) edit_description: String,

    pub(crate) stop_prompt_open: bool,
    pub(crate) stop_prompt_note: String,
    pub(crate) stop_prompt_status: TaskStatus,

    pub(crate) review_date: chrono::NaiveDate,
    pub(crate) editing_note_id: Option<Uuid>,
    pub(crate) editing_note_text: String,

    pub(crate) deleting_entry_id: Option<Uuid>,
    pub(crate) deleting_task_id: Option<Uuid>,

    pub(crate) screen: Screen,
}
```

Run `cargo check` — should compile identically (the change is purely additive for a
single-binary crate).

---

## Step 4 — Move `review_screen()` to `src/ui/review.rs`

Copy this block into `src/ui/review.rs`:

```rust
use crate::app::{App, Message};
use crate::data::entry::TimeEntry;
use chrono::{DateTime, Utc};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use uuid::Uuid;

pub enum ReviewRow {
    Entry { entry: TimeEntry, task_name: String },
    Gap { minutes: i64 },
}

pub fn view(app: &App) -> Element<'_, Message> {
    // paste the full body of review_screen() here,
    // replacing every self.x with app.x
    // replacing every Self::format_hhmm with format_hhmm
    // replacing every self.build_review_rows() with build_review_rows(app)
}

pub fn build_review_rows(app: &App) -> Vec<ReviewRow> {
    // paste the full body of build_review_rows() here,
    // replacing every self.x with app.x
}

pub fn format_hhmm(rfc3339: &str) -> String {
    // paste the full body of format_hhmm() here — no self references
}
```

**In `src/app.rs`:**

- Delete the `ReviewRow` enum
- Delete `review_screen()`, `build_review_rows()`, `format_hhmm()`
- Remove any imports only used by those functions
- In `view()`, change the review route to:
  ```rust
  Screen::Review => return crate::ui::review::view(self),
  ```

Run `cargo check`.

---

## Step 5 — Move `stop_prompt_view()` to `src/ui/stop_prompt.rs`

```rust
use crate::app::{App, Message};
use crate::data::task::TaskStatus;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    // paste the full body of stop_prompt_view() here,
    // replacing every self.x with app.x
}
```

**In `src/app.rs`:**

- Delete `stop_prompt_view()`
- In `view()`, change to:
  ```rust
  stack![base, crate::ui::stop_prompt::view(self)].into()
  ```

Run `cargo check`.

---

## Step 6 — Move `status_bar()` to `src/ui/status_bar.rs`

```rust
use crate::app::{App, Message};
use chrono::{DateTime, Utc};
use iced::widget::{button, row, text, text_input};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    // paste the full body of status_bar() here,
    // replacing every self.x with app.x
    // replacing self.today_total_minutes() with today_total_minutes(app)
}

fn today_total_minutes(app: &App) -> i64 {
    // paste the full body of today_total_minutes() here,
    // replacing every self.x with app.x
}
```

**In `src/app.rs`:**

- Delete `status_bar()` and `today_total_minutes()`
- In `view()`, change to:
  ```rust
  column![
      crate::ui::timer_bar::view(self),
      crate::ui::task_list::view(self),
      crate::ui::status_bar::view(self),
  ]
  ```
  (You will add the other two in steps 7 and 8 — for now keep the old calls for timer_bar
  and task_list until those steps are done)

Run `cargo check`.

---

## Step 7 — Move task list functions to `src/ui/task_list.rs`

```rust
use crate::app::{App, Message};
use crate::data::task::Task as AppTask;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use uuid::Uuid;

pub fn view(app: &App) -> Element<'_, Message> {
    // paste the full body of task_list() here,
    // replacing every self.x with app.x
    // replacing self.task_row_or_edit(task) with task_row_or_edit(app, task)
}

fn task_row_or_edit<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    if app.deleting_task_id == Some(task.id) {
        delete_task_confirm_row(app, task)
    } else if app.editing_task_id == Some(task.id) {
        edit_row(app, task)
    } else {
        task_row(task)
    }
}

fn task_row<'a>(task: &'a AppTask) -> Element<'a, Message> {
    // paste the full body of task_row() here — no self references needed
}

fn edit_row<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    // paste the full body of edit_row() here,
    // replacing self.edit_urgent with app.edit_urgent, etc.
}

fn delete_task_confirm_row<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    // paste the full body of delete_task_confirm_row() here,
    // replacing self.entries with app.entries
}
```

Note: `task_row` no longer needs `app` — it only uses the task argument.

**In `src/app.rs`:**

- Delete `task_list()`, `task_row_or_edit()`, `task_row()`, `edit_row()`, `delete_task_confirm_row()`

Run `cargo check`.

---

## Step 8 — Move `timer_bar()` to `src/ui/timer_bar.rs`

```rust
use crate::app::{App, Message};
use chrono::{DateTime, Utc};
use iced::widget::{button, container, row, text};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    // paste the full body of timer_bar() here,
    // replacing every self.x with app.x
    // replacing Self::elapsed_display(...) with elapsed_display(...)
}

pub fn elapsed_display(started_at: &str, paused_at: Option<&str>, paused_minutes: i64) -> String {
    // paste the full body of elapsed_display() here — no self references
}
```

**In `src/app.rs`:**

- Delete `timer_bar()` and `elapsed_display()`

Run `cargo check`.

---

## Step 9 — Clean up `app.rs` `view()` and imports

After all moves, `view()` in `app.rs` should look like this:

```rust
pub fn view(&self) -> Element<'_, Message> {
    use crate::ui;

    match self.screen {
        Screen::Review => return ui::review::view(self),
        Screen::Tracker => {}
    }

    let base = container(
        column![
            ui::timer_bar::view(self),
            ui::task_list::view(self),
            ui::status_bar::view(self),
        ]
            .spacing(0),
    )
        .width(Length::Fill)
        .height(Length::Fill);

    if self.stop_prompt_open {
        stack![base, ui::stop_prompt::view(self)].into()
    } else {
        base.into()
    }
}
```

Remove any imports from `app.rs` that are no longer needed there. Keep:

```rust
use crate::data::entry::TimeEntry;
use crate::data::store::{self, AppData};
use crate::data::task::{Task as AppTask, TaskStatus};
use chrono::{DateTime, Utc};
use iced::widget::{container, column, stack};
use iced::{Element, Length, Task};
use uuid::Uuid;
```

Run `cargo check` — should compile clean with no warnings.

---

## Step 10 — Final check

```
cargo check
```

Expected: zero errors, zero warnings. If `cargo check` reports unused imports in any of
the new ui files, remove them. Do not suppress with `#[allow(unused_imports)]` — fix them.

---

## What does NOT change

- All `Message` variants — identical
- All `update()` arms — identical
- All behaviour — identical
- `data/` module — untouched
- `main.rs` — one line added (`mod ui;`)
- The JSON data file — untouched

---

## Commit message (draft)

```
refactor(app): split view functions into src/ui/ modules

Moves all rendering functions out of app.rs into focused files under
src/ui/. app.rs now owns only state, messages, update(), and private
helpers. No behaviour changes.
```
