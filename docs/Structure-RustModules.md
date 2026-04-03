# Structure — Rust Module System and Project Layout

**Date:** 2026-04-03
**Mode:** STRUCTURE
**Applies to:** The Rust + iced tasktracker rewrite

---

## Why Rust's module system feels hard

Coming from Go or TypeScript, Rust's module system has one property that causes most of the
confusion: **the file system and the module tree are separate things**. In Go, a file's
location determines its package. In TypeScript, you import by file path directly. In Rust,
neither is true. You have to *explicitly declare* every module, and the file is only found
once it has been declared. If you create a file but forget to declare it, the compiler does
not know it exists — no error, it just ignores it.

Once you understand that one thing, the rest follows.

---

## How Rust finds your code

The compiler starts at `src/main.rs` (or `src/lib.rs`). That is the *crate root*.
Everything in your project must be reachable from there through a chain of declarations.

There are two ways to declare a module:

### 1. Inline declaration

The entire module lives in the same file, inside curly braces:

```rust
// src/main.rs
mod greet {
    pub fn hello() -> &'static str {
        "hello"
    }
}

fn main() {
    println!("{}", greet::hello());
}
```

This is fine for tiny helper code. You will rarely use it for real modules.

### 2. File declaration

The module lives in its own file. You declare it with `mod name;` (no braces):

```rust
// src/main.rs
mod greet;   // ← Rust now looks for src/greet.rs OR src/greet/mod.rs

fn main() {
    println!("{}", greet::hello());
}
```

```rust
// src/greet.rs
pub fn hello() -> &'static str {
    "hello"
}
```

The `mod greet;` in `main.rs` is the *declaration*. The file `greet.rs` is the *definition*.
Both must exist. Forget either one, and it breaks.

---

## Submodules: the mod.rs pattern

When a module has its own submodules, you use a directory with a `mod.rs` file.

```
src/
├── main.rs
└── data/
    ├── mod.rs     ← the data module itself
    ├── task.rs    ← declared as a submodule inside mod.rs
    └── entry.rs   ← declared as a submodule inside mod.rs
```

The chain works like this:

```rust
// src/main.rs
mod data;   // ← Rust looks for src/data.rs OR src/data/mod.rs
            //   it finds src/data/mod.rs
```

```rust
// src/data/mod.rs
pub mod task;    // ← Rust looks for src/data/task.rs ✓
pub mod entry;   // ← Rust looks for src/data/entry.rs ✓
pub mod store;   // ← Rust looks for src/data/store.rs ✓
```

`task.rs`, `entry.rs`, and `store.rs` define their own content, but Rust only finds them
because `mod.rs` declared them. If you add a new file `src/data/report.rs` but forget to
add `pub mod report;` to `mod.rs`, it is invisible to the entire project.

**This is the most common mistake.** When you create a new `.rs` file, always add the
`mod` declaration at the same time.

---

## The alternative: the "path" pattern (Rust 2018+)

Rust 2018 introduced an alternative where you can use a `.rs` file AND a same-named
directory together, without `mod.rs`:

```
src/
├── main.rs
├── data.rs      ← the data module
└── data/
    ├── task.rs
    └── entry.rs
```

```rust
// src/data.rs  (instead of src/data/mod.rs)
pub mod task;
pub mod entry;
pub mod store;
```

Both patterns work identically. This project uses the **`mod.rs` pattern** throughout
because it keeps all files related to a module inside one directory — easier to see at a glance.

---

## Visibility: pub, pub(crate), and private

In Go, capitalisation controls visibility. In Rust, everything is **private by default**
and you opt in to visibility with `pub`.

| Keyword | Who can see it |
|---------|---------------|
| *(nothing)* | Only the current module and its descendants |
| `pub(super)` | The parent module only |
| `pub(crate)` | Anywhere in this crate — but not external crates |
| `pub` | Anywhere, including external crates |

For this app, `pub` and nothing are the two you will use. `pub(crate)` is good practice
for things that should not be exposed if this ever becomes a library, but do not stress
about it now.

**Structs: fields are private separately from the struct itself.**

```rust
pub struct Task {
    pub id: Uuid,       // accessible from outside the module
    pub name: String,   // accessible from outside the module
    status: TaskStatus, // PRIVATE — only code inside data/task.rs can read/write this
}
```

If you forget `pub` on a field that another module needs, the compiler gives a clear error:
`field 'status' of struct 'Task' is private`. Add `pub` and it is fixed.

---

## The `use` statement and path syntax

`use` brings names into scope so you do not have to write the full path every time.

There are two kinds of paths:

| Path | Meaning |
|------|---------|
| `crate::` | Start from the crate root (src/main.rs) — absolute |
| `super::` | Start from the parent module — relative |
| `self::` | The current module — rarely needed |

**Examples for this project:**

```rust
// In src/app.rs — using things from the data module

use crate::data::task::{Task, TaskStatus};  // import two things from the same module
use crate::data::entry::TimeEntry;
use crate::data::store;                     // import the module itself (call store::load())

// Alternatively, import a specific function:
use crate::data::store::load as load_data;  // rename on import with `as`
```

```rust
// In src/data/store.rs — using things from sibling modules

use super::task::Task;    // super:: = src/data/ (the parent of store.rs)
use super::entry::TimeEntry;
```

**Rule of thumb:**
- From `app.rs` or `main.rs`: use `crate::` paths
- From inside a submodule (e.g. `data/store.rs`): use `super::` for siblings

---

## The naming conflict you will hit in this project

iced has a type called `Task` (used for async commands). Your data also has a type called
`Task`. When both are in scope, Rust cannot tell which you mean.

The fix is to alias one on import:

```rust
use crate::data::task::Task as AppTask;   // your data Task
// iced::Task is used as iced::Task<Message> directly, no import needed
```

You will see this in `app.rs`. It is not a hack — aliasing on import is idiomatic Rust.

---

## This project's full module structure

This is the target structure for the complete app. You build it incrementally — Phases 1–2
only create the shaded files.

```
tasktracker/
├── Cargo.toml
├── CLAUDE.md
├── docs/
│   └── (all your planning docs)
└── src/
    ├── main.rs              ← declares: mod app, mod data, mod ui (Phase 3+)
    │                           starts the iced application
    │
    ├── app.rs               ← declares: nothing (imports via use)
    │                           owns: App struct, Message enum, update(), view(),
    │                                 subscription()
    │
    ├── data/                ← PHASE 1: create all of these
    │   ├── mod.rs           ← declares: pub mod task, pub mod entry, pub mod store
    │   ├── task.rs          ← owns: Task struct, TaskStatus enum
    │   ├── entry.rs         ← owns: TimeEntry struct
    │   └── store.rs         ← owns: load(), save(), data_path()
    │
    └── ui/                  ← PHASE 3+: extract view functions from app.rs
        ├── mod.rs           ← declares: pub mod task_list, pub mod timer_bar,
        │                                pub mod status_bar, pub mod quick_add,
        │                                pub mod stop_prompt, pub mod review
        ├── task_list.rs     ← owns: task_list_view() function
        ├── timer_bar.rs     ← owns: timer_bar_view() function
        ├── status_bar.rs    ← owns: status_bar_view() function
        ├── quick_add.rs     ← owns: quick_add_view() function
        ├── stop_prompt.rs   ← owns: stop_prompt_view() function
        └── review.rs        ← owns: review_view() function
```

### Why ui/ does not exist in Phase 1

In Phase 1, all view functions are methods on `App` inside `app.rs`. This is intentional:
you start simple, get the app compiling and running, and only split files when a single
file becomes hard to navigate. `app.rs` will grow through Phases 2 and 3. You extract
`ui/` in Phase 3 when it is big enough to warrant it.

**How the extraction works (Phase 3 preview):**

In Phase 1–2, `app.rs` has:
```rust
impl App {
    fn timer_bar(&self) -> Element<Message> { ... }
    fn task_list(&self) -> Element<Message> { ... }
}
```

In Phase 3, you move those functions into `src/ui/timer_bar.rs`:
```rust
// src/ui/timer_bar.rs
use crate::app::{App, Message};   // imports from app.rs
use iced::Element;

pub fn view(app: &App) -> Element<Message> {
    // exact same code, but as a standalone function
}
```

Then in `app.rs`, you call it like:
```rust
use crate::ui::timer_bar;

fn view(&self) -> Element<Message> {
    column![
        timer_bar::view(self),
        // ...
    ].into()
}
```

The function signature changes from `&self` (a method) to `app: &App` (a plain function
taking a reference). The code inside is identical.

---

## Declaring the chain: what goes where

This is the most important thing to internalise. Every file must be declared *somewhere*.

| File | Declared by |
|------|-------------|
| `src/app.rs` | `mod app;` in `src/main.rs` |
| `src/data/mod.rs` | `mod data;` in `src/main.rs` |
| `src/data/task.rs` | `pub mod task;` in `src/data/mod.rs` |
| `src/data/entry.rs` | `pub mod entry;` in `src/data/mod.rs` |
| `src/data/store.rs` | `pub mod store;` in `src/data/mod.rs` |
| `src/ui/mod.rs` | `mod ui;` in `src/main.rs` (Phase 3) |
| `src/ui/task_list.rs` | `pub mod task_list;` in `src/ui/mod.rs` (Phase 3) |

When you add a new file, the question to ask is: **who is the parent of this file?**
Add `pub mod filename;` to that parent's `mod.rs` (or `main.rs` if the parent is the root).

---

## Common errors and what they mean

**`error[E0583]: file not found for module 'foo'`**

```
error[E0583]: file not found for module `foo`
  --> src/data/mod.rs:3:9
   |
 3 | pub mod foo;
   |         ^^^
   = help: to create the module `foo`, create file `src/data/foo.rs`
```

You wrote `pub mod foo;` but forgot to create the file. Create `src/data/foo.rs`.

---

**`error[E0603]: module 'task' is private`**

```
error[E0603]: module `task` is private
  --> src/app.rs:3:12
   |
 3 | use crate::data::task::Task;
   |             ^^^^
```

You wrote `mod task;` in `data/mod.rs` but forgot `pub`. Change it to `pub mod task;`.

---

**`error[E0432]: unresolved import`**

```
error[E0432]: unresolved import `crate::data::store`
  --> src/app.rs:5:5
   |
 5 | use crate::data::store;
   |     ^^^^^^^^^^^^^^^^^^
```

Either `store.rs` does not exist, or it is not declared in `data/mod.rs`.
Check both. The path in the error tells you exactly where to look.

---

**`error[E0425]: cannot find function 'load' in module 'store'`**

The module was found, but the function inside it is not `pub`. Add `pub` to `fn load()`.

---

## Adding a new file — the checklist

When you add a new `.rs` file at any point in the project, follow this:

1. Create the file
2. Find the parent module (`main.rs` for top-level, `mod.rs` for subdirectories)
3. Add `pub mod filename;` to the parent
4. Add `use` statements in the files that need to use it
5. Run `cargo build` — fix any path errors the compiler reports

---

## The CLAUDE.md for the new project

When you create the new project folder, create `CLAUDE.md` at the root with this content:

```markdown
# CLAUDE.md — tasktracker (Rust rewrite)

## Commit Scopes

| Scope | What it covers |
|-------|---------------|
| `data` | Task and TimeEntry structs, JSON persistence |
| `ui`   | iced view functions, widget layout |
| `app`  | App struct, Message enum, update(), subscription() |
| `chore`| Dependencies, project setup, Cargo.toml |

## Project-Specific Notes

- Rust + iced 0.14. Single-language — no JavaScript.
- Storage: JSON files at ~/.tasktracker/data.json (two arrays: tasks, entries)
- Quick-add is an overlay panel in the main window, not a separate window
- System tray and global hotkey are deferred — add after core app works
- Reports are plain text — no charting libraries needed
- See docs/Brainstorm-RustIcedRewrite.md for all architectural decisions
```
