# Brainstorm — What's missing before the app is truly useful

**Date:** 2026-04-05
**Context:** Phases 1–6 are implemented. Core loop works: track → review → report to TANNSS.
This session maps everything still missing or deferred, then prioritises ruthlessly.

---

## What the original plan said was still coming

The `Brainstorm-RustIcedRewrite.md` planned these explicitly and they were never built:

| Planned feature                                          | Status              |
|----------------------------------------------------------|---------------------|
| Quick-add filter panel (search existing tasks by typing) | Not built           |
| Code split into `src/ui/` module files                   | Not built           |
| System tray                                              | Explicitly deferred |
| Global hotkey                                            | Explicitly deferred |

---

## What's missing by category

### A — Code structure split

**Everything is in `src/app.rs` (~1000 lines).** The original plan called for:

```
src/
├── main.rs
├── app.rs          # App struct, Message, update(), subscription() only
├── data/
│   ├── mod.rs
│   ├── task.rs
│   ├── entry.rs
│   └── store.rs
└── ui/
    ├── mod.rs
    ├── timer_bar.rs
    ├── task_list.rs
    ├── status_bar.rs
    ├── stop_prompt.rs
    └── review.rs
```

`app.rs` becomes a coordinator only — it owns state and handles messages, but delegates
all rendering to functions in `src/ui/`. Each ui file receives `&App` and returns an
`Element<Message>`. No state lives in view functions.

**Why now, before more features:** Each new feature adds another 100–200 lines to `app.rs`.
At 1500+ lines the file becomes hard to navigate. The split is mechanical
(move functions, fix imports) and pays off immediately and permanently.

---

### B — Timezone

**The problem:** Times are stored in UTC and displayed in UTC. If you're in CET (UTC+1),
a session that started at 10:00 local shows as 09:00 in the review. TANNSS expects local
time.

**Fix:** One change in `format_hhmm()` — switch from `.with_timezone(&Utc)` to
`.with_timezone(&chrono::Local)`. Storage stays in UTC. Only the display layer changes.

---

### C — Live notes during a running timer

**The problem:** Notes are only captured at stop time. If you notice something mid-session
("fixed the edge case with empty inputs", "blocked by missing API key") you either keep
it in your head until you stop, or you stop the timer just to write it down.

**What's needed:** A modal overlay — same pattern as the stop prompt, triggered by a "Note"
button in the timer bar (and later a hotkey). A single text input pre-filled with whatever
notes already exist on the active entry. Saving writes immediately to `active_entry.notes`
and syncs to `self.entries`. Closing without saving discards changes.

**Behaviour:**

- Timer bar shows a "Note" button (with a small indicator if notes already exist)
- Clicking opens a full modal overlay with a text area pre-filled from `active_entry.notes`
- Save writes to the active entry immediately; the timer keeps running
- When the stop prompt opens later, its note field is pre-filled from `active_entry.notes`
- If you don't write any live notes, the stop prompt note field starts empty as before

**Why modal instead of inline:** Consistent with how all other transient actions work in
this app (stop prompt, delete confirmation). Also makes hotkey wiring natural — one key
opens it, Escape closes it, regardless of where focus was.

---

### D — Stop prompt redesign

**The problem:** The three status buttons (To Do / In Progress / Done) all look identical.
There is no visual indication of which is currently selected — you have to read the
"Mark as: X" text below them. The layout has too many rows to parse quickly.

**What's needed:** Replace the three buttons with a `pick_list` (dropdown). This is iced's
`widget::pick_list` — a compact control that shows the current selection and opens a list
on click. Much less visual noise, clearly communicates the selected state.

**New layout:**

```
Stopping: [Task name]

[Note text input — pre-filled from live notes if any]

Status: [pick_list: To Do ▾]

[Stop & Save]  [Cancel]
```

Four elements instead of six. The status line reads naturally ("Status: Done") rather than
requiring you to decode which of three identical buttons is active.

**`pick_list` requires `TaskStatus` to implement `Display` and the list needs to be a
`Vec<TaskStatus>` or static slice.** Both are small additions.

---

### E — Quick-add filter

**The current friction:** To start working on a task you scan the full list and click ▶.
If the task doesn't exist yet, you type it in the status bar, press Enter, then click ▶.
Two interactions. With 15+ tasks, scanning takes real time.

**What's needed:** A modal overlay with a single text field. Typing filters existing tasks
by name as you type — like autocomplete in an IDE. You see a short list of matches, click
or press arrow keys to select one, press Enter to start its timer. If nothing matches,
Enter creates a new task and starts it immediately. One gesture, always.

**Scope (lean version):**

- Triggered by a button in the status bar (in-app hotkey wired in a later phase)
- Modal overlay using `stack` — same pattern as stop prompt
- Text input with a live-filtered list of matching tasks below it
- Arrow keys + Enter for keyboard navigation
- Create + start if no match
- Escape to close without action

**Why modal:** Makes hotkey wiring trivial later — press a key, modal appears, Escape
or Enter closes it. Consistent with all other transient actions in the app.

---

### F — Per-entry copy button (review)

**The problem:** After reviewing your entries, you copy each task's description into TANNSS
manually. The app has the note — you just need to get it to the clipboard quickly.

**What's needed:** A small "Copy" button on each entry row in the review. Pressing it puts
that entry's note text on the system clipboard. You switch to TANNSS, paste, done.

**Scope:** One button per row. Clipboard write via the `arboard` crate (cross-platform,
widely used). No formatting, no full export — just the note text.

---

### G — Entry time editing

**The problem:** You forget to stop the timer. It runs for 3 hours instead of 45 minutes.
Currently you must delete the entry and lose the note. You cannot correct the times.

**What's needed:** In the review, each entry row gets an "Edit times" mode. Shows two
HH:MM text inputs (start and end time). Saving recomputes `minutes = (end - start) -
total_paused`. Validates that end is after start.

**Scope:** HH:MM input only. The entry's date stays fixed. Recompute `minutes` on save.

---

### H — UI Polish

**What's needed (lean):**

- Visual separator between timer bar, task list, and status bar
- Primary vs. secondary button distinction (Stop/Confirm vs. Cancel/Edit)
- Active task name in timer bar more prominent
- "Needs Sorting" header visually muted compared to "TODOS"

Not in scope: custom colour theme, icons, window chrome.

---

### I — In-app keyboard shortcuts

**Why this is separate from global hotkeys:** In-app shortcuts work entirely within iced
using `iced::keyboard::on_key_press` as a subscription. No platform crates, no X11/Wayland
concerns. They are immediately useful and fast to implement.

**Useful shortcuts:**

- `Q` — open quick-add modal
- `N` — open quick-note modal (only active when a timer is running)
- `P` — pause / resume active timer
- `S` — open stop prompt (only when timer is running)
- `Escape` — close any open modal (stop prompt, quick-add, quick-note)
- `R` — open / close review

`Escape` to close modals is the highest-value shortcut — it makes every modal feel
keyboard-native without needing to reach for buttons.

**How iced handles this:** Add a second `Subscription` alongside the tick timer.
`iced::keyboard::on_key_press` emits a message when a key is pressed. The `update()`
arm maps the key to the appropriate existing message (e.g., `Key::Named(Named::Escape)`
→ `Message::CancelStop` or `Message::CloseQuickAdd` depending on what's open).

---

### J — System tray (deferred)

Requires the `tray-icon` crate. Add after in-app shortcuts are solid.

---

### K — Global hotkey (deferred)

Requires `global-hotkey` crate. Wayland support uncertain. Add after system tray works.

---

## Prioritised plan

| Phase        | Feature                                 | Rationale                                                                                         |
|--------------|-----------------------------------------|---------------------------------------------------------------------------------------------------|
| **7**        | Code structure split                    | Before more features. Mechanical, zero risk, permanent payoff.                                    |
| **8**        | Timezone + Stop prompt redesign         | Both are small. Timezone is a correctness fix. Stop prompt redesign pairs naturally with Phase 9. |
| **9**        | Live notes modal + pre-fill stop prompt | New modal pattern. Stop prompt note field pre-filled from live notes.                             |
| **10**       | Quick-add filter modal                  | Biggest daily workflow improvement. Reuses modal pattern from Phase 9.                            |
| **11**       | In-app keyboard shortcuts               | Escape to close modals is immediately useful. Q/N/P/S/R wired to existing messages.               |
| **12**       | Entry time editing + copy button        | Two small review additions. Completes the TANNSS reporting loop.                                  |
| **13**       | UI Polish                               | Makes it enjoyable. After all functionality is in place.                                          |
| **Deferred** | System tray                             | Requires `tray-icon` crate. Add when app is solid.                                                |
| **Deferred** | Global hotkey                           | Requires `global-hotkey` crate. After system tray.                                                |

---

## What is deliberately NOT on this list

- **Task archiving** — Done tasks are already visually distinct. Hiding them is a preference, not a need.
- **Per-task time totals** — interesting stat, not needed for TANNSS reporting.
- **Tags / projects** — scope creep. TANNSS has its own project structure.
- **Multiple workspaces** — not needed for a personal tool.
- **Charts / analytics** — the review is the reporting surface.
