# Feature — Phase 1: Project Setup and Core Data Layer

**Date:** 2026-04-03
**Mode:** FEATURE
**Architecture reference:** `docs/Brainstorm-RustIcedRewrite.md`
**Permanent record:** `docs/features/Phase1CoreSetup.md`

---

## What this phase builds

By the end of Phase 1, you will have:
- A Rust project that compiles and opens a window
- The Task and TimeEntry data structs with serialisation
- JSON load/save working on disk
- A three-zone window layout (timer bar / task list / status bar)
- An empty task list and a non-functional but rendering UI

Nothing is wired up yet — buttons do not work, the timer does not tick. That is intentional.
Get the structure right first. Everything in later phases slots into what you build here.

---

## Task 1 — Create the Rust project

In your terminal, from whatever directory you keep your projects:

```
cargo new tasktracker
cd tasktracker
git init
```

`cargo new` creates a minimal project and already adds a `.gitignore` that excludes the
`target/` build directory (which gets large). The `git init` makes it a repository.

```
tasktracker/
├── .gitignore      ← already excludes /target
├── Cargo.toml      ← dependencies and project metadata (like go.mod)
└── src/
    └── main.rs     ← entry point (like cmd/main.go)
```

**Copy your docs into the new project now:**

```
cp -r /path/to/old/track/docs ./docs
```

**Create `CLAUDE.md` at the project root** — copy the template from the bottom of
`docs/Structure-RustModules.md`. This is the project-level instruction file for future
Claude sessions.

You now have a clean starting point. Everything below builds on this.

---

## Task 2 — Cargo.toml: dependencies

`Cargo.toml` is Rust's equivalent of `go.mod` — it declares your project and its dependencies.
Open it and replace the contents with this:

```toml
[package]
name = "tasktracker"
version = "0.1.0"
edition = "2021"

[dependencies]
# The GUI framework
iced = { version = "0.14", features = ["tokio"] }

# Serialisation — turns your structs into JSON and back
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# UUIDs — unique IDs for tasks and entries without a database to coordinate
uuid = { version = "1", features = ["v4", "serde"] }

# Date and time handling
chrono = { version = "0.4", features = ["serde"] }

# Platform-appropriate data directory (~/.tasktracker on Linux)
dirs = "5"
```

**What `features = [...]` means:**

Rust crates can be split into optional parts called features. You opt into only what you need.
This keeps compile times down and avoids pulling in code you will not use.

- `serde/derive` — enables the `#[derive(Serialize, Deserialize)]` macro. Without this,
  you would have to implement serialisation by hand for every struct.
- `uuid/v4` — enables UUID v4 generation (random UUIDs). `uuid/serde` enables serialising
  UUIDs directly to/from JSON strings.
- `iced/tokio` — iced uses an async runtime for subscriptions (the timer tick). `tokio`
  is the standard async runtime in Rust.
- `chrono/serde` — same idea: enables `DateTime` values to serialise/deserialise automatically.

After editing `Cargo.toml`, run `cargo build` once. Cargo will download and compile all
dependencies. This takes a few minutes the first time. Subsequent builds are fast.

---

## Task 3 — Create the file structure

Create these directories and empty files. This is Phase 1's complete structure — nothing more.

```
src/
├── main.rs          (already exists — keep it, you will replace its contents)
├── app.rs           (create — the iced application: App struct, update, view)
└── data/
    ├── mod.rs       (create — declares the data submodule)
    ├── task.rs      (create — Task struct)
    ├── entry.rs     (create — TimeEntry struct)
    └── store.rs     (create — JSON load/save)
```

**There is no `src/ui/` directory yet.** In Phase 1, all view functions (timer bar, task list,
status bar) live as methods on `App` inside `app.rs`. When `app.rs` grows large enough to be
unwieldy, Phase 3 extracts them into `src/ui/`. See `docs/Structure-RustModules.md` for the
full picture and how that extraction works.

In Rust, every directory that contains `.rs` files needs a `mod.rs` file to declare it as
a module. Think of it like an `index.ts` in TypeScript — it is the entry point for that module.
See `docs/Structure-RustModules.md` for a full explanation of how Rust finds your files.

---

## Task 4 — The Task struct (`src/data/task.rs`)

This is where the Rust concepts are densest. Read the explanation alongside the code.

```rust
// src/data/task.rs

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- The TaskStatus enum ---

// In Go, you expressed status as a string constant ("todo", "in_progress", "done").
// In Rust, enums are far more powerful. A Rust enum is a type that can be exactly
// one of a fixed set of variants — and the compiler enforces that you handle all of them.
//
// #[derive(...)] is a macro that automatically generates trait implementations for you.
// Here:
//   - Serialize, Deserialize → serde handles JSON conversion automatically
//   - Debug → lets you print the value with {:?} for debugging
//   - Clone → lets you make copies of the value with .clone()
//   - PartialEq → lets you compare two values with ==

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

// impl means "implement methods on this type" — same idea as Go's method receivers.
// In Go: func (s TaskStatus) Next() TaskStatus { ... }
// In Rust: impl TaskStatus { fn next(&self) -> TaskStatus { ... } }

impl TaskStatus {
    // Cycles through statuses when the user taps the status indicator on a task row.
    // &self means "a reference to self" — we are reading self, not consuming it.
    // -> TaskStatus means this function returns a TaskStatus value.
    pub fn next(&self) -> TaskStatus {
        match self {
            TaskStatus::Todo       => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Done,
            TaskStatus::Done       => TaskStatus::Todo,
        }
    }

    // A human-readable label for the UI.
    // &str is a string slice — a reference to string data.
    // The 'static lifetime means this string lives for the whole program (it's a literal).
    // You don't need to fully understand lifetimes yet — just know &'static str means
    // "a string literal baked into the binary".
    pub fn label(&self) -> &'static str {
        match self {
            TaskStatus::Todo       => "To Do",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Done       => "Done",
        }
    }
}

// --- The Task struct ---

// Same idea as the Go Task struct, but with Rust types.
// pub means this struct is visible outside this module (like capitalisation in Go).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,

    pub name: String,

    pub status: TaskStatus,

    // Stored as an RFC 3339 string, same as the Go prototype.
    // We use String rather than chrono::DateTime because we only need to display it
    // and store it — we never do date arithmetic on created_at.
    pub created_at: String,

    pub urgent: bool,
    pub important: bool,

    // Option<T> is Rust's way of expressing "this value may or may not exist".
    // It replaces nullable pointers. In Go, you used *string for nullable strings.
    // In TypeScript, you used string | null.
    //
    // In Rust, Option<String> is either:
    //   Some("the actual string")   — value is present
    //   None                        — value is absent
    //
    // The compiler will not let you use an Option<String> as if it were a String.
    // You must explicitly handle both cases. This eliminates null pointer crashes.
    pub description: Option<String>,
}

impl Task {
    // A constructor — Rust does not have a `new` keyword, but by convention you
    // define a `new` function in the impl block that builds the struct.
    pub fn new(name: String) -> Self {
        // Self means "the type we are implementing" — same as writing Task { ... }
        Self {
            id: Uuid::new_v4(),     // generates a random UUID
            name,                   // shorthand: if variable name matches field name,
                                    // you don't repeat it. Same as JS shorthand { name }.
            status: TaskStatus::Todo,
            created_at: Utc::now().to_rfc3339(),
            urgent: false,
            important: false,
            description: None,      // None = no description yet
        }
    }

    // Returns which Eisenhower quadrant this task belongs to.
    // u8 is an unsigned 8-bit integer — values 1–4 are well within range.
    pub fn quadrant(&self) -> u8 {
        // match on a tuple (pair of values) — Rust pattern matching is very expressive
        match (self.urgent, self.important) {
            (true,  true)  => 1,    // Do First
            (false, true)  => 2,    // Schedule
            (true,  false) => 3,    // Delegate
            (false, false) => 4,    // Eliminate
        }
    }

    // Returns true if either flag is set — used to split the task list into
    // "sorted" and "needs sorting" sections.
    pub fn has_priority(&self) -> bool {
        self.urgent || self.important
    }
}
```

**The key Rust concept here: `match` is exhaustive.**

In Go, a `switch` without a `default` case compiles fine — it just falls through silently.
In Rust, `match` must cover every possible value. If you add a new `TaskStatus` variant
later and forget to handle it in `next()`, the compiler will refuse to compile with a clear
error telling you exactly which arms are missing. This is one of Rust's most powerful safety
features — it is impossible to silently miss a case.

---

## Task 5 — The TimeEntry struct (`src/data/entry.rs`)

```rust
// src/data/entry.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: Uuid,

    pub task_id: Uuid,      // references a Task — same concept as the FK in SQLite

    pub started_at: String, // RFC 3339

    // Option<String> for nullable fields — same pattern as Task::description.
    // These are None while the timer is running, Some(...) after it stops.
    pub ended_at: Option<String>,
    pub minutes: Option<i64>,   // i64 = signed 64-bit integer (can be negative, but won't be)
    pub notes: Option<String>,
}

impl TimeEntry {
    pub fn new(task_id: Uuid) -> Self {
        use chrono::Utc;
        Self {
            id: Uuid::new_v4(),
            task_id,
            started_at: Utc::now().to_rfc3339(),
            ended_at: None,
            minutes: None,
            notes: None,
        }
    }

    // Returns true if this entry has not been stopped yet.
    // ended_at.is_none() checks if the Option is None.
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }
}
```

---

## Task 6 — JSON persistence (`src/data/store.rs`)

This is where you learn two important Rust concepts: `Result<T, E>` and the `?` operator.

**`Result<T, E>` — Rust's error handling**

In Go, functions that can fail return two values: `(T, error)`. You check the error every time.

In Rust, functions that can fail return a `Result<T, E>`, which is either:
- `Ok(value)` — success, contains the value
- `Err(error)` — failure, contains the error

The `?` operator is shorthand for "if this is an `Err`, return it immediately; if it's `Ok`,
unwrap the value and continue." It is equivalent to Go's:
```go
if err != nil { return nil, err }
```

So instead of writing that check after every operation, you write `?` and Rust handles it.

```rust
// src/data/store.rs

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::entry::TimeEntry;   // super:: means "the parent module" (data/)
use super::task::Task;

// We serialise both collections into one wrapper struct.
// This way there is one file to read and write.
//
// #[serde(default)] means: if a field is missing from the JSON file,
// use its Default trait value. Vec::default() is an empty Vec.
// This lets old data files load cleanly if we add new fields later.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppData {
    #[serde(default)]
    pub tasks: Vec<Task>,

    #[serde(default)]
    pub entries: Vec<TimeEntry>,
}

// Returns the path to our data file: ~/.tasktracker/data.json
// PathBuf is an owned, heap-allocated path — like String but for file paths.
pub fn data_path() -> PathBuf {
    // dirs::home_dir() returns Option<PathBuf> — the home directory might not exist
    // on some systems. We use unwrap_or_else to fall back to the current directory.
    //
    // unwrap_or_else takes a closure (anonymous function) that runs only if the value is None.
    // The closure syntax is: |argument| expression
    // Here we take no arguments (the || is empty), and return PathBuf::from(".")
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // .join() appends a path component — like path.Join() in Go
    let dir = home.join(".tasktracker");

    // Create the directory if it does not exist. The "all" means create parent
    // directories too (like mkdir -p). We ignore the error here — if it fails,
    // the write will fail too and we will catch it then.
    let _ = fs::create_dir_all(&dir);

    dir.join("data.json")
}

// Loads app data from disk. Returns an empty AppData if the file does not exist yet.
//
// The return type is Result<AppData, String>.
//   - On success: Ok(AppData)
//   - On failure: Err(String) containing an error message
//
// We use String for errors here to keep it simple. A production app would use
// a proper error enum, but that is complexity you do not need yet.
pub fn load() -> Result<AppData, String> {
    let path = data_path();

    // If the file does not exist, return empty data — this is first run.
    // path.exists() returns a bool.
    if !path.exists() {
        return Ok(AppData::default());
    }

    // fs::read_to_string reads the whole file into a String.
    // It returns Result<String, std::io::Error>.
    // The ? at the end: if it is an Err, convert it to a String and return early.
    // .map_err(|e| e.to_string()) converts the io::Error into a String.
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read data file: {e}"))?;

    // serde_json::from_str parses JSON text into our AppData struct.
    // The ? again: return early if parsing fails.
    let data: AppData = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse data file: {e}"))?;

    Ok(data)
}

// Saves app data to disk.
// &AppData is a reference — we are borrowing the data to read it, not taking ownership.
pub fn save(data: &AppData) -> Result<(), String> {
    let path = data_path();

    // serde_json::to_string_pretty serialises to nicely indented JSON.
    // Human-readable files are useful when you want to inspect or debug your data directly.
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialise data: {e}"))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write data file: {e}"))?;

    Ok(())
}
```

---

## Task 7 — Wire the data module (`src/data/mod.rs`)

`mod.rs` declares which files are part of the `data` module. Without these declarations,
Rust does not know those files exist.

```rust
// src/data/mod.rs

// pub mod makes the module visible outside data/ (from app.rs, main.rs, etc.)
pub mod entry;
pub mod store;
pub mod task;
```

---

## Task 8 — The iced application skeleton (`src/main.rs` and `src/app.rs`)

This is where you meet iced's core model. Read the explanation carefully before looking
at the code — understanding the model makes the code obvious.

**The Elm architecture in iced**

iced uses a pattern called the Elm architecture (borrowed from the Elm language).
It has three parts that you implement, and iced calls them for you:

```
1. Model  — the complete state of your app (an App struct you define)
2. Update — a function that handles events and mutates the model
3. View   — a function that looks at the model and returns the UI to render
```

The cycle looks like this:

```
          ┌─────────────────────────────────────────┐
          │                                         │
          ▼                                         │
      view(model)          user clicks button
    renders the UI    ──►  iced captures the event
                           wraps it in a Message
                                    │
                                    ▼
                           update(model, message)
                           mutates the model
                                    │
                                    └──────────────►  loop
```

**Messages** are the key concept. Nothing mutates the model directly. Instead, every
interaction (a button click, a text field change, a timer tick) produces a `Message` value.
iced delivers that message to your `update()` function. `update()` is the *only* place
state changes. This is strict unidirectional data flow — the same idea as Svelte stores,
but more explicit.

Think of it like this: if in Svelte you would write `tasks = [...tasks, newTask]` inside
an event handler, in iced you instead emit a `Message::AddTask(name)` and handle it in
`update()`.

**The `update` function also returns a `Task` (iced's name for `Command`)**

iced renamed `Command` to `Task` in 0.13. It represents async work — something that runs
in the background and eventually produces another `Message`. For Phase 1 you will mostly
return `Task::none()` (do nothing async). File saving is small enough to do synchronously
inside `update()`.

Now the code:

```rust
// src/app.rs

use iced::widget::{column, container, row, text};
use iced::{Element, Length, Task};

use crate::data::entry::TimeEntry;
use crate::data::store::{self, AppData};
use crate::data::task::{Task as AppTask, TaskStatus};

// crate:: means "from the root of this crate" — like an absolute import path.
// We alias Task as AppTask because iced also has a type called Task, and we need
// to distinguish them. You will use iced::Task for iced commands, AppTask for your data.

// --- The Model ---

// This struct IS the entire app state. iced holds one instance of it.
// Everything the UI needs to render must be reachable from here.
pub struct App {
    tasks: Vec<AppTask>,
    entries: Vec<TimeEntry>,

    // The currently active time entry. We keep it separate for fast access —
    // the timer bar needs it every second.
    // Option<TimeEntry>: either Some(entry) if a timer is running, or None.
    active_entry: Option<TimeEntry>,
}

// --- The Message enum ---

// Every possible event in the app is one variant of this enum.
// Right now most interactions are not wired up — we will add variants as we build features.
// For Phase 1 we just need enough to make the app start and display.
#[derive(Debug, Clone)]
pub enum Message {
    // Fired every second when a timer is running. We will wire this to a Subscription later.
    Tick,

    // Placeholder messages for Phase 1 — no logic yet, just so the UI can compile
    // with buttons that reference them.
    AddTask,
    OpenReview,
}

// --- impl App ---
// This block implements the three things iced needs from us.

impl App {
    // new() is called once at startup. Load data from disk and initialise the model.
    //
    // The return type is (Self, Task<Message>).
    //   - Self: the initial App struct
    //   - Task<Message>: any async work to kick off at startup (none for now)
    pub fn new() -> (Self, Task<Message>) {
        // Load data from disk. If it fails, log the error and start with empty data.
        let data = store::load().unwrap_or_else(|e| {
            eprintln!("Failed to load data: {e}");
            AppData::default()
        });

        // Find the active entry (one without an ended_at) from the loaded entries.
        // iter() creates an iterator over the Vec.
        // find() returns the first element matching the condition, wrapped in Option.
        // cloned() turns &TimeEntry into TimeEntry (makes a copy).
        let active_entry = data.entries.iter()
            .find(|e| e.is_active())
            .cloned();

        let app = Self {
            tasks: data.tasks,
            entries: data.entries,
            active_entry,
        };

        // Task::none() means "no async work to start". We return this for now.
        (app, Task::none())
    }

    // update() receives a Message and mutates self accordingly.
    // Returns Task<Message> for any follow-up async work.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // The timer bar will redraw automatically when this message arrives.
                // No state change needed — view() will recalculate elapsed time.
            }
            Message::AddTask => {
                // Placeholder — Phase 2 will implement this properly
            }
            Message::OpenReview => {
                // Placeholder — Phase 5 will implement this
            }
        }

        Task::none()
    }

    // view() returns the UI tree. It is called after every update().
    // &self means we have a read-only reference — view() cannot mutate state.
    // It returns Element<Message> — a widget tree that can produce Message values.
    pub fn view(&self) -> Element<Message> {
        // column! is a macro that stacks widgets vertically.
        // It is equivalent to a VBox in Fyne or a flex-column in CSS.
        // Each argument is a widget that gets placed in order, top to bottom.
        let content = column![
            self.timer_bar(),
            self.task_list(),
            self.status_bar(),
        ]
        .spacing(0);   // no gap between zones — each zone manages its own padding

        // container wraps a widget and gives it sizing/alignment properties.
        // .width(Length::Fill) means "take all available horizontal space".
        // .height(Length::Fill) means "take all available vertical space".
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()     // .into() converts the container into an Element<Message>
                        // Most iced widgets need .into() at the end to become Element.
    }

    // These three functions build each zone of the UI.
    // They return Element<Message> — a piece of the widget tree.
    // Keeping them as separate functions makes view() readable and each zone easy to edit.

    fn timer_bar(&self) -> Element<Message> {
        let label = match &self.active_entry {
            Some(entry) => {
                // Later: look up the task name and show elapsed time.
                // For now: just show that something is running.
                format!("Timer running — task {}", entry.task_id)
            }
            None => "No timer running".to_string(),
        };

        container(text(label))
            .width(Length::Fill)
            .padding(12)
            .into()
    }

    fn task_list(&self) -> Element<Message> {
        if self.tasks.is_empty() {
            // .into() at the end of a widget converts it to Element<Message>
            return container(text("No tasks yet. Add one below."))
                .width(Length::Fill)
                .padding(20)
                .into();
        }

        // Build a column of task rows.
        // iter() iterates over &AppTask (references, not copies).
        // map() transforms each item — here, into a widget.
        // collect() gathers the results. The type annotation tells Rust what to collect into.
        let rows: Vec<Element<Message>> = self.tasks.iter()
            .map(|task| self.task_row(task))
            .collect();

        // column(rows) creates a column from a Vec — used when the list is dynamic.
        // column![] (the macro) is used when you know the widgets at compile time.
        // For dynamic lists, always use column(vec).
        column(rows)
            .spacing(4)
            .into()
    }

    fn task_row(&self, task: &AppTask) -> Element<Message> {
        // row! stacks widgets horizontally — like HBox or flex-row.
        row![
            text(&task.name).width(Length::Fill),   // name expands to fill space
            text(task.status.label()),               // status on the right
        ]
        .padding(8)
        .spacing(8)
        .into()
    }

    fn status_bar(&self) -> Element<Message> {
        // Today's total — placeholder text for now
        let total_label = text("Today: 0h 0m");

        row![
            total_label,
        ]
        .padding(8)
        .into()
    }

    // subscription() tells iced what background events to listen for.
    // iced calls this after every update() to know if anything changed.
    //
    // Return type: iced::Subscription<Message>
    //   A Subscription is a description of an ongoing event source — like a stream
    //   that produces Message values over time.
    //
    // In Phase 1, no timer is running so we return Subscription::none().
    // In Phase 2, when a timer is active, this will return a 1-second tick.
    // The method must exist now so that main.rs can reference it.
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}
```

---

## Task 9 — The entry point (`src/main.rs`)

```rust
// src/main.rs

// mod declarations tell Rust that these modules exist.
// Without these, the files in src/ are invisible to the compiler.
mod app;
mod data;

use app::{App, Message};

fn main() -> iced::Result {
    // iced::application() sets up the application.
    // It takes three things:
    //   1. A title string
    //   2. Your update function (App::update — a function pointer, not a call)
    //   3. Your view function   (App::view   — same)
    //
    // The method chaining (.subscription, .run_with) configures the builder
    // before handing control to iced.
    iced::application("Tasktracker", App::update, App::view)
        // Wire in the subscription so iced knows to call App::subscription
        // after each update. In Phase 1 this always returns Subscription::none(),
        // but the wiring must be in place for Phase 2 to just work.
        .subscription(App::subscription)
        // .run_with() is called instead of .run() when your app needs to initialise state.
        // It takes a function that returns (App, iced::Task<Message>).
        // App::new is a function pointer — it points to the App::new function we defined.
        .run_with(App::new)
}
```

**Why `App::update` and `App::view` as arguments, not calls?**

This is a function pointer — you are passing the function itself, not calling it.
iced stores these pointers and calls them for you at the right time.

In TypeScript, this is the difference between `arr.map(fn)` and `arr.map(fn())`.
You pass `fn`, the function, not `fn()`, its result.

---

## Task 10 — Verify it compiles and runs

```
cargo run
```

**Expected:** a window opens with "No timer running" at the top, "No tasks yet" in the middle,
and "Today: 0h 0m" at the bottom. It will not look polished yet — that is fine.

**If you see errors:** Rust's compiler errors are unusually good. Read the first error
carefully — it usually tells you exactly what is wrong and often suggests the fix.
The most common first-time errors are:

- `cannot find module` — you forgot a `mod foo;` declaration somewhere
- `cannot find type X in this scope` — you forgot a `use` statement
- `use of moved value` — an ownership issue; for now, try adding `.clone()` after the value

---

## What you have built

At the end of Phase 1 you have:
- A real iced application that compiles and opens a window
- Task and TimeEntry structs that can be serialised to/from JSON
- A persistence layer that loads on startup and is ready to save
- The three-zone layout skeleton
- A Message enum and update() that can be extended one message at a time

Phase 2 wires up the first real interaction: adding a task and starting a timer.

---

## Commit message (draft)

```
feat(chore): initialise Rust project with iced, data structs, and JSON persistence

Sets up the project structure with iced 0.14, serde/serde_json for JSON storage,
and the core Task/TimeEntry data model. Window opens with a three-zone skeleton layout.
No interactive features yet.
```
