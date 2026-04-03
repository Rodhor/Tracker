# Brainstorm — Framework Migration: Tauri v2 vs Wails v3 vs Flutter Desktop

- **Date:** 2026-04-02
- **Context:** tasktracker is currently built on Fyne v2 (Go, native widget toolkit). This document evaluates whether migrating to a different framework would better serve the app's requirements, and which framework that should be.
- **Status:** Research complete. Decision pending.

---

## Problem Statement

Fyne is functional but has a known ceiling. Its widget set is limited, the look is "Material-ish" but not actually customisable to a modern polished standard without substantial effort, and the ecosystem for advanced features (PDF export, charts, rich data tables) is thin. The question is whether it is worth migrating, and if so, to what.

This document is a structured comparison of the three most credible alternatives as of early 2026.

---

## App Requirements (used as evaluation criteria)

| Requirement | Priority |
|---|---|
| Windows + macOS + Linux support | Must |
| System tray with menu | Must |
| Global keyboard shortcut (quick-add trigger) | Must |
| SQLite local storage | Must |
| Fast startup, low memory use | Must |
| Modern, polished UI (not native widget style) | Must |
| PDF/chart reporting | Future |
| External API integrations | Future |
| Single-language stack or minimal language surface | Preferred |
| Works on Wayland (Linux) | Important |

---

## Current State: Fyne v2

Included for reference.

- **Stars:** ~26k GitHub stars. Active but smaller community than the alternatives.
- **Language:** Pure Go. Single-language stack, no JS/HTML.
- **System tray:** Supported since v2.2 (2022). Cross-platform.
- **Global hotkey:** Not built in. Uses `golang.design/x/hotkey` externally. Works on X11, Wayland support depends on the library — same underlying Wayland problem as everything else.
- **SQLite:** Pure-Go modernc driver works fine.
- **UI look:** Material-inspired, somewhat dated. Limited customisation. Rounded corners added in recent versions but not truly "polished" by modern standards. Custom widgets require significant work.
- **PDF/charts:** No first-party support. Third-party Go libraries (goreport, unipdf) are available but not Flutter/JS-tier.
- **Startup / memory:** Fast. Native binary, no webview overhead. Memory around 20–40 MB typical.
- **Verdict for this app:** Adequate for current scope. Ceiling is real: a reporting view with charts and PDF export would be painful to build. The UI will always look "Fyne-like".

---

## Option 1 — Tauri v2

**What it is:** Rust backend + system WebView (not bundled Chromium) + any web frontend. The Rust core handles native APIs; the frontend is HTML/CSS/JS via your choice of framework.

### Release status
- **v2 stable** since October 2024. Currently at v2.10.3 (March 2026). Actively maintained with releases every few weeks.
- CLI, runtime, and all core plugins ship together. No alpha/beta caveat.

### System tray
- Official `tauri-plugin-tray` included in the plugins workspace.
- All three platforms: confirmed supported.
- Known issue: tray icon rendered twice/overlapping on Linux (bug filed September 2025, still open). Minor cosmetic issue, not a blocker.

### Global hotkey on Linux / Wayland
This is the most nuanced point in the entire comparison.

- Official `tauri-plugin-global-shortcut` lists Linux as supported, but this is X11 only.
- The underlying `tauri-apps/global-hotkey` library has had a Wayland feature request open since 2022 (issue #28).
- **As of March 27, 2026**, PR #172 — "Unified Wayland support via XDG GlobalShortcuts portal" — was opened. It is not a draft. It closes issue #28. It uses `ashpd` (Rust D-Bus library) to talk to the `org.freedesktop.portal.GlobalShortcuts` portal.
- The PR is open but not merged as of April 2, 2026. There is also a competing PR #162 (opened September 2025) with a different approach that is still open.
- **Real-world implication:** If you are on Wayland today (which CachyOS with a modern compositor likely is), global hotkeys may silently not fire. The fix is in-flight but not shipped. You could track `main` or wait for a release.
- A workaround exists: register the hotkey as a DE-level shortcut (GNOME Settings / KDE System Settings) that runs your app. Less seamless but functional.
- The XDG portal approach, once merged, will work on GNOME, KDE Plasma, and Hyprland (all of which implement the portal). It will not work on compositors that do not implement the portal spec (rare in practice for modern setups).

### SQLite
- Official `tauri-plugin-sql` supports SQLite, MySQL, and PostgreSQL. Ships in the plugins workspace, actively maintained.
- Alternatively: pure Rust crate (`rusqlite`) in the backend, no plugin needed.

### PDF / reporting
- Two approaches work well:
  1. **Frontend (easier):** Use `@react-pdf/renderer` or `jsPDF` in the web layer. Since you control the HTML, you can render any chart library (Recharts, Chart.js, D3) and print to PDF.
  2. **Backend (Rust):** `printpdf` crate for programmatic PDF generation. Less flexible for chart rendering but avoids JS dependency.
- For charts in the UI: any modern JS charting library works (Recharts, Visx, Chart.js, Observable Plot). This is a major advantage over Fyne.

### Startup time / memory
- Benchmarks (Elanis comparison, release build, Windows): ~757ms startup. Lower than Electron by a factor of 2–3x but not the fastest.
- Memory: ~4 MB at idle (Windows benchmark). Real-world app with a webview will be higher — typically 30–50 MB. Significantly better than Electron's 200–300 MB.
- Linux release build startup in CI: ~25 seconds — this is a CI artefact, not real. Local release builds are sub-second.

### Svelte support
- First-class. Tauri's official "create project" scaffolding includes a Svelte/SvelteKit template.
- SvelteKit needs `@sveltejs/adapter-static` to build static output for Tauri. SSR must be disabled (or only used for routes that do not need Tauri APIs). This is standard, well-documented practice.
- TailwindCSS: works exactly as it does in any Svelte project — add the Tailwind plugin to Vite config. Zero special handling for Tauri.
- **Stack fit for this user:** Go (current) → Rust (new, learning investment) + TypeScript/Svelte/Tailwind (already known). The frontend side is immediately productive. The Rust side is learning.

### Rust learning curve (for a Go developer)
- Consensus across multiple 2024–2025 sources: steeper than Go, but closer than it used to be.
- The borrow checker is the main obstacle. Go developers are used to GC; Rust requires explicit thinking about ownership and lifetimes.
- For a Tauri app, the Rust surface is actually small: you write command handlers that receive typed arguments and return results. You are not implementing complex data structures or async runtimes from scratch. The Tauri framework absorbs most of the Rust complexity.
- Realistic assessment: productive in Tauri Rust commands within 2–4 weeks for a competent Go developer. Not painful. The Rust compiler's error messages genuinely help.

### Community health
- 104,854 GitHub stars. Growing.
- 1,390 open issues — proportionate to the project size and multi-platform scope.
- Daily commits. Commercial and open-source projects shipping with Tauri.
- Discord active. Tauri v2 + Svelte has a visible community with blog posts and tutorials.

### Summary for this app
| Criterion | Assessment |
|---|---|
| System tray | Yes, all platforms |
| Global hotkey (X11) | Yes |
| Global hotkey (Wayland) | Not yet in stable — fix in review |
| SQLite | Yes, official plugin |
| UI polish | Unlimited — it's a browser |
| PDF/charts | Excellent — JS ecosystem |
| Startup/memory | Good. ~30–50 MB real-world |
| Svelte/Tailwind | First-class |
| Language investment | Rust learning required for backend |

---

## Option 2 — Wails v3

**What it is:** Go backend + system WebView + any web frontend. Conceptually identical to Tauri but Go instead of Rust for the backend.

### Release status
- **v3 is in alpha.** Latest alpha: v3.0.0-alpha.74 (March 2026). Not a stable release.
- v2 is stable at v2.12.0 (March 2026). v2 is the production recommendation.
- The Wails team has explicitly stated there is no release date for v3 stable. The alpha API is described as "reasonably stable" and some teams are running it in production, but that is a developer's own risk assessment, not an official endorsement.
- v3 alpha has active bugs: window API issues, cross-compilation problems for darwin builds, Linux Wayland blank window issues.

### System tray
- v3 has system tray support. Official docs at `v3alpha.wails.io/features/menus/systray/` confirm it.
- Linux uses StatusNotifierItem (modern DEs). GNOME may require AppIndicator extension.
- Bug filed: context menu on Linux systray hides the app window (issue from August 2025, still open in v3).
- v2 has systray support via the `wails/v2` runtime API, which is stable.

### Global hotkey
- **Not built into Wails at all** — v2 or v3. Issue #3112 was closed as "won't implement natively; use a third-party library."
- The suggested approach: use `golang-design/hotkey` package directly in Go, bypassing Wails.
- `golang-design/hotkey` has the same Wayland limitation as every other approach: X11 only. No XDG portal integration as of this writing.
- This means Wails is in the same position as Fyne on this point — you rely on an external Go library that does not support Wayland.

### SQLite
- No official plugin. Use Go database libraries directly: `modernc.org/sqlite` (pure Go, no CGO) or `mattn/go-sqlite3` (requires CGO). Since you are already using modernc in the current app, this is a direct carry-over.

### UI / frontend
- Same as Tauri: any web framework. Svelte/Tailwind/TypeScript works identically.
- Go-to-JavaScript bridge is the same concept as Tauri's commands but implemented differently. Wails uses code generation to produce TypeScript bindings from your Go structs and methods — a praised feature.

### PDF / reporting
- Same as Tauri: JS ecosystem available in the frontend. Go PDF libraries (goreport, unipdf) available in the backend.

### Startup / memory
- Benchmarks (release, Windows): ~660ms startup, ~4 MB memory. Comparable to Tauri.
- Linux: ~212ms release startup in benchmarks — faster than Tauri in that test.

### Community health
- 33,562 stars. Healthy for a Go-specific tool, but much smaller than Tauri.
- 347 open issues. Tidy issue tracker relative to size.
- Active daily commits — the v3 alpha is being worked on continuously.
- Smaller ecosystem of tutorials, examples, and third-party plugins compared to Tauri.

### Summary for this app
| Criterion | Assessment |
|---|---|
| System tray | Yes (v3 alpha has a known bug) |
| Global hotkey (X11) | Via third-party Go library |
| Global hotkey (Wayland) | Not supported anywhere in the stack |
| SQLite | Yes, via Go libraries directly |
| UI polish | Unlimited — it's a browser |
| PDF/charts | Good — JS ecosystem |
| Startup/memory | Comparable to Tauri |
| Svelte/Tailwind | Works, same as any web stack |
| Language investment | Zero — already Go |
| Stable release | v2 yes, v3 no |

---

## Option 3 — Flutter Desktop

**What it is:** Google's cross-platform UI framework. Does not use a WebView — Flutter renders every pixel with its own engine (Impeller, successor to Skia). Single language: Dart.

### Release status
- Flutter 3 desktop (Windows/macOS/Linux) has been **stable since Flutter 3.0 (2022)**. As of early 2026, Flutter is at approximately v3.38–v3.41 (confirmed stable). Not in beta or alpha.
- Google is the primary sponsor. Active full-time engineering team.

### System tray
- Not in the Flutter SDK. Requires third-party packages.
- `tray_manager` (pub.dev, LeanFlutter): supports Windows, macOS, Linux. Based on a native C++ core. Last release: relatively recent. Used in production Flutter desktop apps.
- GNOME caveat: AppIndicator extension required (same as Wails/Tauri on GNOME without app indicator support).
- Quality: the LeanFlutter ecosystem (`tray_manager`, `hotkey_manager`, `window_manager`) is the de facto standard for Flutter desktop extras, maintained by one person with community contributions. Not Google-official.

### Global hotkey on Linux / Wayland
- `hotkey_manager` (pub.dev, LeanFlutter, v0.2.3, last updated May 2024): supports Linux, Windows, macOS.
- On Linux it uses X11 under the hood. **No Wayland support confirmed.** Last update was May 2024 and there is no changelog entry for Wayland.
- Flutter's own Wayland support for rendering is progressing (GTK4 + WebKitGTK 6.0 discussion in Wails is a separate concern; Flutter uses its own renderer). Flutter renders on Wayland via GTK's Wayland backend, but the global hotkey library sits outside that.
- The issue tracker for `hotkey_manager` shows no Wayland-specific work.
- **Bottom line:** same problem as Wails — no Wayland global hotkey support, and no active work toward it.

### SQLite
- `sqflite_common_ffi` package supports SQLite on desktop via FFI. Well maintained.
- Alternative: `drift` (formerly Moor) — a type-safe reactive persistence library for Dart/Flutter that uses SQLite under the hood. Well regarded, actively maintained, significantly more ergonomic than raw SQL.

### PDF / reporting
- Strong ecosystem: `syncfusion_flutter_pdf` (commercial/community license), `pdf` package (pure Dart, MIT), `printing` package for print/PDF export.
- Charts: `fl_chart` (free, good quality), Syncfusion charts (commercial), `graphic` (grammar of graphics).
- This is probably Flutter's strongest point relative to Fyne — reporting is much more achievable.

### Startup time / memory
- Published benchmarks (getstream.io Flutter desktop vs Electron): Flutter ~38 MB memory vs Electron ~100 MB. Flutter startup under 200 ms in their test.
- The Elanis comparison shows Flutter at ~109 ms startup on Windows in CI — fastest of all frameworks tested.
- Linux CI number (31 seconds) is almost certainly a CI environment artefact — Flutter on Linux in CI is known to have cold-start issues. Real local startup is sub-second.
- Memory is moderate and steady. Larger baseline than a pure Go binary but much smaller than Electron.

### Dart learning curve (for a Go/TypeScript developer)
- Dart is often described as the easiest new language to pick up for someone coming from Go or TypeScript. It is garbage-collected, has a familiar C-style syntax, strong typing, async/await, and no borrow checker.
- The Flutter widget tree model is the real learning investment, not the language itself.
- Downside: Dart is essentially Flutter-exclusive. Learning it does not translate to other ecosystems the way that learning Rust or TypeScript does.

### UI polish
- Flutter renders its own pixels — it does not use native widgets. This means it looks identical on all platforms (no native widget inconsistencies) but also does not integrate with the OS look-and-feel.
- The Material and Cupertino design systems are built in. Custom themes and designs are achievable and many production Flutter apps look very modern and polished.
- For a personal tool where you control the entire design language, this is an advantage.

### Community health
- 175,805 GitHub stars. Largest of all frameworks compared here, by a large margin.
- 12,531 open issues — the most of any option, but proportionate to user base.
- Google-backed, enterprise-adopted. Not at risk of abandonment.
- Desktop is still a "second class citizen" in the Flutter ecosystem compared to mobile. Most packages target mobile first. Some desktop-specific packages are maintained by one person.

### Summary for this app
| Criterion | Assessment |
|---|---|
| System tray | Yes, via third-party (LeanFlutter) |
| Global hotkey (X11) | Yes, via hotkey_manager |
| Global hotkey (Wayland) | Not supported |
| SQLite | Yes, via sqflite_common_ffi or Drift |
| UI polish | Excellent — own renderer, full control |
| PDF/charts | Excellent — best of the three options |
| Startup/memory | Fast (~38 MB, sub-200ms) |
| Svelte/Tailwind | N/A — Flutter has its own UI system |
| Language investment | Dart (easy to learn, low transfer value) |
| Stable release | Yes |

---

## The Wayland Global Hotkey Problem: A Cross-Framework Assessment

This deserves its own section because it affects all three options.

**The core constraint:** Wayland does not expose global keyboard input to applications for security reasons. A protocol extension (`org.freedesktop.portal.GlobalShortcuts`) was added to the XDG Desktop Portal spec to address this. Support for that portal exists in GNOME (partial), KDE Plasma, and Hyprland compositors, but it is not universally implemented.

**Framework positions as of April 2026:**

| Framework | Wayland hotkey status |
|---|---|
| Tauri | PR #172 open (March 2026) — not merged, not released. When merged and released, will work on compositors that support the portal. |
| Wails | Not planned. Would need to be implemented in `golang-design/hotkey` independently. No active work. |
| Flutter | Not supported in `hotkey_manager`. No active work. |
| Current Fyne | Same as Wails — uses `golang-design/hotkey`, same limitation. |

**Practical guidance for this app on CachyOS (likely Wayland):** The app's central feature (global hotkey to trigger quick-add) does not work reliably on Wayland with any of these frameworks today. Tauri is the closest to a working solution. The fallback in all cases is to register the shortcut at the desktop environment level.

---

## Other Options Considered

### Dioxus (Rust, native or webview)
- Rust-based UI framework. Can target desktop via webview (like Tauri) or native rendering.
- ~25k stars. Community smaller than Tauri. Tooling described as immature.
- No meaningful advantage over Tauri for this use case.
- Not recommended.

### Neutralino
- Tiny webview framework. Lighter than Tauri in benchmarks (~279ms startup, ~4 MB).
- Very small ecosystem. Minimal official plugin set.
- Would have all the same Wayland/hotkey/tray pain points without the ecosystem to solve them.
- Not recommended.

### .NET MAUI
- Microsoft's cross-platform framework for C#. Desktop support is there but Linux support is not first-class (community-maintained, not Microsoft-official).
- Requires C# — a new language for this user. Higher overhead than necessary for a personal tool.
- Not recommended.

### Fyne (status quo)
- Sticking with Fyne is a valid choice if the UI ceiling and reporting gap are acceptable.
- No migration cost. The current stack works.
- If the app stays at its current scope (task list, timer, system tray, quick-add), Fyne is fine.
- If reporting (PDF/charts) becomes important, the gap widens.

---

## Trade-off Summary

| | Tauri v2 | Wails v3 | Flutter |
|---|---|---|---|
| **Stable today** | Yes | No (v3 alpha; v2 stable) | Yes |
| **Language fit** | Rust (new) + TS/Svelte (known) | Go (known) + TS/Svelte (known) | Dart (new, easy) |
| **System tray** | Yes | Yes (v3 bug; v2 stable) | Yes (third-party) |
| **Global hotkey X11** | Yes | Yes (third-party Go lib) | Yes (third-party) |
| **Global hotkey Wayland** | Almost (PR open) | No | No |
| **SQLite** | Official plugin | DIY (easy carryover) | Good packages |
| **UI polish ceiling** | Unlimited (web stack) | Unlimited (web stack) | Excellent (own renderer) |
| **Reporting/PDF** | Excellent | Good | Excellent |
| **Startup/memory** | ~30–50 MB, ~750ms | ~30–50 MB, ~660ms | ~38 MB, <200ms |
| **Ecosystem size** | Large | Medium | Very large (mobile-first) |
| **Wayland future** | Active PR — likely soon | Unlikely without external work | Unlikely near-term |

---

## Recommendation

**If Wayland global hotkey support matters most:** Tauri v2 is the only realistic path in the near-to-medium term. The Wayland PR (#172) is ready for review and not a draft. When it merges, Tauri will be the only framework here with real Wayland global hotkey support. The cost is learning Rust backend commands — manageable given the small surface area involved.

**If staying in Go is the priority:** Wails v3 is the natural choice, but wait for stable. v2 is usable now and v3 is well along. The Wayland hotkey gap exists and has no clear resolution path within the Wails ecosystem.

**If reporting quality matters most and Wayland hotkey is a lower priority:** Flutter has the best out-of-the-box story for charts and PDF, a fast/light runtime, and a stable desktop target. The language cost (Dart) is low. The main gap is Wayland global hotkeys, and the LeanFlutter ecosystem being community-maintained.

**If staying with Fyne:** Entirely reasonable for the current scope. Revisit when reporting becomes a concrete requirement, or if the Tauri Wayland PR ships and the Fyne UI ceiling becomes a more active frustration.

---

## Open Questions

1. How important is the global hotkey on Wayland right now, given you are on CachyOS? Is the DE-level workaround acceptable in the short term?
2. Is the reporting requirement (PDF/charts) near-term or future-year thinking?
3. Is learning Rust a feature (an opportunity to learn) or a cost (time you do not have)?
4. Would you consider running Wails v2 now and migrating to v3 when it stabilises, accepting the migration work?

---

## Next Step

If the direction is Tauri: move to FEATURE to plan the migration sequence (DB layer migration, window/tray, quick-add, existing UI).

If staying with Fyne: close this brainstorm and revisit when a concrete trigger arrives.

If Wails v3: revisit when v3 reaches stable, or proceed with v2 as the foundation.
