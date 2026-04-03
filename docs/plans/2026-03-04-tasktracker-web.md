# tasktracker — Implementation Plan (Web Stack)

**Supersedes:** `docs/plans/2026-03-03-tasktracker.md`
**Date:** 2026-03-04
**Architecture reference:** `docs/Architecture.md`
**Stack decision:** `docs/Brainstorm-WebStack.md`

---

## How to use this plan

Each task explains what to build and what Go or web concept it introduces. You write the code. Share it when done and it will be reviewed before moving to the next task.

Say **"done with task N"** or paste your code to trigger a review.

---

## Task 1: Initialise the module and folder structure

**What you're building:** The corrected `go.mod` and the empty folder skeleton.

**Concept: Go modules and packages**

In Go, a **module** is identified by the name in `go.mod`. That name becomes the import prefix for every package in the project. If the module is `tasktracker`, then a file in `internal/tasks/` is imported as `tasktracker/internal/tasks`. The name in `go.mod` and the directory name on disk are independent — the folder can be called anything.

`internal/` is enforced by the Go toolchain: packages inside it can only be imported by code within the same module. It marks packages as implementation details, not public API.

**What to do:**

1. Edit `go.mod` — change `module track` to `module tasktracker`
2. Create the folder structure from `docs/Architecture.md`:
   - `cmd/`
   - `internal/db/`
   - `internal/tasks/`
   - `internal/timer/`
   - `internal/handler/`
   - `internal/templates/`
   - Add a `.gitkeep` file to each empty folder (Git does not track empty directories)
3. Create `cmd/main.go` with `package main` and an empty `func main() {}`
4. Run `go build ./...` to verify it compiles

**What to look up:**
- What `go.sum` is and why it exists (you never edit it manually)
- Why Go uses `internal/` instead of just relying on naming conventions

**Commit:**
```
git add .
git commit -m "chore: initialise module and project structure"
```

---

## Task 2: Add dependencies

**What you're building:** The `go.mod` entries for the full dependency set.

**New concept: the templ CLI**

Unlike most Go libraries, templ has two parts:

1. The **runtime package** — imported in your Go code like any other dependency
2. The **code generation tool** — a CLI (`templ generate`) that compiles `.templ` files into regular `.go` files

The generated `.go` files are what the Go compiler actually sees. You run `templ generate` every time you edit a `.templ` file. Think of it as a mandatory build step, similar to how `protoc` works with Protocol Buffers.

Install the CLI first (this puts it in your `$GOPATH/bin`):
```bash
go install github.com/a-h/templ/cmd/templ@latest
```

Then add runtime dependencies:
```bash
go get github.com/a-h/templ
go get github.com/jmoiron/sqlx
go get modernc.org/sqlite
go get golang.design/x/hotkey
```

htmx and Tailwind are loaded from CDN in the HTML — no Go dependency.

**What to look up:**
- `go install` vs `go get` — what is the difference and when do you use each?
- Why templ generates `.go` files rather than interpreting `.templ` files at runtime — what does this give you that a runtime interpreter does not?

**Commit:**
```
git add go.mod go.sum
git commit -m "chore: add templ, sqlx, sqlite, and hotkey dependencies"
```

---

## Task 3: Database connection and migrations — `internal/db/db.go`

**What you're building:** A function that opens the SQLite database, creates it if needed, and runs the schema.

**New concept: `database/sql`, sqlx, and driver registration**

Go's standard library provides `database/sql` — a generic SQL interface. The actual driver (SQLite here) registers itself when its package is blank-imported:

```go
import _ "modernc.org/sqlite"
// The _ means: run this package's init() but don't use its exports directly.
// init() registers the driver under the name "sqlite" with database/sql.
```

After that, `sqlx.Open("sqlite", path)` knows what to do. `sqlx.Open` is identical to `sql.Open` but returns a `*sqlx.DB` with additional methods you'll use in Tasks 4 and 5. Always call `.Ping()` after opening — `Open` itself doesn't actually connect.

Go functions signal errors as a second return value, not exceptions:
```go
value, err := someFunction()
if err != nil {
    return nil, err  // propagate upward — the caller decides what to do
}
```

**What to do:**

1. Create `internal/db/db.go`
2. Export a function: `func Open() (*sqlx.DB, error)`
3. Inside it: build path to `~/.tasktracker.db` via `os.UserHomeDir()`, open with sqlx, ping, run `CREATE TABLE IF NOT EXISTS` for both tables (schema in `docs/Architecture.md`), return the pool
4. Blank-import the SQLite driver at the top of the file

**What to look up:**
- `os.UserHomeDir()` — what does it return and when does it error?
- `db.MustExec` vs `db.Exec` — when would you choose each?

**Verify:** Temporarily call `db.Open()` in `cmd/main.go`, print "connected". Run it and confirm `~/.tasktracker.db` was created on disk.

**Commit:**
```
git add internal/db/ cmd/main.go
git commit -m "feat(db): connect to SQLite and create schema"
```

---

## Task 4: Task model and store — `internal/tasks/`

**What you're building:** The `Task` struct and a `Store` with methods to add and list tasks.

**New concept: structs, methods, and receivers**

Go has no classes. Instead, you define a `struct` for data and attach **methods** to it. A method is a function with a **receiver** — the type it belongs to:

```go
// Regular function — no receiver:
func Add(name string) error { ... }

// Method on Store — has a receiver:
func (s *Store) Add(name string) error { ... }
//    ^^^^^^^^ s is a pointer to Store — changes to s are visible to the caller
```

Use `*Store` (pointer receiver) for methods that hold or modify state. This is the standard Go pattern.

**New concept: struct tags and sqlx scanning**

sqlx maps database columns to struct fields using struct tags — annotations in backticks after a field:

```go
type Task struct {
    ID   int64  `db:"id"`
    Name string `db:"name"`
}
// The db:"..." tag tells sqlx which column maps to which field.
// Column names must match exactly what the SQL query returns.
```

With tags in place, use:
- `db.Select(&slice, query, args...)` — fills a slice with all matching rows
- `db.Exec(query, args...)` — runs INSERT / UPDATE / DELETE

**What to build:**

`internal/tasks/model.go`:
- `Task` struct: `ID` (int64), `Name` (string), `Status` (string), `CreatedAt` (string)
- `db:"..."` tags on every field matching the schema column names

`internal/tasks/store.go`:
- `Store` struct holding `*sqlx.DB`
- `NewStore(db *sqlx.DB) *Store`
- `(s *Store) Add(name string) error` — inserts with status `"todo"` and current time
- `(s *Store) All() ([]Task, error)` — returns all tasks ordered by `created_at`

**What to look up:**
- `time.Now().UTC().Format(time.RFC3339)` — current time as a string
- What happens if you forget a `db:` tag — does sqlx error or silently skip it?

**Verify:** In `cmd/main.go`, call `store.Add("my first task")` then `store.All()` and print the result.

**Commit:**
```
git add internal/tasks/
git commit -m "feat(tasks): add Task model and store with Add and All"
```

---

## Task 5: Timer model and store — `internal/timer/`

**What you're building:** The `TimeEntry` struct and a `Store` with Start, Stop, and Active.

**New concept: pointer fields and nullable columns**

In Go, a pointer to a value (`*string`, `*int64`) can be `nil` — meaning "no value present". This is how you represent nullable database columns. `ended_at` and `minutes` are NULL while a timer is running, so they map to `*string` and `*int64`:

```go
type TimeEntry struct {
    EndedAt *string `db:"ended_at"`
    // nil  → column is NULL (timer still running)
    // &str → column has a value (timer stopped)
}
```

**What to build:**

`internal/timer/model.go`:
- `TimeEntry` struct: `ID` (int64), `TaskID` (int64), `StartedAt` (string), `EndedAt` (*string), `Minutes` (*int64)
- `db:"..."` tags on every field

`internal/timer/store.go`:
- `Store` struct, `NewStore`
- `(s *Store) Start(taskID int64) error` — call `Active()` first; return error if a timer is already running; otherwise insert a new row
- `(s *Store) Stop() error` — get the active entry, calculate elapsed minutes, update the row with `ended_at` and `minutes`
- `(s *Store) Active() (*TimeEntry, error)` — use `db.Get` to find the row where `ended_at IS NULL`; if `sql.ErrNoRows`, return `nil, nil` (no timer running is not an error)

**What to look up:**
- `db.Get(&dest, query, args...)` — for SELECT returning exactly one row
- `errors.Is(err, sql.ErrNoRows)` — how to check for the "nothing found" case
- `time.Parse(time.RFC3339, s)` — parse a stored time string back into `time.Time`
- `time.Since(t).Minutes()` — elapsed time as a float; cast to `int64` for storage

**Verify:** Start a timer, call `Active()` and print it. Stop it, call `Active()` again and confirm it returns nil.

**Commit:**
```
git add internal/timer/
git commit -m "feat(timer): add TimeEntry model and store with Start, Stop, Active"
```

---

## Task 6: HTTP server skeleton — `internal/handler/handler.go` + `cmd/main.go`

**What you're building:** A Go HTTP server that listens on localhost and responds to requests.

**New concept: `net/http` and the handler function signature**

Go's standard library includes a complete HTTP server. The core types are:

- `http.ServeMux` — the router; maps URL patterns to handler functions
- `http.ResponseWriter` — where you write the response (status, headers, body)
- `http.Request` — the incoming request (URL, headers, form data)

Every handler has the same signature:
```go
func(w http.ResponseWriter, r *http.Request)
// w — write your response here
// r — read the request here
```

Register handlers with the mux, then start the server:
```go
mux := http.NewServeMux()
mux.HandleFunc("GET /", h.HandleDashboard)
mux.HandleFunc("POST /tasks", h.HandleTasks)

http.ListenAndServe("127.0.0.1:8080", mux)
// This blocks — put it at the end of main, or in a goroutine if other work follows
```

**What to build:**

`internal/handler/handler.go`:
- `Handler` struct holding `*tasks.Store` and `*timer.Store`
- `NewHandler(tasks *tasks.Store, timer *timer.Store) *Handler`
- `Routes() *http.ServeMux` — registers placeholder routes and returns the mux:
  - `GET /` → writes `"dashboard coming soon"`
  - `GET /quickadd` → writes `"quick add coming soon"`

`cmd/main.go`:
- Open DB, create stores, create handler, call `h.Routes()`
- Start server: `http.ListenAndServe("127.0.0.1:8080", mux)`

**What to look up:**
- `http.Handle` vs `http.HandleFunc` — what is the difference?
- Go 1.22 method-and-path routing syntax: `"GET /path"` in `HandleFunc` — look this up, it changed in 1.22

**Verify:** Run the server. Open `http://127.0.0.1:8080` in a browser — you should see "dashboard coming soon".

**Commit:**
```
git add internal/handler/handler.go cmd/main.go
git commit -m "feat(ui): add HTTP server skeleton with placeholder routes"
```

---

## Task 7: Base layout and dashboard shell — `internal/templates/`

**What you're building:** Your first templ components — the HTML shell with Tailwind + htmx, and a dashboard page that uses it.

**New concept: templ components**

A `.templ` file defines components using Go-like syntax with HTML inside:

```go
package templates

templ Layout(title string) {
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <title>{ title }</title>
        // { } renders a Go expression — templ escapes it automatically
    </head>
    <body>
        { children... }
        // children... is the slot — the caller's content goes here
    </body>
    </html>
}
```

After running `templ generate`, this becomes a regular Go function you call from a handler:
```go
templates.Layout("tasktracker").Render(r.Context(), w)
```

Components can nest with `@`:
```go
templ Dashboard() {
    @Layout("tasktracker") {
        <h1>Tasks</h1>
    }
}
```

**New concept: Tailwind CDN**

Add these two tags to `<head>` in your layout:
```html
<script src="https://cdn.tailwindcss.com"></script>
<script src="https://unpkg.com/htmx.org@2.0.3"></script>
```

Every Tailwind utility class becomes available immediately — no build step, no config file. Apply classes directly on HTML elements: `class="bg-gray-950 text-white p-4"`.

**What to do:**

1. Create `internal/templates/layout.templ` — base shell with `<head>`, CDN tags, a `{ children... }` slot, and Tailwind classes on `<body>`
2. Create `internal/templates/dashboard.templ` — a `Dashboard()` component using `@Layout` with a placeholder heading
3. Run `templ generate` — confirm `layout_templ.go` and `dashboard_templ.go` appear
4. Update `handler/dashboard.go` to call `templates.Dashboard().Render(r.Context(), w)` instead of writing a string
5. Run `go build ./...` — must compile

**What to look up:**
- templ documentation — component syntax, `{ children... }`, calling with `@`
- Tailwind utility classes to start with: `min-h-screen`, `bg-gray-950`, `text-white`, `p-8`, `text-2xl`, `font-bold`

**Verify:** Reload `http://127.0.0.1:8080` — a styled page with a heading should appear.

**Commit:**
```
git add internal/templates/ internal/handler/dashboard.go
git commit -m "feat(ui): add templ layout with Tailwind and dashboard shell"
```

---

## Task 8: Task list with htmx — `internal/templates/tasklist.templ` + `handler/tasks.go`

**What you're building:** Tasks fetched from the database and displayed, with an add-task form that submits without a page reload.

**New concept: htmx attributes**

htmx works by adding attributes to plain HTML elements. When the user interacts with them, htmx makes an AJAX request and swaps part of the page with the response:

```html
<form hx-post="/tasks" hx-target="#task-list" hx-swap="outerHTML">
    <input name="name" placeholder="New task..." />
    <button type="submit">Add</button>
</form>
```

- `hx-post="/tasks"` — POST to this path on submit
- `hx-target="#task-list"` — put the response into this element
- `hx-swap="outerHTML"` — replace the entire target element (not just its inner HTML)

The server returns only the updated fragment — not the full page. htmx detects this automatically using the `HX-Request: true` header it adds to every request it makes.

**What to build:**

`internal/templates/tasklist.templ`:
- `TaskList(tasks []tasks.Task)` component — a `<ul id="task-list">` with one `<li>` per task
- Each `<li>` shows task name and status
- Include the add-task `<form>` with htmx attributes at the top or bottom

`internal/handler/tasks.go`:
- `(h *Handler) HandleTasks(w http.ResponseWriter, r *http.Request)`
- On `POST`: read `r.FormValue("name")`, call `h.tasks.Add(name)`, fetch all tasks, return `templates.TaskList(tasks).Render(r.Context(), w)`
- Register route in `handler.go`: `mux.HandleFunc("POST /tasks", h.HandleTasks)`

Update `dashboard.templ` to include `@TaskList(tasks)` — pass tasks into the `Dashboard` component from the handler.

**What to look up:**
- `r.FormValue("name")` — reading a posted form field
- The `HX-Request` header — check it with `r.Header.Get("HX-Request") == "true"` if you want to return a fragment vs full page
- Tailwind for a list: `space-y-2`, `rounded`, `p-3`, `bg-gray-800`, `flex`, `justify-between`

**Verify:** Open the dashboard, add a task — the list updates without a full page reload.

**Commit:**
```
git add internal/templates/ internal/handler/
git commit -m "feat(ui): task list and add form with htmx partial swap"
```

---

## Task 9: Live timer with SSE — `handler/timer.go` + templates

**What you're building:** Start/Stop timer controls and an elapsed-time display that updates every second.

**New concept: Server-Sent Events (SSE)**

SSE is a simple protocol for one-way streaming from server to client over a regular HTTP connection. The server:

1. Sets the response header `Content-Type: text/event-stream`
2. Keeps the connection open
3. Writes `data: <value>\n\n` for each event (the double newline is required)
4. Calls `Flush()` after each write to push the data immediately

On the client side, htmx's SSE extension handles the connection automatically:

```html
<!-- load the SSE extension (add to layout <head>) -->
<script src="https://unpkg.com/htmx-ext-sse@2.2.2/sse.js"></script>

<!-- connect and update a target element on each event -->
<div hx-ext="sse" sse-connect="/timer/events">
    <span sse-swap="tick">No active timer</span>
</div>
```

When an event named `tick` arrives, htmx replaces the `<span>` content. No JS required.

**New concept: context cancellation**

An SSE handler keeps the connection open until the client disconnects. `r.Context()` is cancelled when the client closes the tab or navigates away. Use a `select` to handle both the ticker and the disconnect:

```go
flusher := w.(http.Flusher)  // cast to get Flush()
ticker := time.NewTicker(time.Second)
defer ticker.Stop()

for {
    select {
    case <-ticker.C:
        fmt.Fprintf(w, "event: tick\ndata: %s\n\n", elapsed())
        flusher.Flush()
    case <-r.Context().Done():
        return  // client disconnected — stop the goroutine
    }
}
```

**What to build:**

`internal/handler/timer.go`:
- `HandleTimerStart(w, r)` — reads task ID from the form, calls `h.timer.Start(taskID)`, returns an updated timer panel fragment
- `HandleTimerStop(w, r)` — calls `h.timer.Stop()`, returns an updated timer panel fragment
- `HandleTimerEvents(w, r)` — sets SSE headers, ticks every second, sends elapsed time as a `tick` event, returns on context done

`internal/templates/dashboard.templ`:
- Add a timer panel: Start/Stop buttons with htmx POST, the SSE-connected `<div>` with the `<span sse-swap="tick">`

**What to look up:**
- `http.Flusher` — why you need to cast `w` to get `Flush()`
- SSE event format exactly: `event: name\ndata: value\n\n` (two newlines at the end)
- htmx SSE extension documentation

**Verify:** Select a task, click Start — elapsed time ticks up each second. Click Stop — it resets.

**Commit:**
```
git add internal/handler/timer.go internal/templates/
git commit -m "feat(ui): live timer with SSE and htmx"
```

---

## Task 10: Global hotkey — `cmd/main.go`

**What you're building:** A goroutine that registers an OS-level keyboard shortcut and opens the quick-add popup when triggered.

**New concept: `golang.design/x/hotkey`**

This library lets a Go process register a global hotkey — one the OS delivers even when the app has no focused window:

```go
hk := hotkey.New([]hotkey.Modifier{hotkey.ModCtrl, hotkey.ModShift}, hotkey.KeySpace)
if err := hk.Register(); err != nil {
    log.Println("hotkey failed to register:", err)
    return
}
defer hk.Unregister()

for range hk.Keydown() {
    openQuickAdd()  // called every time the shortcut is pressed
}
```

`Keydown()` returns a channel. Ranging over it blocks until the hotkey is pressed, then your function runs. This must run in a goroutine so it doesn't block the HTTP server.

**New concept: `os/exec` and cross-platform commands**

To open a browser window in `--app` mode (no address bar, no tabs):

```go
// Linux — try chrome, fall back to chromium
exec.Command("google-chrome", "--app=http://127.0.0.1:8080/quickadd", "--window-size=420,100").Start()

// Windows
exec.Command("cmd", "/c", "start", "chrome", "--app=http://127.0.0.1:8080/quickadd").Start()

// macOS
exec.Command("open", "-a", "Google Chrome", "--args", "--app=http://127.0.0.1:8080/quickadd").Start()
```

Use `runtime.GOOS` to detect the platform: it returns `"linux"`, `"windows"`, or `"darwin"`.

**What to build:**

In `cmd/main.go`:
- Start the HTTP server in a goroutine (so `main` doesn't block there)
- Register the hotkey — `Ctrl+Shift+Space` is a reasonable default
- Start a goroutine that ranges over `hk.Keydown()` and calls an `openQuickAdd()` function
- `openQuickAdd()` switches on `runtime.GOOS` and runs the appropriate command

**What to look up:**
- `golang.design/x/hotkey` — `Modifier` constants, `Key` constants
- `exec.Command(...).Start()` vs `.Run()` — what is the difference?
- `runtime.GOOS` — the possible values

**Verify:** Run the app. Press the hotkey while another window is focused — a small borderless browser window should open.

**Commit:**
```
git add cmd/main.go
git commit -m "feat(tray): register global hotkey and open quickadd popup"
```

---

## Task 11: Quick-add window — `handler/quickadd.go` + `templates/quickadd.templ`

**What you're building:** The quick-add page — a small focused form that saves a task and closes the window.

**New concept: closing a browser window with `window.close()`**

After a successful POST, the server needs to trigger the window to close. The simplest approach: respond with a tiny HTML snippet containing a script tag:

```html
<script>window.close()</script>
```

When htmx swaps this into the page, the browser executes it. `window.close()` works for windows opened via `--app` mode or `window.open()`. It does not work for tabs the user opened themselves — which is fine here since the hotkey always opens in `--app` mode.

**New concept: `autofocus`**

The `autofocus` HTML attribute tells the browser to focus an input as soon as the page loads. In a quick-add popup, this means the user can start typing immediately without clicking:

```html
<input autofocus name="name" placeholder="Task name..." />
```

**What to build:**

`internal/templates/quickadd.templ`:
- A standalone HTML page — do not use `@Layout`. It needs its own minimal `<head>` with Tailwind CDN.
- A single `<input autofocus>` inside a centred form
- `hx-post="/tasks/quick"` on the form, `hx-swap="outerHTML"` targeting the form itself
- Use Tailwind to make it visually compact and centred: `h-screen flex items-center justify-center`

`internal/handler/quickadd.go`:
- `HandleQuickAdd(w, r)` — on `GET`, render `templates.QuickAdd()`
- `HandleQuickAddSubmit(w, r)` — on `POST`: call `h.tasks.Add(r.FormValue("name"))`, write `<script>window.close()</script>`
- Register both routes in `handler.go`

**What to look up:**
- `autofocus` attribute — does it work when the element is already in the DOM, or only on page load?
- Tailwind `h-screen`, `flex`, `items-center`, `justify-center` — how do these combine to centre content?

**Verify:** Press the hotkey, type a task name, press Enter — window closes, task appears in the dashboard when you reload or add the next task.

**Commit:**
```
git add internal/handler/quickadd.go internal/templates/quickadd.templ internal/handler/handler.go
git commit -m "feat(tray): quick add window with auto-close on submit"
```

---

## Done

Core app complete. Concepts introduced per task:

| Task | New concept |
|------|-------------|
| 1–2 | Go modules, packages, `internal/`, templ CLI, `go install` |
| 3 | `database/sql`, sqlx, driver registration, error handling |
| 4 | Structs, struct tags, sqlx `Select`/`Exec`, pointer receivers |
| 5 | Pointer fields, nullable columns, `sql.ErrNoRows` |
| 6 | `net/http`, `ServeMux`, handler function signature |
| 7 | templ components, `templ generate`, Tailwind CDN |
| 8 | htmx attributes, partial swap, `HX-Request` header |
| 9 | SSE, `http.Flusher`, context cancellation, htmx SSE extension |
| 10 | `golang.design/x/hotkey`, `os/exec`, `runtime.GOOS` |
| 11 | `autofocus`, `window.close()`, htmx response fragments |

**Natural next steps:**
- Task status cycling (todo → in progress → done) — click a status badge to toggle
- Weekly summary view showing total time per task
- Task deletion with a confirmation dialog
- Port configuration via a command-line flag
- Tailwind CLI build step for production (removes unused classes)
- PWA manifest so the dashboard can be installed as a desktop app
