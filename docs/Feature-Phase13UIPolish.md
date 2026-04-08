# Feature Plan — Phase 13: UI Polish

**Date:** 2026-04-08
**Permanent record:** `docs/features/Phase13UIPolish.md`

---

## What this phase builds

1. **Theme** — dark theme by default, switchable in one line of code
2. **Modal backgrounds** — panels appear solid above content, not transparent
3. **Layout fixes** — review rows restructured so buttons and text do not fight for space
4. **Separators** — thin lines between the three main zones
5. **Button hierarchy** — destructive red for Delete, primary for Save/Stop, muted for Cancel
6. **Active task highlight** — the tracked row has a tinted background
7. **"Needs Sorting" header muted** — visually quieter than the TODOS header
8. **Note modal auto-focus** — editor is focused when the modal opens (missed in Phase 9)
9. **Window minimum size** — prevents the window becoming unusably small

---

## Key API facts confirmed from source

**Themes:** `iced::Theme` is an enum with 22 built-in variants:
`Light`, `Dark`, `Dracula`, `Nord`, `SolarizedLight/Dark`, `GruvboxLight/Dark`,
`CatppuccinLatte/Frappe/Macchiato/Mocha`, `TokyoNight/Storm/Light`,
`KanagawaWave/Dragon/Lotus`, `Moonfly`, `Nightfly`, `Oxocarbon`, `Ferra`

**Button styles** (pass directly to `.style()`):
`button::primary`, `button::secondary`, `button::danger`, `button::success`,
`button::warning`, `button::text`, `button::background`

**Container styles** (pass directly to `.style()`):
- `container::rounded_box` — `background.weak` color, rounded corners — **use for modals**
- `container::bordered_box` — `background.weakest` color with border
- `container::transparent` — explicit no-background

**Separators:** `iced::widget::rule::horizontal(thickness)` — free function

**Text color:** `text("...").color(Color::from_rgb(r, g, b))`

---

## Step 1 — Theme selection (`src/main.rs`)

Define one function that returns the active theme. Change the single return value to switch:

```rust
use iced::window;

fn app_theme() -> iced::Theme {
    iced::Theme::Dark  // ← change this line to switch themes
    // Other options: iced::Theme::Dracula, ::CatppuccinMocha, ::Nord, etc.
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Tasktracker")
        .subscription(App::subscription)
        .theme(|_| app_theme())
        .window(window::Settings {
            size: iced::Size::new(800.0, 550.0),
            min_size: Some(iced::Size::new(640.0, 420.0)),
            ..window::Settings::default()
        })
        .run()
}
```

**Why a function and not a `const`?**

`iced::Theme` is not `Copy`, so it cannot be stored in a `const` that can be returned by
value. A function with a single return statement is just as easy to find and change, with no
tradeoffs.

**Why `.theme(|_| app_theme())`?**

`.theme()` takes a closure `|state| -> Theme`. Since the theme does not depend on app state
(no in-app switching), the state parameter is ignored. The closure calls `app_theme()` on
every render — negligible cost since it just returns an enum variant.

---

## Step 2 — Modal backgrounds (`src/ui/stop_prompt.rs`, `note_modal.rs`, `quick_add.rs`)

All three modal views currently use a plain `container()` with no background style. They are
fully transparent — the base UI is visible through them. Add `container::rounded_box` to each
panel container:

```rust
// In all three files, change the outer container call:

// Before:
container(panel)
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()

// After:
container(
    container(panel)
        .style(container::rounded_box)
        .padding(4)  // small extra padding so the box has breathing room
)
.width(Length::Fill)
.height(Length::Fill)
.center_x(Length::Fill)
.center_y(Length::Fill)
.into()
```

**Why two nested containers?**

The outer container centers the inner one within the full screen. The inner container
(`rounded_box`) provides the visible panel — background color, rounded corners. Applying
`rounded_box` to the outer container would make the entire screen one big rounded box.

**What `rounded_box` gives you:**

It uses `theme.extended_palette().background.weak.color` — a shade slightly different from
the main background (lighter on dark themes, slightly darker on light themes). It also sets
`text_color` so text within the panel contrasts correctly. The result is a panel that reads
as "a separate surface" sitting above the content.

---

## Step 3 — Separators between zones (`src/app.rs`)

Add one import and three separator widgets:

```rust
use iced::widget::rule;

// In view(), change the base column:
let base = container(
    column![
        ui::timer_bar::view(self),
        rule::horizontal(1),
        ui::task_list::view(self),
        rule::horizontal(1),
        ui::status_bar::view(self),
    ]
    .spacing(0),
)
```

---

## Step 4 — Fix review row layout (`src/ui/review.rs`)

**The problem:** the note widget is a `row!` with note text and four buttons. On most window
sizes these compress each other. Even with `Length::Fill` on the text, the buttons have fixed
natural widths that add up.

**The fix:** restructure the note widget into a `column` — text on top, buttons below.
This gives each element the full row width and removes the competition.

```rust
// Before (everything fighting in one row):
row![
    text(note_text).width(Length::Fill),
    button(text("Edit")).on_press(Message::OpenEditNote(entry.id)),
    if let Some(btn) = copy_btn { btn } else { button(text("Copy")) },
    button(text("Edit time")).on_press_maybe(...),
    button(text("Delete")).on_press_maybe(...),
]
.spacing(4)
.into()

// After (text above, controls below):
column![
    text(note_text).width(Length::Fill),
    row![
        button(text("Edit")).on_press(Message::OpenEditNote(entry.id)),
        if let Some(btn) = copy_btn { btn } else { button(text("Copy")) },
        button(text("Edit time")).on_press_maybe(...),
        button(text("Delete")).on_press_maybe(...),
    ]
    .spacing(4),
]
.spacing(4)
.width(Length::Fill)
.into()
```

Also give the outer `entry_row` an explicit `FillPortion` for the note column so the four
`FillPortion` columns divide the row predictably:

```rust
let entry_row = row![
    text(task_name).width(Length::FillPortion(2)),
    text(format!("{start_str} -> {end_str}")).width(Length::FillPortion(2)),
    text(format!("{duration_label} {pause_label}")).width(Length::FillPortion(1)),
    note_column.width(Length::FillPortion(3)),  // most space — has the most content
]
.padding(8)
.spacing(8);
```

**Why `FillPortion` instead of `Fill` for the note column?**

`Length::Fill` inside a row that already has `FillPortion` siblings can behave unexpectedly
in iced — `Fill` and `FillPortion` use different internal layout pass strategies. Making all
four columns use `FillPortion` puts them in the same layout pass, producing predictable
proportional sizing.

**Also fix the time edit row** which has the same structure issue — two `text_input` widgets
and two buttons all in one row. Keep the row structure here (it is intentional — it reads
as an inline form) but add explicit widths:

```rust
row![
    text(task_name).width(Length::Fill),
    text_input("HH:MM", &app.edit_time_start)
        .on_input(Message::EditTimeStartChanged)
        .width(Length::Fixed(60.0)),
    text(" → "),
    text_input("HH:MM", &app.edit_time_end)
        .on_input(Message::EditTimeEndChanged)
        .width(Length::Fixed(60.0)),
    button(text("Save")).on_press(Message::SaveEditTime).style(button::primary),
    button(text("Cancel")).on_press(Message::CancelEditTime).style(button::text),
]
```

`text(task_name).width(Length::Fill)` takes all remaining space after the fixed-width inputs
and button labels — no compression.

---

## Step 5 — Active task row highlight (`src/ui/style.rs` and `src/ui/task_list.rs`)

**`src/ui/style.rs`:**

```rust
use iced::widget::container;
use iced::{Background, Theme};

pub fn active_row(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.extended_palette().primary.weak.color,
        )),
        ..container::Style::default()
    }
}
```

**`src/ui/task_list.rs`:**

Change `task_row_or_edit` to compute and pass the active flag:

```rust
fn task_row_or_edit<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    let is_active = app.active_entry.as_ref().is_some_and(|e| e.task_id == task.id);

    if app.deleting_task_id == Some(task.id) {
        delete_task_confirm_row(app, task)
    } else if app.editing_task_id == Some(task.id) {
        edit_row(app, task)
    } else {
        task_row(task, is_active)
    }
}
```

Change `task_row` to wrap in a styled container when active:

```rust
use crate::ui::style;

fn task_row(task: &AppTask, is_active: bool) -> Element<'_, Message> {
    let r = row![
        button(text("▶")).on_press(Message::StartTimer(task.id)),
        text(&task.name).width(Length::Fill),
        button(text(task.status.label())).on_press(Message::CycleStatus(task.id)),
        button(text("Edit")).on_press(Message::OpenEditTask(task.id))
    ]
    .padding(8)
    .spacing(8);

    if is_active {
        container(r).style(style::active_row).width(Length::Fill).into()
    } else {
        r.into()
    }
}
```

---

## Step 6 — Mute "Needs Sorting" header (`src/ui/task_list.rs`)

```rust
use iced::Color;

// The TODOS header — unchanged, renders in default text color

// "Needs Sorting" — explicitly muted:
container(
    text("Needs Sorting").color(Color::from_rgb(0.55, 0.55, 0.55))
)
.padding(iced::Padding::new(8.0).bottom(4))
.into()
```

---

## Step 7 — Button styles throughout

Add `use iced::widget::button;` to each file. Apply as follows:

| File | Button | Style |
|------|--------|-------|
| `stop_prompt.rs` | Stop and save | `button::primary` |
| `stop_prompt.rs` | Cancel | `button::text` |
| `note_modal.rs` | Save | `button::primary` |
| `note_modal.rs` | Cancel | `button::text` |
| `quick_add.rs` | Create "..." | `button::primary` |
| `quick_add.rs` | Cancel | `button::text` |
| `task_list.rs` edit row | Save | `button::primary` |
| `task_list.rs` edit row | Cancel | `button::text` |
| `task_list.rs` edit row | Delete | `button::danger` |
| `task_list.rs` delete confirm | Confirm delete | `button::danger` |
| `task_list.rs` delete confirm | Cancel | `button::text` |
| `review.rs` note row | Delete | `button::danger` |
| `review.rs` note row | Edit | *(default)* |
| `review.rs` note row | Copy | *(default)* |
| `review.rs` note row | Edit time | *(default)* |
| `review.rs` delete confirm | Confirm delete | `button::danger` |
| `review.rs` delete confirm | Cancel | `button::text` |
| `review.rs` note editor | Save | `button::primary` |
| `review.rs` note editor | Cancel | `button::text` |
| `review.rs` time editor | Save | `button::primary` |
| `review.rs` time editor | Cancel | `button::text` |

**Why leave Edit / Copy / Edit time as default?**

These are navigation/utility actions — they open an editor or put text on the clipboard but
do not commit or destroy anything. Default (bordered, neutral) is the right visual weight:
present but not demanding attention.

---

## Step 8 — Note modal auto-focus (`src/ui/note_modal.rs` and `src/app.rs`)

**`src/ui/note_modal.rs`** — add a constant and an `.id()` call:

```rust
pub const NOTE_MODAL_ID: &str = "note_modal_editor";

// On the text_editor:
text_editor(&app.note_modal_content)
    .id(iced::widget::Id::new(NOTE_MODAL_ID))
    .on_action(Message::NoteModalChanged)
    .height(Length::Fixed(160.0)),
```

**`src/app.rs`** — import and return a focus task from `OpenNoteModal`:

```rust
use crate::ui::note_modal::NOTE_MODAL_ID;

// In OpenNoteModal arm, at the end (inside the is_some() guard):
self.note_modal_open = true;
use iced::widget::Id;
return iced::widget::operation::focus(Id::new(NOTE_MODAL_ID));
```

---

## Step 9 — Verify

```
cargo check
cargo run
```

1. **Theme** — the app opens in dark mode. Change `app_theme()` to `iced::Theme::Dracula`,
   recompile, verify it switches. Change back to `Theme::Dark`.
2. **Modals** — open stop prompt, note modal, quick-add. Each appears as a solid rounded
   panel over the dimmed content — not transparent.
3. **Separators** — thin lines visible between timer bar / task list / status bar.
4. **Review layout** — open review with several entries. Resize the window narrower. Notes
   text and buttons remain readable, no overlap.
5. **Active task** — start a timer. That task row has a faint tinted background.
6. **"Needs Sorting"** — visually greyer than the TODOS header above it.
7. **Button hierarchy** — delete buttons are red; save/stop buttons are filled/accent;
   cancel buttons are text-only.
8. **Note modal focus** — press N with a timer running. Start typing immediately, no click.

---

## What this phase does not do

- **No in-app theme switching** — one line of code to change, no UI controls
- **No custom colour palette** — all colors come from the chosen built-in theme
- **No icons** — text labels only
- **No Done task greying** — left for a future pass if list length becomes a problem

---

## Commit message (draft)

```
feat(ui): Phase 13 polish — theme, modal backgrounds, layout, button hierarchy

Sets dark theme via a single app_theme() function. Adds rounded_box
backgrounds to all modal panels so they appear solid. Restructures
review note rows from a single cramped row into a column layout.
Adds horizontal rule separators. Applies danger/primary/text button
styles and highlights the active task row.
```
