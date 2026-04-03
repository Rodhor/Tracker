# Brainstorm — Rust + iced Rewrite

**Date:** 2026-04-03
**Decision:** Replace Go + Fyne with Rust + iced
**Storage decision:** JSON files instead of SQLite
**Follows:** `docs/Brainstorm-UIEvolution.md`, `docs/Brainstorm-FrameworkMigration.md`
**Updated:** 2026-04-03 — corrected iced 0.14 API, Rust 2024 lifetime syntax, resolved open questions

---

## What we are building

A clean rewrite of the tasktracker app in Rust using the iced GUI library. The Go prototype
established what the app should do and how the data should be structured. The rewrite takes
that knowledge and builds it properly — better visual quality, a single language throughout,
and a simpler storage layer for a tool of this size.

No features are added in this rewrite. The scope is: everything currently working in the
Go app, plus everything planned in `docs/Brainstorm-UIEvolution.md`, implemented in Rust.

---

## What carries over

| Thing | Status |
|-------|--------|
| Data model: Task with Eisenhower fields | Carries over — same fields, Rust struct |
| Data model: TimeEntry with notes | Carries over — same fields, Rust struct |
| Business rules: one active timer at a time | Carries over — enforced in update() |
| Business rules: auto-stop on new timer start | Carries over |
| Business rules: Eisenhower quadrant sorting | Carries over — same logic, Q1→Q2→Q3→Q4 |
| Feature: quick-add filter/create | Carries over |
| Feature: stop-with-note prompt | Carries over |
| Feature: review view with gap rows | Carries over |
| Feature: delete task (cascades to entries) | Carries over |
| Feature: text-based reporting | Carries over — simpler in Rust than Fyne |

## What does not carry over

| Thing | Reason |
|-------|--------|
| SQLite | Replaced with JSON files — simpler for this data size |
| Fyne UI patterns (refresh(), bindings, widgets) | iced has a different model entirely |
| Go store pattern | Replaced by iced's Model + a thin persistence layer |
| CGO-free constraint | Irrelevant in Rust |

---

## Framework decision: iced

**Why iced:**
- Pure Rust — single language, no TypeScript/JavaScript context switching
- Elm architecture (Model → Update → View) — explicit, traceable state
- Visual quality is achievable with iced's style system — no CSS but full control
  over colours, borders, radius, spacing per widget
- 30k+ stars, active development, 0.14 described as the last pre-1.0 release
- System76 COSMIC desktop is built on it — strongest signal it can produce a real product

**Known trade-offs:**
- Pre-1.0: APIs may change between versions. Mitigation: pin the version in Cargo.toml
  and update deliberately.
- System tray: not built in. Use `tray-icon` crate (Tauri org). Add after core app works.
- Global hotkey: `global-hotkey` crate (Tauri org). X11 works; Wayland TBD (deferred).
- No browser = no JS charting libraries. Not a concern: reports are text-based.

---

## Storage: JSON files

SQLite is appropriate for data that is queried, filtered, and sorted in the database.
This app's data set is small enough (hundreds of tasks, thousands of entries at most)
that loading everything into memory at startup and operating on it in Rust is faster
and simpler than SQL.

**Two files:**

```
~/.tasktracker/
├── tasks.json      # all Task records
└── entries.json    # all TimeEntry records
```

Use the `dirs` crate to resolve `~/.tasktracker/` correctly on Linux, Windows, and macOS
(`dirs::data_dir()` returns the platform-appropriate location).

**Serialisation:** `serde` + `serde_json`. Both structs derive `Serialize, Deserialize`.

**IDs:** `uuid` crate. Each Task and TimeEntry gets a UUID v4 on creation. No auto-increment
needed — no database to coordinate with.

**Write strategy:** Load once at startup into the Model. On every mutation, write the full
file back to disk. For a personal tool with data this size, this is correct — no partial
writes, no migration logic, no schema versioning.

**Timestamps:** Store as RFC 3339 strings (`chrono::DateTime<Utc>::to_rfc3339()`).
Consistent with the Go prototype.

---

## iced architecture

iced uses the Elm architecture. The entire app state lives in one `App` struct. Every
interaction produces a `Message`. The `update()` function handles messages and mutates
the Model. The `view()` function renders the current Model as a widget tree.

### The Model

```rust
struct App {
    // Data — loaded from JSON on startup
    tasks: Vec<Task>,
    entries: Vec<TimeEntry>,

    // Timer state — derived from entries on load, kept in sync
    active_entry: Option<TimeEntry>,  // the entry with ended_at == None

    // Navigation
    current_view: View,  // Main | Review

    // Quick-add panel state
    quick_add_open: bool,
    quick_add_input: String,
    quick_add_suggestions: Vec<usize>,  // indices into self.tasks
    quick_add_selected: Option<Uuid>,   // if user picked from suggestion list

    // Review state
    review_date: NaiveDate,

    // Stop-with-note modal state
    stop_prompt_open: bool,
    stop_prompt_note: String,
    stop_prompt_status: TaskStatus,
}

enum View {
    Main,
    Review,
}
```

### The Message enum

```rust
enum Message {
    // Timer
    Tick,                       // fires every second from subscription — no Instant needed,
                                // elapsed time is computed from active_entry.started_at in view()
    StartTimer(Uuid),           // start on this task (auto-stops previous)
    OpenStopPrompt,             // user clicked Stop — show the prompt
    StopPromptNoteChanged(String),
    StopPromptStatusChanged(TaskStatus),
    ConfirmStop,                // user confirmed — stop with note + status
    CancelStop,

    // Tasks
    AddTask { name: String, urgent: bool, important: bool },
    UpdateTaskMetadata { id: Uuid, urgent: bool, important: bool, description: Option<String> },
    CycleTaskStatus(Uuid),
    DeleteTask(Uuid),

    // Quick-add
    ToggleQuickAdd,
    QuickAddInputChanged(String),
    QuickAddSuggestionSelected(Uuid),
    QuickAddSubmit,

    // Review
    ShowReview,
    ShowMain,
    ReviewNavigate(i64),            // -1 = previous day, +1 = next day
    ReviewNoteChanged(Uuid, String),
    ReviewNoteSave(Uuid),
    ReviewDeleteEntry(Uuid),

    // Reporting
    ExportReport,               // writes text file for today's review date
}
```

### The subscription

The live timer display needs to update every second. iced handles this with a Subscription:

```rust
fn subscription(&self) -> Subscription<Message> {
    if self.active_entry.is_some() {
        // iced::time::every() produces Instant values — we discard them with |_|
        // because elapsed time is computed from active_entry.started_at, not from the Instant.
        iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    } else {
        Subscription::none()
    }
}
```

Only subscribes when a timer is running — no background ticking when idle.

---

## File structure

```
tasktracker/                    (new Rust project — cargo new tasktracker)
├── Cargo.toml
├── src/
│   ├── main.rs                 # iced::application() entry point
│   ├── app.rs                  # App struct, Message, update(), view(), subscription()
│   ├── data/
│   │   ├── mod.rs
│   │   ├── task.rs             # Task struct, TaskStatus enum, Quadrant helper
│   │   ├── entry.rs            # TimeEntry struct
│   │   └── store.rs            # load_tasks(), save_tasks(), load_entries(), save_entries(), data_dir()
│   └── ui/
│       ├── mod.rs
│       ├── task_list.rs        # task_list_view() — Eisenhower sections, row layout
│       ├── timer_bar.rs        # timer_bar_view() — active task name, elapsed, stop button
│       ├── status_bar.rs       # status_bar_view() — today's total, New Task, Review button
│       ├── quick_add.rs        # quick_add_view() — filter/create overlay panel
│       ├── stop_prompt.rs      # stop_prompt_view() — note + status modal
│       └── review.rs           # review_view() — date nav, entry list, gap rows, summary
├── docs/
│   └── (existing docs)
└── CLAUDE.md
```

**Rule:** `app.rs` owns the Model and all message handling. The `ui/` files are pure
view functions — they receive immutable references to the Model and return iced `Element`s.
They produce `Message` values but never mutate state directly. No state lives in view functions.

---

## Data model in Rust

### Task

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub status: TaskStatus,
    pub created_at: String,     // RFC 3339
    pub urgent: bool,
    pub important: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl Task {
    pub fn quadrant(&self) -> u8 {
        match (self.urgent, self.important) {
            (true,  true)  => 1,  // Do First
            (false, true)  => 2,  // Schedule
            (true,  false) => 3,  // Delegate
            (false, false) => 4,  // Eliminate
        }
    }

    pub fn has_priority(&self) -> bool {
        self.urgent || self.important
    }
}
```

### TimeEntry

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: Uuid,
    pub task_id: Uuid,
    pub started_at: String,     // RFC 3339
    pub ended_at: Option<String>,
    pub minutes: Option<i64>,
    pub notes: Option<String>,
}
```

---

## Feature scope

Everything in `docs/Brainstorm-UIEvolution.md` is in scope. Implementation order:

### Phase 1 — Core (app is usable)
1. Project setup: Cargo.toml with iced + serde + uuid + chrono + dirs
2. Data structs and JSON persistence (load/save)
3. Main window layout: timer bar + task list + status bar
4. Add task (name only, no priority)
5. Start and stop timer (auto-stop on new start)
6. Status cycling on tasks
7. Today's total in status bar
8. Live elapsed display in timer bar (subscription)

### Phase 2 — Quick-add panel
9. Quick-add overlay panel (toggle with button — hotkey wired later)
10. Filter existing tasks by name as you type
11. Create new task if no match selected
12. Urgent/Important toggles in quick-add (applies to new tasks only)

### Phase 3 — Eisenhower
13. Eisenhower fields on Task struct (already in model from Phase 1)
14. Sort task list by quadrant (Q1 → Q2 → Q3 → Q4)
15. "Needs sorting" section below sorted tasks
16. Edit dialog: set urgent/important/description on existing task
17. New task dialog gains priority fields

### Phase 4 — Stop prompt + notes
18. Stop-with-note modal (note field + status selector)
19. Notes stored on TimeEntry on stop

### Phase 5 — Review
20. Review view: date navigation, entry list, gap rows, summary bar
21. Inline notes editing in review
22. Delete time entry
23. Delete task (removes from list + entries)

### Phase 6 — Reporting
24. Export today's review as a plain text file
25. Open the exported file in the system's default text editor (`open` crate)

### Deferred (not in this rewrite scope)
- System tray (`tray-icon` crate — add after Phase 1 works)
- Global hotkey (`global-hotkey` crate — add after quick-add panel works without it)

---

## Key iced concepts to understand before starting

**1. No shared mutable state.** Everything goes through Messages. A button's `on_press`
returns a `Message` value, not a closure that mutates state. State only changes in `update()`.

**2. View functions are called every frame.** They are cheap Rust functions, not React-style
component trees with lifecycle hooks. Build views from the Model every time.

**3. `Task` (renamed from `Command` in iced 0.13) for async work.** File I/O on save can be done
synchronously in `update()` for this app's data size — no async needed. Return `Task::none()`
from every `update()` arm that does not kick off async work.

**4. `iced::widget::stack` for the quick-add panel and stop-prompt modal.** These are drawn
over the main content using `widget::stack`, which layers widgets on top of each other. The
old `container::overlay` API no longer exists in iced 0.14 — `stack` is the correct primitive.
iced does support multiple windows, but an in-window overlay is simpler for these use cases.

**5. Styling is per-widget.** Every widget's `.style()` method accepts a closure or a type
implementing that widget's style trait. Colours, borders, and radius are set in Rust, not CSS.
Define a `theme.rs` module with constants and style functions to keep it consistent.

---

## Open questions — resolved

1. **iced version:** Pinned to 0.14 in `Cargo.toml`. ✓
2. **Multi-window vs overlay for quick-add:** Overlay panel confirmed. `iced::widget::stack` is
   the correct widget. Multi-window available later if needed. ✓
3. **`data_dir()` path:** Using `dirs::home_dir()` → `~/.tracker/data.json`. Chosen for
   simplicity; can migrate to `dirs::data_dir()` later without breaking existing data. ✓

## iced 0.14 API — what changed from the planning docs

The original planning docs were written targeting iced 0.13's API. iced 0.14 changed the
`application()` entry point signature. All code examples in this repo reflect the 0.14 API.

### Old (0.13) — do not use
```rust
iced::application("Tasktracker", App::update, App::view)
    .subscription(App::subscription)
    .run_with(App::new)
```

### Correct (0.14)
```rust
iced::application(App::new, App::update, App::view)
    .title("Tasktracker")
    .subscription(App::subscription)
    .run()
```

**What changed:**
- First argument is the *boot function* (`App::new`), not the title string
- `.title()` is now a builder method on the returned `Application`
- `.run_with()` does not exist — the boot function is passed to `application()` directly
- `.run()` is the terminal call (no arguments)

## Rust 2024 edition notes

This project uses `edition = "2024"` in `Cargo.toml`. Two things from the 2024 edition affect
this codebase:

**1. `Element<'_, Message>` required on all view functions.**
Rust 2024 enforces the `mismatched_lifetime_syntaxes` lint as a hard warning. Any function
that takes `&self` and returns `Element<Message>` must write `Element<'_, Message>` instead —
the `'_` makes the borrow relationship explicit. Functions that borrow from a *parameter*
(not just `self`) need a named lifetime:

```rust
// Takes &self only — use '_
fn timer_bar(&self) -> Element<'_, Message> { ... }

// Borrows from the `task` parameter — use named lifetime
fn task_row<'a>(&self, task: &'a AppTask) -> Element<'a, Message> { ... }
```

**2. `gen` is a reserved keyword.**
Do not name any variable, function, or type `gen`. It is reserved for the upcoming
generator syntax. Use `generate`, `generated`, or a domain-specific name instead.
