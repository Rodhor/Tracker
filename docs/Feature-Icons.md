# Feature: Icons via iced_fonts

- **Mode:** FEATURE
- **Date:** 2026-04-08

---

## What it does

Replaces text labels on compact action buttons with Bootstrap Icons from the `iced_fonts` crate. No `.ttf` file required — the font bytes are bundled inside the crate and registered at startup.

---

## Dependency

Add to `Cargo.toml`:

```toml
iced_fonts = { version = "0.3.0", features = ["bootstrap"] }
```

---

## Font registration

In `src/main.rs`, chain `.font()` onto the application builder before `.run()`:

```rust
use iced_fonts::BOOTSTRAP_FONT_BYTES;

iced::application(App::new, App::update, App::view)
    .title("Tasktracker")
    .subscription(App::subscription)
    .theme(|_: &App| app_theme())
    .font(BOOTSTRAP_FONT_BYTES)   // ← add this
    .window(window::Settings { ... })
    .run()
```

---

## API — how iced_fonts 0.3.0 works

The API is **function-based**, not enum-based. Each icon is a free function in the `iced_fonts::bootstrap` module that directly returns a ready-to-use `Text` widget with the correct font already applied:

```rust
use iced_fonts::bootstrap;

// Each function returns a Text widget — pass it straight into button()
button(bootstrap::play_fill()).on_press(Message::StartTimer(task.id))
button(bootstrap::trash()).on_press(Message::RequestDeleteTask(task.id)).style(button::danger)
```

No helper function needed in `style.rs`. No `icon_to_string`. Just import the module and call the function.

Each view file that uses icons needs one new import:

```rust
use iced_fonts::bootstrap;
```

---

## Icon map

| Location | Current label | Function |
|----------|--------------|----------|
| `timer_bar` — note button | "Note" | `bootstrap::journal_text()` |
| `timer_bar` — pause | "Pause" | `bootstrap::pause_fill()` |
| `timer_bar` — resume | "Resume" | `bootstrap::play_fill()` |
| `timer_bar` — stop | "Stop" | `bootstrap::stop_fill()` |
| `task_list` — start row | "▶" | `bootstrap::play_fill()` |
| `task_list` — edit row open | "Edit" | `bootstrap::pencil()` |
| `task_list` — edit row save | "Save" | `bootstrap::floppy()` |
| `task_list` — edit row cancel | "Cancel" | `bootstrap::x()` |
| `task_list` — edit row delete | "Delete" | `bootstrap::trash()` |
| `task_list` — confirm delete | keep text — destructive action | — |
| `review` — prev day | "<- Prev" | `bootstrap::chevron_left()` |
| `review` — next day | "Next ->" | `bootstrap::chevron_right()` |
| `review` — close | "Close" | `bootstrap::x_lg()` |
| `review` — edit note | "Edit" | `bootstrap::pencil()` |
| `review` — copy note | "Copy" | `bootstrap::clipboard()` |
| `review` — edit time | "Edit time" | `bootstrap::clock_history()` |
| `review` — delete entry | "Delete" | `bootstrap::trash()` |
| `review` — confirm delete entry | keep text — destructive action | — |
| `review` — save note | "Save" | `bootstrap::check()` |
| `review` — cancel note | "Cancel" | `bootstrap::x()` |
| `review` — save time | "Save" | `bootstrap::check()` |
| `review` — cancel time | "Cancel" | `bootstrap::x()` |
| `stop_prompt` — confirm | keep text — primary action | — |
| `stop_prompt` — cancel | "Cancel" | `bootstrap::x()` |
| `note_modal` — save | keep text — primary action | — |
| `note_modal` — cancel | "Cancel" | `bootstrap::x()` |
| `status_bar` — review | "Review" | `bootstrap::clock_history()` |
| `quick_add` — cancel | "Cancel" | `bootstrap::x()` |

**Rule:** icon-only for compact repeated actions (edit, delete, copy, nav, close, cancel). Keep text for primary confirmations ("Stop and save", "Confirm delete") so the user always reads what they are committing to.

**Note on exact function names:** if any name above does not compile, check `https://docs.rs/iced_fonts/0.3.0/iced_fonts/bootstrap/` for the full list — names follow snake_case Bootstrap icon names.

---

## Implementation order

1. Add `iced_fonts` to `Cargo.toml`
2. Register `BOOTSTRAP_FONT_BYTES` in `main.rs`
3. Update `timer_bar.rs`
4. Update `task_list.rs`
5. Update `review.rs`
6. Update `stop_prompt.rs`
7. Update `note_modal.rs`
8. Update `status_bar.rs`
9. Update `quick_add.rs`

---

## Testing

- All icon buttons render glyphs rather than boxes (boxes = font not registered)
- Confirm destructive buttons still show text, not icon-only
- Layout does not break — icon buttons are narrower than text; verify nothing clips or wraps
- Pause/Resume icons switch correctly when timer state changes
