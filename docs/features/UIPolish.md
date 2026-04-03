---
name: UIPolish
description: Custom Fyne theme, button hierarchy, icon buttons, task list row layout, timer/status bar visual separation, quick-add auto-focus
type: feature
---

# UIPolish

- **Status:** Planned
- **Created:** 2026-04-02
- **Last updated:** 2026-04-02
- **Touches:** `internal/ui/theme.go`, `internal/ui/app.go`, `internal/ui/time_bar.go`, `internal/ui/task_list.go`, `internal/ui/status_bar.go`, `internal/ui/quick_add.go`

---

## What It Does

Makes the app look considered and consistent. Implements a custom Fyne theme in `theme.go` for improved spacing. Adds button visual hierarchy via `widget.Importance`. Replaces Unicode character buttons with proper icon buttons. Fixes task list row layout so names expand to fill width. Adds visual separation between the three main zones.

---

## Why It Was Built

The functional UI was built first to validate the architecture. Now that the core flow works, polish makes the app pleasant to use every day. An app you enjoy looking at is one you actually reach for.

---

## Implementation Checklist

### Initial implementation — 2026-04-02

- [ ] Implement `appTheme` in `theme.go` with improved inner padding and outer padding
- [ ] Apply theme in `app.go` via `app.Settings().SetTheme()`
- [ ] Set `HighImportance` on primary action buttons (Submit, New Task, Stop)
- [ ] Set `DangerImportance` on the stop button
- [ ] Set `LowImportance` on secondary/quiet buttons (confirm/status dot, edit)
- [ ] Replace `"▶"` and `"■"` with `NewButtonWithIcon` using `theme.*Icon()` throughout
- [ ] Fix task list row: `container.NewBorder` so name expands, controls right-aligned
- [ ] Add `label.Truncation = fyne.TextTruncateEllipsis` to task name labels
- [ ] Wrap timer bar and status bar in `container.NewPadded()`
- [ ] Add `widget.NewSeparator()` between timer bar and task list
- [ ] Return a `show()` function from `newQuickAdd` that calls `Show()` + `Canvas().Focus()`
- [ ] Call the `show()` function in `app.go` instead of `quickAdd.Show()` directly

---

## Technical Notes

_To be filled in after implementation. Say "I finished UIPolish" to trigger this._

---

## Change History

_No changes yet — initial implementation in progress._
