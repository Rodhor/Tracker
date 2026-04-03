# Brainstorm — Framework Research 2026

**Research date:** 2026-04-02
**Context:** Evaluating cross-platform desktop framework options for a small personal time-tracking app (system tray, global hotkey, SQLite, future PDF reporting, must look modern on Windows/macOS/Linux).

---

## Area 1: Pure Rust GUI Frameworks

### GitHub star counts as of 2026-04-02

| Framework | Stars | Last push |
|-----------|-------|-----------|
| Dioxus    | 35,512 | 2026-03-31 |
| iced      | 30,040 | 2026-04-02 |
| egui      | 28,605 | 2026-03-29 |
| Slint     | 22,144 | 2026-04-02 |
| floem     |  4,070 | 2026-03-30 |

---

### 1. egui / eframe

**Current version:** 0.34.0 (released 2026-03-26)
**Release cadence:** Roughly quarterly. Recent versions: 0.33.0 (Oct 2025), 0.34.0 (Mar 2026).

**What it is:** Immediate-mode GUI. Every frame you re-describe the entire UI imperatively in Rust code. There are no retained widget objects; the draw call and the event handling happen in the same pass.

**Visual quality ceiling:** Intentionally non-native. egui renders entirely with its own GPU-first renderer (wgpu or glow backend) and does not touch platform widgets at all. The default theme looks utilitarian — closer to Dear ImGui than to a polished consumer app. The ceiling is higher than it looks: third-party theme crates (catppuccin-egui, egui-aesthetix) exist and the Visuals struct exposes colors, rounding, spacing, and shadows. Community showcase screenshots range from "developer tool" to "pretty usable product UI," but achieving something that looks like a modern macOS/Windows native app requires sustained theming effort.

**System tray:** Not built into eframe. The `tray-icon` crate (maintained by the Tauri organisation, ~500k downloads) works alongside eframe. Integration is manual: you spin up the tray icon independently and wire events through your own channel. Working examples exist in the wild (ClipVault is one).

**Global hotkey:** The `global-hotkey` crate (also from the Tauri org) works alongside eframe. Same pattern: external crate, manual wiring. Widely used.

**Cross-platform parity:** Good. wgpu backend gives consistent rendering across Windows, macOS, and Linux. No platform-specific visual surprises because nothing native is used.

**Accessibility:** The April 2025 Rust GUI survey found egui is the only Rust-native (non-WebView) GUI that actually passes Windows Narrator. IME has known rough edges (Tab key eaten by egui during composition), but basic Latin text entry is fine.

**Production readiness for a personal tool:** Yes, with caveats. The author's own statement is that "egui has breaking changes each version" and is not stable-API. However, it builds, it ships, and third-party apps (Rerun, ClipVault, etc.) are shipping with it today. For a personal tool you own end-to-end, version-pinning is not a problem.

**Known gaps for this use case:**
- No built-in system tray or global hotkey — external crates required, with manual event-loop integration
- Visual polish requires active theming work; defaults look like a debug tool
- Immediate-mode has philosophical overhead for complex reactive state (timers, live updating values) — manageable but different from retained-mode thinking

---

### 2. iced

**Current version:** 0.14.0 (released 2025-12-07)
**Roadmap:** One more experimental version before 1.0. The author (Héctor Ramón) has stated 0.14 → 1.0 is the plan.

**What it is:** Retained-mode, Elm-architecture (Model / Update / View). You define a message type, an update function, and a view function. The runtime diffs widget trees between frames. Structurally similar to a Rust-native React.

**Visual quality ceiling:** Higher than egui out of the box. iced has a proper styling system with cascading themes. System76 is using it for the COSMIC desktop shell — the only production desktop environment built on iced — which gives it genuine credibility. Community-built apps look noticeably more polished than typical egui apps.

**System tray:** Not built in. A PR (#3021) adding native tray-icon integration was merged or in late-stage review in mid-2025. The `tray-rs` crate explicitly targets iced + egui. The situation is "works via external crate" rather than "turn on a flag."

**Global hotkey:** No built-in support. Same external-crate approach as egui (global-hotkey crate). No first-class iced integration documented.

**Cross-platform parity:** Good. wgpu renderer (called "Iced graphics") gives consistent output. Linux support is solid; Wayland is handled by winit underneath.

**Accessibility:** The April 2025 survey found iced is one of the notable failures: "Windows Narrator: nope," IME "won't even switch into active mode." This matters less for a personal tool but is worth knowing.

**Production readiness for a personal tool:** Approaching it. 0.14 is still labeled experimental. The ecosystem (e.g., iced-aw for extra widgets) sometimes lags behind the main crate version, causing compatibility headaches. For a personal project, this is tolerable if you stay close to the main crate. The architecture is significantly more structured than egui, which is an advantage for anything with complex state.

**Known gaps for this use case:**
- System tray and global hotkey via external crates, no native integration
- Still pre-1.0 with occasional breaking changes
- Ecosystem crates (iced-aw, etc.) sometimes pin to older versions

---

### 3. Slint

**Current version:** 1.15.1 (released 2026-02-12)
**Release cadence:** Very active. A major or minor release roughly every 6–8 weeks.

**What it is:** A declarative UI toolkit with its own DSL (`.slint` files) compiled to Rust (or C++, JS, Python). The DSL handles layout, data binding, animation, and styling. Rust code implements business logic and connects to the UI through generated structs and callbacks.

**License for a personal app:** Free. Slint offers:
- **GPLv3** — free for open-source projects
- **Royalty-free 2.0** — free for proprietary desktop/mobile/web applications (the key one: you can ship a closed personal app for free)
- **Commercial** — for ISVs shipping products where GPL is incompatible

A personal time-tracker qualifies for the royalty-free license with no cost.

**Visual quality ceiling:** The highest of any native-renderer Rust framework. Slint's default widgets look like a real modern application. The DSL allows pixel-perfect custom components. The team actively maintains a design-system-style component library. This is the only Rust GUI where the defaults are presentable to non-technical users without extra theming work.

**System tray:** Active support. Slint can manage system tray icons with context menus on Windows, macOS, and X11/Wayland on Linux (via dbus). Slint 1.4 added `run_event_loop_until_quit()` specifically to keep the event loop alive when all windows are hidden — the pattern required for tray-only apps.

**Global hotkey:** Partial/in-progress. As of Slint 1.13, you can capture key presses globally via `FocusScope` and `capture-key-pressed`. Slint is working on a declarative `keyboard-shortcut` type with `@keys(...)` macro. This is in-window focus capture, not a true OS-level global hotkey. True global hotkeys (fire when app is unfocused/hidden) likely still require an external crate.

**Cross-platform parity:** Excellent. Slint renders its own widgets (not native platform widgets) consistently. It also supports a "native style" backend that uses platform-native widgets on some platforms.

**Accessibility:** The April 2025 survey found "Narrator works perfectly" for Slint — the best result of any native-renderer Rust GUI tested.

**Production readiness for a personal tool:** Yes, this is the most production-ready option in the native Rust space. Version 1.x with a stable API commitment. Active company (SixtyFPS GmbH) behind it.

**Known gaps for this use case:**
- The DSL is a learning curve if you want to stay in pure Rust
- True OS-level global hotkey (fires when the app window is hidden/unfocused) still needs verification — the in-window key capture is not the same thing
- The DSL-to-Rust binding layer adds some friction compared to writing UI imperatively

---

### 4. Dioxus

**Current version:** 0.7.x (active, 35k+ stars)
**What it is:** React-inspired Rust framework. You write components with hooks (use_state, use_effect, etc.) like React. Targets web, desktop, mobile from a shared component model.

**How the desktop target works:** WebView-based. It embeds the system WebView (WebView2 on Windows, WebKit on macOS, WebKitGTK on Linux) and renders your component tree as HTML/CSS/JS inside it. This is the same fundamental architecture as Tauri — not a native renderer. A future "Blitz" native renderer (wgpu-based) is planned but not production-ready.

**Visual quality ceiling:** Because it renders HTML/CSS, the ceiling is as high as any web UI — Tailwind, shadcn/ui, anything goes. This is its biggest visual advantage over native Rust GUIs.

**System tray:** Supported via Dioxus Desktop APIs.

**Global hotkey:** Still maturing in Dioxus's own abstraction layer. Current recommendation is to use Tauri's underlying `tao`/`wry` crates directly. The April 2025 survey found Dioxus "possibly usable without being constantly miserable" — the most cautiously positive assessment for a WebView-based native Rust GUI.

**Cross-platform rendering consistency:** The WebKitGTK version on Linux can behave differently from WebKit on macOS and WebView2 on Windows. CSS/JS compatibility varies. This is the same problem as Tauri, Electrobun, and Neutralino.

**Accessibility and IME:** The April 2025 survey found Dioxus actually works well here — "Narrator can see the text" and "IME works perfectly." This is because it inherits these from the system WebView.

**Production readiness for a personal tool:** Moderate. It works, but global hotkey and some desktop-specific APIs still require reaching through the abstraction to underlying Tauri crates. Not pre-1.0 in a way that suggests instability, but desktop-specific features are less mature than the web target.

**For this use case:** If you're comfortable with React-style thinking and HTML/CSS, Dioxus desktop is a viable path. But it's effectively Tauri written in Rust rather than a Rust+TypeScript split — same WebView tradeoffs apply.

---

### 5. floem

**Current version:** Pre-1.0 (no stable release)
**Stars:** 4,070 (smallest community of those reviewed)
**Built by:** The Lapce team (Lapce is an open-source code editor)

**What it is:** Fine-grained reactive UI framework for Rust with a signals-based reactivity model (similar to SolidJS on the web). Native renderer, not WebView.

**Visual quality:** Unknown at consumer scale — floem's primary real-world user is Lapce itself, which looks like a code editor, not a consumer app.

**System tray / Global hotkey:** Not found in documentation. Likely requires external crates like everything else, but no established pattern found.

**Accessibility:** The April 2025 survey found floem completely fails accessibility: "Windows Narrator can't see any of this text," IME "won't even start." The worst result of any framework tested.

**Production readiness for a personal tool:** No. The crate itself documents "occasional breaking changes on the way to v1." The community is small (4k stars vs 28k+ for egui/iced/Slint). No known production apps outside Lapce itself.

**Verdict:** Skip for now. Interesting architecture, not ready.

---

### Native Rust GUI: Summary Table

| Framework | Visual Quality | Tray | Global HK | Stable API | Recommended? |
|-----------|---------------|------|-----------|-----------|--------------|
| egui      | Functional, needs theming | External crate | External crate | No (breaking changes) | Yes, for tools |
| iced      | Good, modern feel | External crate | External crate | Pre-1.0 | Yes, for structured apps |
| Slint     | Best of group  | Built-in | Partial (OS-level unconfirmed) | Yes (1.x) | Yes, strongest overall |
| Dioxus    | HTML/CSS ceiling | Built-in | Via tao/wry | Moderate | Yes, if you want web stack |
| floem     | Unknown | Unknown | Unknown | No | No |

---

## Area 2: Electrobun (and TypeScript-only alternatives)

### What is Electrobun?

Electrobun (spelled "Electrobun," not "Elektrobun") is a TypeScript desktop framework built by the Blackboard team (https://blackboard.sh). It was announced in early 2026 as a complete framework for building "ultra fast, tiny, and cross-platform desktop apps with TypeScript."

**How it differs from Electron:**
- Uses **Bun** as the main process runtime instead of Node.js
- Uses the OS's **system WebView** instead of bundling Chromium — resulting in ~14MB compressed bundles vs ~150MB for Electron
- Native bindings written in C++, Objective-C++, and Zig (the Zig layer is shrinking as Bun's FFI matures)
- Built-in differential update system (bsdiff/zstd-based patches that are often just a few KB)

**How it differs from Tauri:**
- Everything is TypeScript — no Rust required for the native layer (unlike Tauri where you write Rust for the backend)
- Faster developer velocity for TypeScript/JavaScript developers
- The trade-off: you lose Tauri's memory safety and ecosystem depth

---

### Current version and release status

- **v1.0** launched **February 6, 2026**
- As of April 2026: **v1.16.0** (released March 15, 2026), with v1.17.x in beta
- The project is actively maintained with a roughly bi-weekly release cadence
- The README contains a reference to `BETA_RELEASE.md`, suggesting some internal beta tracking still exists, but the public version is labeled v1 stable

---

### How the "TypeScript only" claim works

The **main process** runs on **Bun** — a fast JavaScript/TypeScript runtime that includes:
- Built-in SQLite driver (`bun:sqlite`) — no npm install required
- Native file system APIs
- TypeScript execution without a separate compile step

The **webview** (your UI) is also TypeScript/HTML/CSS, bundled separately. Communication between main and webview happens through a **typed RPC channel** — you define function signatures in TypeScript and Electrobun generates the bidirectional interface. This is more structured than Electron's ipcMain/ipcRenderer.

The "native layer" (OS integration: tray, hotkeys, window management) is implemented in Zig/C++ and exposed as TypeScript APIs. As a developer, you never touch Zig — it is an implementation detail.

---

### Cross-platform support

| Platform | Architecture | Status | WebView |
|----------|-------------|--------|---------|
| macOS 14+ | x64, ARM64 | Stable (official) | WebKit |
| Windows 11+ | x64 | Stable (official) | Edge WebView2 |
| Ubuntu 22.04+ | x64, ARM64 | Stable (official) | WebKitGTK or CEF |
| Other Linux | x64, ARM64 | Community | gtk3 + webkit2gtk-4.1 |

**Important Linux constraint:** Official support is Ubuntu 22.04+. Other distributions that have `gtk3` and `webkit2gtk-4.1` get community-level support. Arch, Fedora, and similar modern distros will likely work, but are not officially tested.

**WebView consistency problem:** The system WebView varies by platform (WebKit, WebView2, WebKitGTK), and CSS/JS support is not identical. The documentation recommends bundling CEF (Chromium Embedded Framework) for Linux distributions to avoid WebKitGTK's limitations with advanced rendering. CEF increases bundle size but gives Chromium-consistent rendering.

---

### System tray support

Yes, supported. Electrobun has a Tray API. Cross-platform: macOS, Windows, and Linux (X11/GTK).

---

### Global hotkey support

**Yes, but with a critical Wayland limitation.**

- **Windows:** Uses `RegisterHotKey` — exclusive global shortcut registration.
- **Linux X11:** Uses `XGrabKey` — exclusive global shortcut registration.
- **Linux Wayland:** Does **not work**. Wayland's protocol does not allow applications to grab global keyboard input. This is a Wayland-level architectural limitation, not an Electrobun bug. No workaround exists in the current version.

**The Wayland problem in 2026:** As of early 2026, many Linux desktop environments (GNOME, KDE Plasma, recent Ubuntu) default to Wayland. Users running Wayland will not have global hotkeys unless the compositor specifically implements the `global-shortcuts` XDG portal protocol (which is compositor-specific and not universally supported).

**Implication for this app:** The system tray hotkey (the entire core workflow of the time tracker) would not work under Wayland. This is a significant practical concern for a Linux-first developer who runs KDE/GNOME on Wayland.

The Electrobun documentation recommends bundling CEF for Linux, which uses "pure X11 windows" — but this means running in X11 mode (Xwayland) on Wayland, not native Wayland.

---

### SQLite / local storage

**Excellent.** Bun includes `bun:sqlite` as a first-class built-in — zero npm dependencies. The recommended pattern:

```typescript
// Main process (Bun) — runs with full file system access
import { Database } from "bun:sqlite";
const db = new Database(Utils.paths.userData + "/tasks.db");
const stmt = db.prepare("SELECT * FROM tasks");

// Expose to webview via typed RPC
rpc.define("getTasks", () => stmt.all());
```

The webview (sandboxed) never touches SQLite directly — it calls the RPC. This is a cleaner separation than Electron's pattern of either exposing a Node.js API to the renderer or using IPC with manual typing.

---

### PDF / reporting story

No built-in support. Because the webview is HTML, you can use browser-based PDF libraries (jsPDF, pdfmake, Puppeteer via Bun) from the main process. Charting libraries (Chart.js, Recharts) work in the webview like any web page. This is actually more flexible than native Rust options — the HTML/CSS rendering pipeline is already there.

---

### Memory footprint and startup time

From the BetterStack benchmark (Electrobun vs alternatives, 2025):

| Metric | Electron | Tauri | Electrobun |
|--------|----------|-------|-----------|
| Bundle size | ~150 MB | ~25 MB | ~14 MB (compressed) / ~63 MB (uncompressed on macOS) |
| Startup time | 2–5 sec | ~500 ms | <50 ms |
| Memory at idle | 100–200 MB | 30–50 MB | 15–30 MB |

The <50ms startup claim is notable. The 15–30MB idle memory is roughly comparable to Tauri (which uses the same system WebView trick).

**The Hacker News thread caveat:** Commenters noted the uncompressed app size (~63MB on macOS) is larger than the 14MB marketing claim. The 14MB is the compressed download — the installed app is larger.

---

### Community size and ecosystem health

- GitHub: The project is at https://github.com/blackboardsh/electrobun
- Stars count was not retrieved directly; the HN thread for v1 had active discussion (2026-02)
- The team (Blackboard) is a small commercial entity building apps on their own framework — "eating their own cooking" is a positive signal
- Community-level Linux support (non-Ubuntu) means you are somewhat on your own for Fedora, Arch, etc.
- Documentation described as "developing" — better than early-stage projects but thinner than Electron's

---

### Known dealbreakers for this use case

1. **Wayland global hotkey does not work.** The core workflow of the time tracker (hotkey → quick-add window) fails silently or requires an X11/Xwayland session. This is a hard blocker on Wayland-first Linux setups.

2. **Linux officially means Ubuntu.** Other distros are community-supported. This is fine for most modern distros but is worth knowing.

3. **WebView rendering inconsistency.** The UI must be tested on all three platforms because WebKit, WebView2, and WebKitGTK behave differently. On native Rust frameworks, the renderer is the same everywhere.

4. **Young ecosystem.** TypeScript types reported as sometimes outdated. Documentation still developing.

---

### Other TypeScript-only desktop frameworks worth knowing

| Framework | How it works | Stars | Notes |
|-----------|-------------|-------|-------|
| **Neutralino.js** | System WebView, tiny C++ binary | Moderate | Tray supported; no global hotkey API found; poor bundler story; ~2MB uncompressed |
| **Buntralino** | Bun + Neutralino hybrid | Small | TypeScript-first; emerging; not production-ready |
| **NW.js** | Bundles Chromium like Electron | Moderate | Older; Electron is usually preferred over NW.js now |
| **Tauri** (with TS frontend) | System WebView, Rust backend | Very large (~90k) | Requires Rust for native layer; best-in-class for Wayland hotkeys via global-hotkey crate |

**Note on Tauri:** If the requirement is "TypeScript for the UI" (not "TypeScript for everything"), Tauri is far more mature than Electrobun. Its Rust layer has a global-hotkey crate that works via the `global-shortcuts` XDG portal on Wayland (with compositor support) and via X11 otherwise. The tradeoff is that you write the native backend in Rust.

---

## Synthesis for the time-tracker app

### If staying in Rust

**Slint** is the strongest recommendation for this use case:
- Best visual quality ceiling without WebView
- Version 1.x with stable API commitment
- Built-in system tray (including Wayland/dbus on Linux)
- Free license (royalty-free 2.0) for a personal tool
- Global hotkey: OS-level support needs verification but the framework is most likely to have a solution or get one soon
- Accessibility actually works

**egui** is a viable second choice if you want maximum simplicity and are comfortable theming:
- Easier to learn than Slint's DSL
- System tray and global hotkey via external crates — both well-established
- Faster iteration for a personal tool you do not need to maintain long-term

**iced** is worth watching but its pre-1.0 state and accessibility gaps make it slightly less compelling than Slint for a project that needs to just work.

### If going TypeScript-only

**Electrobun is not the right choice for this specific app** as long as it must work on Linux under Wayland (which is the developer's current system — running KDE on CachyOS based on Arch). The Wayland global hotkey limitation is a hard blocker for the core time-tracker workflow.

**Tauri with a Svelte/Tailwind frontend** is the TypeScript-friendly path that actually works on Wayland and is production-ready. You write the UI in Svelte/TypeScript and a thin Rust backend. SQLite works via the tauri-plugin-sql plugin. The global-hotkey crate has the best Wayland support of any framework researched here.

If Electrobun adds Wayland support in future versions, it becomes compelling — the DX and bundle size story is genuinely good.
