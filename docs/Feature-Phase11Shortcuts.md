# Feature Plan — Phase 11: In-App Keyboard Shortcuts

**Date:** 2026-04-06
**Permanent record:** `docs/features/Phase11Shortcuts.md`

---

## What this phase builds

Six keyboard shortcuts for the primary actions in the app. No modifier keys required — the
shortcuts are single character or named keys that work when no text field is focused.

| Key    | Action                                            |
|--------|---------------------------------------------------|
| `Q`    | Open quick-add modal                              |
| `N`    | Open note modal (only when timer is running)      |
| `P`    | Pause or resume the active timer                  |
| `S`    | Open stop prompt (only when timer is running)     |
| `R`    | Toggle review screen (open or close)              |
| Escape | Close whichever modal or prompt is currently open |

Escape also cancels the stop prompt (resuming the timer) and closes the review screen —
anything that can be opened can be closed with the same key.

---

## What already exists

- `iced::event::listen_with` subscription in `subscription()` — already handles Ctrl+Enter
  via `SubmitActiveEditor`; Phase 10 extends it for arrow keys and Escape
- `PauseTimer`, `ResumeTimer`, `OpenReview`, `CloseReview` messages — already implemented
- `OpenNoteModal`, `OpenStopPrompt`, `OpenQuickAdd`, `CloseQuickAdd` — implemented in Phase 9/10
- `active_entry: Option<TimeEntry>` — used to guard timer-specific shortcuts
- `screen: Screen` — used to decide open vs close for review

**Important:** Phase 10 must be implemented before Phase 11. Phase 11's `CloseActiveModal`
message is mentioned in the Phase 10 plan (Escape handling in the subscription). You can
implement `CloseActiveModal` in Phase 10 and leave it as a no-op stub, or implement both
phases together.

---

## Changes overview

| File             | Change                                                                     |
|------------------|----------------------------------------------------------------------------|
| `src/message.rs` | Add 3 new messages: `TogglePauseTimer`, `ToggleReview`, `CloseActiveModal` |
| `src/app.rs`     | Add 3 new `update()` arms; extend `subscription()` for character keys      |

No new files. No new state fields. No UI changes.

---

## Step 1 — Add new messages (`src/message.rs`)

Add these three variants in the keyboard shortcuts section (or at the end, grouped by purpose):

```rust
// Keyboard shortcut dispatchers — resolved to specific messages in update()
TogglePauseTimer,
ToggleReview,
CloseActiveModal,
```

**Why three new messages instead of emitting the existing ones from the subscription?**

The shortcuts `P` and `R` map to different messages depending on current state: `P` maps to
either `PauseTimer` or `ResumeTimer`; `R` maps to either `OpenReview` or `CloseReview`.
The subscription function pointer cannot capture `self`, so it cannot check which state we
are in. The solution is to emit a single "intent" message and let `update()` resolve it —
the same pattern used by `SubmitActiveEditor` for Ctrl+Enter.

`CloseActiveModal` follows the same logic: Escape should close whichever modal is open, but
the fn pointer cannot check which one that is.

---

## Step 2 — Implement the three new `update()` arms (`src/app.rs`)

```rust
Message::TogglePauseTimer => {
    if let Some(entry) = &self.active_entry {
        if entry.is_paused() {
            return Task::done(Message::ResumeTimer);
        } else {
            return Task::done(Message::PauseTimer);
        }
    }
    // No active timer — no-op
}
Message::ToggleReview => {
    return Task::done(match self.screen {
        Screen::Review => Message::CloseReview,
        Screen::Tracker => Message::OpenReview,
    });
}
Message::CloseActiveModal => {
    // Priority order matches view() — stop prompt first, then note modal, then quick-add.
    // CancelStop resumes the timer; the others just close.
    if self.stop_prompt_open {
        return Task::done(Message::CancelStop);
    } else if self.note_modal_open {
        return Task::done(Message::CancelNoteModal);
    } else if self.quick_add_open {
        return Task::done(Message::CloseQuickAdd);
    } else if self.screen == Screen::Review {
        return Task::done(Message::CloseReview);
    }
    // Nothing open — Escape is a no-op
}
```

**Why `Task::done()` instead of directly mutating state?**

Each target arm (`PauseTimer`, `CancelStop`, etc.) already contains the correct mutation
logic — timer state, clearing fields, saving. Duplicating that logic here would create two
code paths for the same operation that can drift out of sync. `Task::done()` re-routes the
message back through `update()` so the existing arm handles it once.

**Why does `CloseActiveModal` include review?**

Escape should always mean "go back" or "close". If the user is on the review screen with no
sub-modal open, pressing Escape returns them to the tracker. The shortcut key `R` already
toggles review — Escape is a second, more intuitive way to close it.

**Why does `TogglePauseTimer` guard on `active_entry`?**

`P` with no timer running should do nothing. Without the guard, pressing `P` would call
`PauseTimer` on an `active_entry` of `None`, which is a no-op in the existing arm anyway —
but the guard makes the intent explicit.

---

## Step 3 — Extend `subscription()` for character key shortcuts (`src/app.rs`)

Phase 10 already sets up `event::listen_with` for arrow keys and Escape. Now extend the
same match to handle character keys when their events are not captured by a focused widget.

Replace (or extend) the `keys` subscription in `subscription()`:

```rust
let keys = iced::event::listen_with(|event, status, _window| {
    use iced::event::Status;
    use iced::keyboard::{self, key::Named};

    if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
        match key {
            // Ctrl+Enter or Shift+Enter — submit whichever text editor is active
            keyboard::Key::Named(Named::Enter)
                if modifiers.control() || modifiers.shift() =>
            {
                Some(Message::SubmitActiveEditor)
            }

            // Arrow keys — quick-add list navigation.
            // Intercepted even when a text input is focused (see Phase 10 plan).
            keyboard::Key::Named(Named::ArrowUp) => Some(Message::QuickAddMoveUp),
            keyboard::Key::Named(Named::ArrowDown) => Some(Message::QuickAddMoveDown),

            // Escape — close whatever is open.
            // No-op if nothing is open (handled in CloseActiveModal arm).
            keyboard::Key::Named(Named::Escape) => Some(Message::CloseActiveModal),

            // Single-character shortcuts — only fire when no widget has captured the event.
            // Status::Ignored means the key was not consumed by any focused text field.
            keyboard::Key::Character(ref c) if status == Status::Ignored => {
                match c.as_str() {
                    "q" | "Q" => Some(Message::OpenQuickAdd),
                    "n" | "N" => Some(Message::OpenNoteModal),
                    "p" | "P" => Some(Message::TogglePauseTimer),
                    "s" | "S" => Some(Message::OpenStopPrompt),
                    "r" | "R" => Some(Message::ToggleReview),
                    _ => None,
                }
            }

            _ => None,
        }
    } else {
        None
    }
});
```

And update the `Subscription::batch` call — replace both `tick` and `submit` with `tick` and
`keys` (the `keys` subscription now covers everything the old `submit` did plus the new
shortcuts):

```rust
iced::Subscription::batch([tick, keys])
```

The old `submit` variable and the conditional block around it can be removed entirely.

**Why `Status::Ignored` for character shortcuts only?**

When a `text_input` or `text_editor` is focused, the user is typing. Every character key is
`Status::Captured` — iced delivered it to the widget. If we ignored the status for `Q` for
example, pressing `Q` while typing a note would both add "Q" to the note AND open the
quick-add modal. `Status::Ignored` gates character shortcuts behind "no widget is focused",
which is the correct condition.

Arrow keys and Escape are different: they are not printable characters, and intercepting them
regardless of status is intentional (see Phase 10 plan for arrow keys; Escape in a modal is
always "close the modal" no matter what is focused).

**Why remove the old conditional block?**

The old `submit` subscription registered `event::listen_with` only when an editor was open.
The new `keys` subscription is always registered. This is fine — the subscription receives
all key events but the function returns `None` for most of them, which is a zero-cost path.
The conditional registration was an optimisation for a case where the cost is already
negligible. Removing the condition simplifies the subscription function significantly.

---

## Step 4 — Guard OpenNoteModal and OpenStopPrompt in `update()` (optional)

Character shortcuts fire without checking state. `N` and `S` should do nothing when no timer
is running. The existing arms for `OpenNoteModal` and `OpenStopPrompt` do not crash if there
is no active entry — they just open empty modals or get pre-fill data from `None`. But
opening an empty stop prompt with no timer is confusing.

Add a guard at the top of each arm:

```rust
Message::OpenNoteModal => {
    if self.active_entry.is_none() {
        // N key with no timer — do nothing
    } else {
        // ... existing arm body unchanged
    }
}
Message::OpenStopPrompt => {
    if self.active_entry.is_none() {
        // S key with no timer — do nothing
    } else {
        // ... existing arm body unchanged
    }
}
```

**Why put the guard in `update()` rather than in the subscription?**

The subscription cannot check state. `update()` is the correct place for all state-based
decisions. Adding the guard here means the message can still be sent from the UI (e.g., a
button that is only rendered when a timer is active) without special-casing.

---

## Step 5 — Verify

```
cargo check
cargo run
```

Test all shortcuts in order:

1. **Q** — no timer running. Quick-add opens. Escape closes. Q opens again, type a task name,
   Enter starts the timer.
2. **N** — timer running. Note modal opens. Type a note. Ctrl+Enter saves.
   N again — note is pre-filled. Escape closes without changes.
   Stop the timer. N — nothing happens (no active timer).
3. **P** — timer running. Timer pauses. Elapsed stops ticking. P again — timer resumes.
   P with no timer — nothing happens.
4. **S** — timer running. Stop prompt opens with pre-filled note. Escape resumes the timer.
   S again — stop prompt opens. Ctrl+Enter stops and saves.
   S with no timer — nothing happens.
5. **R** — opens review screen. R again — returns to tracker. Escape from review — returns to
   tracker (same as R).
6. **Escape in quick-add** — modal closes, no task started.
7. **Escape in note modal** — modal closes, no note saved.
8. **Escape in stop prompt** — modal closes, timer resumes.
9. **No double-open:** press Q while quick-add is open — `OpenQuickAdd` fires again.
   This re-initialises the input to empty. That is acceptable — if it becomes annoying,
   add `if !self.quick_add_open` guard to the `OpenQuickAdd` arm.
10. **Typing does not trigger shortcuts:** open quick-add, type "q r n s p" — those letters
    appear in the input, no other modals open.

---

## What this phase does not do

- **No shortcut display in the UI** — Phase 13 can add a help row or tooltip hints
- **No user-configurable shortcuts** — hardcoded for now
- **No global OS hotkey** — Q opens quick-add from within the app only; a system-wide
  hotkey (like the original Fyne version had) is out of scope for the iced rewrite

---

## Commit message (draft)

```
feat(app): keyboard shortcuts for primary actions

Q opens quick-add, N opens note modal, P toggles pause,
S opens stop prompt, R toggles review, Escape closes any modal.
Single-character shortcuts are gated on Status::Ignored so they
do not fire while typing in a text field.
```
