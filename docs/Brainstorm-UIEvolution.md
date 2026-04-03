# Brainstorm — UI Evolution & Feature Expansion

**Date:** 2026-04-02
**Mode:** BRAINSTORM
**Follows:** `docs/Brainstorm-FyneUI.md`, `docs/Feature-FyneUI.md`

---

## Problem statement

The app captures time but the daily workflow is still too slow and fragmented. Specifically:

1. **Focus breaks** — you currently have to stop what you are doing to log time into TANNSS immediately, or risk forgetting it. The app should absorb that capture cost so you can report deliberately at the end of the day instead.
2. **Missing data for reporting** — TANNSS requires start time, end time, and a description of work done. The app currently captures neither notes nor exact start/end timestamps in a reviewable form.
3. **No task structure** — all tasks are equal, with no way to express urgency or importance. Tasks without priority context are invisible until you remember them.
4. **Poor visual quality** — the UI is functional but unpolished. Buttons are unsized, spacing is inconsistent, and the layout has placeholder artefacts still visible. A pleasant interface is a daily tool you actually reach for.

---

## What success looks like

- Hotkey → quick-add → new task + start timer in under 5 seconds, with the previous timer auto-stopped
- At end of day: open review window, read each entry with its notes, fill TANNSS without having to remember anything
- Task list sorted by Eisenhower priority — you can see at a glance what matters
- The app looks considered and clean — spacing is consistent, buttons have weight and hierarchy

---

## Chosen direction

One cohesive evolution of the existing Fyne app. No architectural changes — the store/UI separation stays. New columns added to existing tables via append-only migrations.

---

## Data model changes

Two migrations required. Append to `internal/db/db.go` — never edit existing ones.

```sql
-- Migration 3: task metadata
ALTER TABLE tasks ADD COLUMN urgent    INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks ADD COLUMN important INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks ADD COLUMN description TEXT;

-- Migration 4: time entry notes
ALTER TABLE time_entries ADD COLUMN notes TEXT;
```

**Why notes on `time_entries`, not `tasks`:**
A task may be worked on across multiple sessions. Each session's notes are specific to that block of time ("fixed the login redirect", "wrote tests for the edge case") — not to the task as a whole. The task has a description (what it is); time entries have notes (what you did).

---

## Feature areas

### 1 — Bug fixes (do first, before any new feature)

These are defects in what is already built. They should be resolved before new work begins.

| Bug | Location | What is wrong |
|-----|----------|---------------|
| Task name not shown in timer bar | `time_bar.go:43` | Shows `"Task ID: 42"` — `taskStore` is available but never used for lookup |
| Elapsed does not reset | `time_bar.go:28–33` | Goroutine never updates binding when timer is nil — binding freezes at last value |
| Status cycle incomplete | `task_list.go:38–44` | `Done` case missing from switch — tapping the confirm button on a Done task does nothing |
| Placeholder in layout | `app.go:55,74` | `bottomWidget` is a literal `widget.NewLabel("bottomWidget")` — status bar was placed in the top HBox instead of the bottom zone |
| Quick-add no default mode | `quick_add.go:29` | No default selected — submitting without choosing a mode silently does nothing |
| Quick-add no Enter shortcut | `quick_add.go` | Text entry `OnSubmitted` not wired — user must click Submit |
| Quick-add does not clear | `quick_add.go` | Name entry text persists after submission |

---

### 2 — UI polish

`theme.go` is currently empty. This is where Fyne custom theming lives.

**What needs to improve:**
- Consistent padding and spacing throughout all panels
- Button visual hierarchy — primary actions (start, submit) use `widget.HighImportance`; destructive actions (delete) use `widget.DangerImportance`; secondary actions use default
- Icons on all icon-capable buttons (play, stop, add, delete) rather than Unicode characters
- Timer bar and status bar given proper height and visual separation from the task list
- Task rows: name expands to fill available width; controls are right-aligned and consistently sized
- Quick-add window: compact, centred, with the text field auto-focused on open

**Approach:** implement `fyne.Theme` in `theme.go` to override padding, spacing, and accent colour. Keep it minimal — only override what is visibly wrong, not every value.

---

### 3 — Quick-add evolution (core workflow)

The quick-add window becomes the primary interaction surface.

**New behaviour:**
- Single text field: type to filter existing tasks by name, or type a new name and create
- If an existing task is selected from the filter: timer starts on that task
- If text does not match any task: new task is created and timer starts
- Urgent / Important toggles — two small checkboxes, optional, skippable
- Optional description field — hidden by default, expandable
- Starting a timer **auto-stops the previous timer** — one action, no prompt needed
- Mode selection removed — the action is always "start timer". Capture-only moves to the main window's New Task button.

**Stop-without-start prompt:**
When the user stops the active timer from the timer bar or tray (without starting a new one), a small modal appears:
- Notes field: "What did you do?" (optional, single line)
- Status selector: Keep In Progress / Reset to To Do / Mark Done
- Confirm button

This is the only time a stop requires interaction. Auto-stop (triggered by starting a new timer) is silent.

---

### 4 — Task list

**Sorting by Eisenhower quadrant:**

| Quadrant | Urgent | Important | Label |
|----------|--------|-----------|-------|
| Q1 | ✓ | ✓ | Do First |
| Q2 | ✗ | ✓ | Schedule |
| Q3 | ✓ | ✗ | Delegate |
| Q4 | ✗ | ✗ | Eliminate |

Tasks are sorted Q1 → Q2 → Q3 → Q4. Tasks with neither flag set (no `urgent`, no `important`) appear in a separate **"Needs sorting"** section below the main list — visually distinct, with a label, so you know they need attention.

**Row improvements:**
- Task name expands to fill available width
- Quadrant indicator shown as a coloured tag or label (Q1/Q2/Q3/Q4)
- Status shown as an icon, not a text label
- ▶ button starts timer (auto-stops any running timer)
- Context menu or swipe/right-click: Edit description, Edit priority, Delete

**Delete task:** confirmation dialog — "Delete task and all its time entries?" (cascades to `time_entries`)

---

### 5 — Review window (new)

A dedicated second window, opened from the main window's toolbar or from the tray menu.

**Layout:**
- Date navigation at the top: ← Yesterday · Today · Tomorrow →
- Chronological list of entries for the selected date:
  - Task name · start time → end time · duration
  - Notes field — inline editable, single line, expands on focus
- Gap rows between entries — shown as "⊘ Untracked — 32 min" in a muted style (these are your breaks)
- Summary bar at the bottom: First start · Last end · Total tracked · Total untracked

**Delete time entry:** trash icon per row, with confirmation.

**Notes editing:** clicking a notes field makes it editable inline. Pressing Enter or clicking away saves.

**Store methods needed:**
- `timerStore.ListByDate(date string) ([]TimeEntry, error)`
- `timerStore.UpdateNotes(id int64, notes string) error`
- `timerStore.Delete(id int64) error`
- `taskStore.Delete(id int64) error` (cascades via FK or explicit delete of entries)

---

### 6 — Task description and metadata editing

**Description:** a longer text field on the task (not the time entry). Explains what the task is. Editable from:
- The new task dialog (status bar → + New Task)
- The quick-add window (expandable secondary field)
- A task detail panel or dialog in the main window

**Eisenhower flags:** urgent/important checkboxes, editable from the same places.

---

## Deferred

These are good ideas but do not block the above work.

- **Global hotkey (Task 9)** — still unbuilt. Implement after quick-add is improved, since the hotkey just opens it.
- **Group-by-task view** in review window
- **Tray shortcuts per task** — start timer on a specific task from the tray menu
- **Manual break entry** — "Start Break" button if gap detection proves insufficient
- **Weekly summary** — total time per task across a week

---

## Open questions

- **Task deletion with time entries:** should deleting a task hard-delete all its time entries, or archive them? For now: hard delete with a clear confirmation warning. Revisit if history matters.
- **`ListRecent(days)` vs `ListAll()`:** the task list currently uses `ListAll()`. With Eisenhower sorting applied in Go (not SQL), `ListAll()` is fine — sorting happens after fetch. `ListRecent()` is still worth adding to avoid showing completed tasks from months ago.

---

## Implementation order (suggested)

1. Bug fixes — unblock daily use
2. UI polish — `theme.go` + layout corrections in each file
3. Data migrations — add columns before any feature that needs them
4. Quick-add evolution — core workflow improvement
5. Task list sorting + Needs Sorting section
6. Stop-without-start prompt (notes + status)
7. Review window
8. Global hotkey
