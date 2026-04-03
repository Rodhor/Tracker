# Architecture — tasktracker

## Project Snapshot

- **Purpose:** Personal desktop task and time tracker. Captures work sessions during the day so you can report into TANNSS (company time system) at the end of the day without breaking focus. Runs as a native desktop app — no browser, no server. Quick-add window triggered by a global keyboard shortcut from any focused app.
- **Stack:** Go 1.22+, Fyne v2 (native desktop GUI), sqlx (DB ergonomics), modernc.org/sqlite (pure-Go SQLite, no CGO), golang.design/x/hotkey (global keyboard shortcuts)
- **Entry point:** `cmd/main.go`
- **Runs as:** Native desktop window. Minimises to system tray on close. Data stored in `~/.tasktracker.db`
- **Target platforms:** Linux, Windows, macOS

> ⚠ Needs update: previously used templ + htmx + Go HTTP server. Replaced with Fyne desktop UI — see `docs/Brainstorm-FyneUI.md` for the decision record.

---

## Directory Map

```
tasktracker/
├── cmd/
│   └── main.go                  # owns: process entry, hotkey registration, wires everything
│                                # does not own: business logic, SQL
├── internal/
│   ├── db/
│   │   └── db.go                # owns: DB connection, migrations, shared pool
│   │                            # does not own: domain queries
│   ├── tasks/
│   │   ├── model.go             # owns: Task struct, TaskStatus, Eisenhower fields
│   │   └── store.go             # owns: all SQL queries for tasks
│   ├── timer/
│   │   ├── model.go             # owns: TimeEntry struct (includes notes field)
│   │   └── store.go             # owns: all SQL queries for time entries
│   └── ui/
│       ├── app.go               # owns: Fyne app + main window setup, wires stores to UI
│       │                        # does not own: individual widget layout or SQL
│       ├── time_bar.go          # owns: timer bar widget (top panel, always visible)
│       ├── task_list.go         # owns: task list widget, Eisenhower sorting, row layout
│       ├── status_bar.go        # owns: bottom bar — today's total time, New Task button
│       ├── quick_add.go         # owns: quick-add window — filter/create task, start timer
│       ├── review.go            # owns: review window — daily log, gap detection, notes editing
│       └── theme.go             # owns: custom Fyne theme — spacing, padding, colour, button hierarchy
├── docs/
│   ├── Architecture.md
│   ├── features/
│   └── plans/
├── go.mod
└── CLAUDE.md
```

---

## Core Patterns

### App struct with injected stores

- **What it is:** An `App` struct in `ui/app.go` holds references to both stores and the Fyne app/window. All UI setup functions receive what they need via parameters — no global state.
- **Why it is used here:** Same dependency injection principle as the previous handler pattern — everything is traceable from `main.go`.
- **Where to see it:** `internal/ui/app.go`
- **Rule:** `app.go` creates the struct and wires everything. Individual files in `ui/` define their own widgets and receive stores via function parameters.

### Binding-driven live updates

- **What it is:** The live timer display uses `binding.NewString()`. A goroutine calls `binding.Set()` every second. The label widget updates automatically.
- **Why it is used here:** Fyne's UI is not thread-safe. Bindings are the correct way to update labels from goroutines — no manual refresh calls needed.
- **Where to see it:** `internal/ui/timer_bar.go`
- **Rule:** Never call `label.SetText()` from a goroutine. Use a binding instead.

### Shared refresh function

- **What it is:** A `refresh()` function created in `app.go` and passed to each widget. When any widget changes data (start timer, stop timer, add task), it calls `refresh()` to reload and redraw all affected panels.
- **Why it is used here:** Simple and explicit at this scale. Avoids a more complex event bus or observer pattern.
- **Rule:** Any action that changes store state must call `refresh()` afterwards.

### Stores own all SQL

- **What it is:** Each domain (`tasks`, `timer`) has a `Store` struct with methods for every database operation.
- **Why it is used here:** Keeps SQL in one place per domain. Handlers never write queries.
- **Rule:** If a function needs to query the database, it belongs on a Store method.

### Dependency injection via function parameters

- **What it is:** The database pool is created once in `main.go` and passed to store constructors. The `Handler` receives the stores.
- **Why it is used here:** No global state. Everything is traceable from `main.go`.
- **Rule:** No package-level `var db *sqlx.DB`. Always pass the pool explicitly.

---

## Data Flow

```
App starts
  → db.Init() opens ~/.tasktracker.db, runs migrations
  → stores created, ui.Run() called
  → Fyne window opens, tasks loaded via taskStore.ListRecent(3)
  → timer bar polls timerStore.Active() every second via goroutine + binding

User clicks ▶ on a task row
  → timerStore.Start(taskID) called
  → refresh() reloads task list and timer bar
  → timer bar goroutine begins showing elapsed time

User clicks ■ in timer bar
  → timerStore.Stop() called
  → taskStore.UpdateStatus(taskID, todo) called
  → refresh() reloads both panels

User presses Ctrl+Shift+Space (any app focused)
  → golang.design/x/hotkey delivers keydown event to goroutine
  → quickAddWindow.Show() called
  → user types task name, selects mode (capture / timer / both), presses Enter
  → taskStore.Add() and/or timerStore.Start() called
  → quick-add window hides, refresh() updates main window

User closes main window
  → SetCloseIntercept hides window instead of quitting
  → app continues running, timer keeps ticking in background
  → system tray menu available for reopen / stop timer / quit
```

---

## Database Schema

```sql
CREATE TABLE IF NOT EXISTS tasks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'todo',  -- 'todo', 'in_progress', 'done'
    created_at  TEXT NOT NULL,
    urgent      INTEGER NOT NULL DEFAULT 0,    -- Eisenhower axis
    important   INTEGER NOT NULL DEFAULT 0,    -- Eisenhower axis
    description TEXT                           -- what the task is (optional)
);

CREATE TABLE IF NOT EXISTS time_entries (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id    INTEGER NOT NULL REFERENCES tasks(id),
    started_at TEXT NOT NULL,
    ended_at   TEXT,          -- NULL while timer is running
    minutes    INTEGER,       -- NULL until stopped
    notes      TEXT           -- what was done during this block (optional)
);
```

> ⚠ `urgent`, `important`, `description` added via migration 3. `notes` added via migration 4. See `internal/db/db.go` for migration history.

---

## Adding a New Feature

1. If the feature needs new data: add a migration in `internal/db/db.go` (append — never edit existing)
2. If it introduces a new domain: create `internal/[domain]/model.go` and `internal/[domain]/store.go`
3. If it extends an existing domain: add a method to the existing Store
4. Update the model struct if new columns were added
5. Add a new file in `internal/ui/` for the new widget or window
6. Wire it into `app.go` — pass stores and the shared `refresh()` function
7. Update the Directory Map in this file

---

## Modifying an Existing Feature

1. Read the relevant `model.go` and `store.go` first
2. Find the handler method and templ component that use the changed data
3. Schema changes: append a new migration, never modify existing ones
4. After editing any `.templ` file: run `templ generate` before building
5. Test by running the server and loading the affected page

---

## Naming Conventions

| Thing | Convention | Example |
|-------|-----------|---------|
| Files | `snake_case` | `store.go`, `task_list.templ` |
| Packages | lowercase, one word | `tasks`, `timer`, `handler`, `templates` |
| Types/structs | `PascalCase` | `Task`, `TimeEntry`, `Handler` |
| Exported functions/methods | `PascalCase` | `NewHandler`, `Routes` |
| Unexported functions/methods | `camelCase` | `renderPage`, `elapsed` |
| HTTP routes | `kebab-case` | `/timer/start`, `/quick-add` |
| Templ components | `PascalCase` | `Dashboard(...)`, `TaskList(...)` |
| DB columns | `snake_case` | `started_at`, `task_id` |

---

## Constraints

- **No CGO** — use `modernc.org/sqlite`. Still applies.
- **No SQL outside Store methods** — still applies.
- **No UI code outside `internal/ui/`** — `cmd/main.go` only calls `ui.Run()`. No Fyne imports in `main.go` beyond that.
- **No store calls outside Store methods** — UI code calls store methods; it never builds SQL queries.
- **`internal/` is intentional** — packages under `internal/` cannot be imported outside this module. Still applies.
- **Schema changes are append-only** — add a new migration step, never modify existing ones.
- **Bindings for goroutine-to-UI updates** — never call widget methods from a goroutine. Use `binding` types for live data.
- **Wayland hotkey note** — `golang.design/x/hotkey` uses X11 APIs. On Wayland without XWayland, hotkey registration will fail. Log the error, do not crash — the app is fully usable without the global shortcut.
