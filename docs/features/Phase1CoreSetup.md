# Phase 1 — Core Setup

- **Status:** Planned
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
- [ ] Create Rust project (`cargo new tasktracker`)
- [ ] Set up `Cargo.toml` with all dependencies (iced, serde, uuid, chrono, dirs)
- [ ] Create directory structure (`src/data/`, `src/ui/`)
- [ ] Implement `src/data/task.rs` — Task struct, TaskStatus enum, Quadrant helper
- [ ] Implement `src/data/entry.rs` — TimeEntry struct
- [ ] Implement `src/data/store.rs` — load/save JSON, data_path()
- [ ] Implement `src/data/mod.rs` — module declarations
- [ ] Implement `src/app.rs` — App struct, Message enum, update(), view()
- [ ] Implement `src/main.rs` — iced::application() entry point
- [ ] Verify: `cargo run` opens a window with three zones visible

---

## Technical Notes

*To be filled in after implementation. Say "I finished Phase 1" to trigger this.*

---

## Change History

*No changes yet.*
