# Phase 1 — Core Setup

- **Status:** Complete
- **Created:** 2026-04-03
- **Last updated:** 2026-04-03
- **Touches:** `Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/data/task.rs`, `src/data/entry.rs`, `src/data/store.rs`, `src/data/mod.rs`

---

## What It Does

Bootstraps the Rust + iced project. Establishes the Task and TimeEntry data structs
with JSON serialisation, a load/save persistence layer, and a three-zone window layout
(timer bar / task list / status bar). No interactions work yet — this phase is about
getting the structure right so everything else slots in cleanly.

---

## Why It Was Built

The Go + Fyne prototype established what the app should do. This phase starts the clean
rewrite in Rust + iced, implementing the architectural decisions from
`docs/Brainstorm-RustIcedRewrite.md`.

---

## Implementation Checklist

### Initial implementation — 2026-04-03
- [x] Create Rust project (`cargo new tasktracker`)
- [x] Set up `Cargo.toml` with all dependencies (iced, serde, uuid, chrono, dirs)
- [x] Create directory structure (`src/data/`, `src/ui/`)
- [x] Implement `src/data/task.rs` — Task struct, TaskStatus enum, Quadrant helper
- [x] Implement `src/data/entry.rs` — TimeEntry struct
- [x] Implement `src/data/store.rs` — load/save JSON, data_path()
- [x] Implement `src/data/mod.rs` — module declarations
- [x] Implement `src/app.rs` — App struct, Message enum, update(), view()
- [x] Implement `src/main.rs` — iced::application() entry point
- [x] Verify: `cargo run` opens a window with three zones visible

---

## Technical Notes

- **iced 0.14 `application()` API:** The planning doc was written for 0.13. In 0.14 the boot
  function is the first argument (`App::new`), the title moves to `.title()`, and `.run_with()`
  is gone — use `.run()` after chaining `.title()` and `.subscription()`.

- **`Element<'_, Message>` on all view functions:** Rust 2024's `mismatched_lifetime_syntaxes`
  lint requires the `'_` to be explicit on all functions returning `Element` that also borrow
  `&self`. `task_row` additionally needs a named `'a` because it borrows from a `task: &AppTask`
  parameter, not just from `self`.

- **`load_data()` / `save_data()` naming:** The implementation used `load_data` and `save_data`
  rather than the planning doc's `load` / `save`. This avoids any future ambiguity if the store
  module gains other functions with similar names.

- **`~/.tracker/data.json`:** The data path uses `dirs::home_dir()` + `.tracker` rather than
  the planning doc's `.tasktracker`. Shorter, less typing. Can rename later without data loss
  by simply moving the directory.

- **`Utc::now().to_string()` vs `to_rfc3339()`:** The implementation uses `.to_string()` on
  `DateTime<Utc>`, which formats as `"2026-04-03 12:00:00 UTC"`. The planning doc specified
  RFC 3339 (`"2026-04-03T12:00:00+00:00"`). Both round-trip through serde correctly since the
  field is stored as a plain `String`. Phase 2 will compute elapsed time by parsing this string
  — switch to `.to_rfc3339()` at that point for a cleaner, standard format.

- **Pattern followed:** iced Elm architecture — `App` struct holds all state, `Message` enum
  covers all events, `update()` is the sole mutation point, `view()` is a pure render function.

---

## Change History

*No changes yet.*
