# Brainstorm — Web Stack Decision

**Date:** 2026-03-04
**Status:** Decided — proceeding with templ + htmx + Tailwind CSS

---

## Problem Statement

The original plan used Fyne (Go GUI framework) for a desktop task tracker. Before writing any code, the question arose: could the same app be built with a web-based UI instead, to better match the developer's learning goals and preferred styling approach?

---

## Requirements

- Task tracking and time tracking with a live updating timer display
- Quick-add popup triggerable from a global keyboard shortcut, regardless of which app is currently focused
- Self-contained binary — no external WM or OS configuration required from the user
- Cross-platform: Linux (Wayland), Windows, macOS
- UI flexibility for styling — HTML/CSS strongly preferred over widget-based frameworks
- No JS framework — the goal is to learn Go web patterns, not React

---

## Approaches Considered

### 1. Fyne (original plan)

- Native Go GUI framework, widget-based
- Systray + global hotkey via native Go libraries — cross-platform, self-contained
- Limited styling — Fyne's theme system is not HTML/CSS; customisation requires significant effort
- **Rejected because:** styling flexibility is limited; developer's learning goal is the HTML/CSS/web tech stack

### 2. PWA (templ + service worker, no native process)

- Web UI installable as a "desktop app" via manifest
- No systray, no global hotkey — the browser security model prevents both from a web context
- A system-level keybind (configured in the WM/DE) could open a browser window, but requires user setup
- **Rejected because:** cannot meet the "self-contained, trigger from anywhere" quick-add requirement without external configuration

### 3. Wails (Go + embedded webview)

- Go backend + web frontend rendered in an embedded webview (WebKit/WebView2/WKWebView)
- Handles native integration including tray and hotkeys
- Closest analogue to Electron but with Go
- The frontend workflow expects JS-built static assets served from disk; templ's server-side rendering model does not fit this pipeline reliably
- **Rejected because:** templ is a first-class requirement; Wails + templ is unsupported and fragile

### 4. templ + htmx + Tailwind + Go HTTP server (chosen)

- Go HTTP server serves templ-rendered HTML over localhost
- htmx handles partial page updates — no JS framework needed
- Tailwind CSS via CDN for styling
- `golang.design/x/hotkey` registers global keyboard shortcuts inside the Go process — the native process has full OS access regardless of what the UI layer is
- On hotkey: `exec.Command` opens the browser in `--app` mode at the quickadd route
- Quick-add window closes itself with `window.close()` after submit
- Full HTML/CSS/web tech stack learning while keeping Go as the single language

---

## Chosen Direction

**Option 4.** The key insight: the Go binary is a native process with full OS access. It can register global hotkeys directly, independent of what the UI layer is. The browser renders the UI; the Go process handles the OS integration. This keeps the UI entirely in HTML/CSS while meeting the self-contained, cross-platform requirement.

---

## Known Limitations

**Wayland global hotkeys:** `golang.design/x/hotkey` uses X11 APIs (XGrabKey). On Wayland, this requires XWayland, which is present on most desktop Linux distributions (KDE Plasma, GNOME, etc.) but not guaranteed on minimal compositors. This is the same limitation Electron apps have on Wayland — most users do not notice it because XWayland is available. If XWayland is unavailable, hotkey registration fails silently; the app remains fully usable but quick-add must be opened by navigating to the URL manually.

**No systray icon:** A web-based app has no path to a native system tray icon without a separate native sidecar process. This is an accepted trade-off — the app runs as a browser tab or installed PWA rather than a tray-resident process. A systray can be added later using a minimal non-Fyne library if needed.

**macOS accessibility permission:** On macOS, registering a global hotkey requires the user to grant accessibility permission to the app in System Preferences. This is a one-time prompt, same as any other Mac app using global shortcuts.

---

## Open Questions Deferred

- Port selection: hardcoded to `8080` for now; make configurable via flag later
- Browser detection for `--app` mode: try `google-chrome`, fall back to `chromium`, fall back to `xdg-open` (Linux); platform-specific logic in Task 10
- Tailwind CLI build step for production (removes unused classes from the CDN bundle): deferred until the app is otherwise complete
