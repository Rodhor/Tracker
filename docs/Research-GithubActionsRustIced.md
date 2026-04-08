# Research: GitHub Actions for Rust/iced Cross-Platform Release Builds

**Date:** 2026-04-08
**Purpose:** Gather enough verified information to write a complete, working `.github/workflows/release.yml` for a Rust desktop app using iced 0.14.

---

## 1. Toolchain Action — What to Use

### actions-rs/toolchain — DEPRECATED

Do not use `actions-rs/toolchain@v1`. It was deprecated in October 2023. The underlying Node.js version (12) it relied on was removed from GitHub Actions, causing failures. It is no longer maintained.

### Recommended replacements (pick one)

| Action | Author | Notes |
|--------|--------|-------|
| `dtolnay/rust-toolchain@stable` | David Tolnay (dtolnay) | Community standard. Extremely concise. Active. |
| `actions-rust-lang/setup-rust-toolchain@v1` | Rust lang org | Extends dtolnay's; adds problem matchers. Good for CI checks. |
| `hecrj/setup-rust-action@v2` | iced's own author (hecrj) | Used in iced's own workflows. Still active. |

**For a release workflow, `dtolnay/rust-toolchain@stable` is the best choice.** It is actively maintained, trusted, and minimal. The iced repo itself uses `hecrj/setup-rust-action@v2` (written by the same author as iced), which is also fine but less universally known.

#### dtolnay usage

```yaml
- uses: dtolnay/rust-toolchain@stable
  # No further config needed for a basic stable release build.
  # Optional extras:
  with:
    targets: aarch64-apple-darwin   # only if cross-compiling
    components: clippy,rustfmt      # only if needed in CI check jobs
```

The `@stable` rev IS the toolchain specifier — do not set `toolchain: stable` redundantly.

---

## 2. Caching — Swatinem/rust-cache

The standard Rust caching action is `Swatinem/rust-cache@v2`. It:

- Caches `~/.cargo` (registry, index, git sources) and `./target` (dep build artifacts)
- Automatically generates cache keys from `Cargo.lock`, `Cargo.toml`, and toolchain version
- Handles the macOS cache-corruption bug in `actions/cache` automatically
- Cleans stale artifacts before saving, keeping cache sizes sane

```yaml
- uses: Swatinem/rust-cache@v2
  # Place this AFTER the toolchain step, BEFORE cargo build.
  # No config needed for basic use.
```

For release builds with a matrix, add `key` variant so each target gets its own cache:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    key: ${{ matrix.target }}
```

---

## 3. Platform Runners and Targets

### Current runner labels (as of April 2026)

| Platform | `runs-on` | Default arch | Notes |
|----------|-----------|--------------|-------|
| Linux | `ubuntu-latest` | x86_64 | Currently Ubuntu 24.04 |
| Windows | `windows-latest` | x86_64 | Currently Windows Server 2025 |
| macOS | `macos-latest` | arm64 (Apple Silicon) | Changed to arm64 with macos-14; will use macos-15 from August 2025 |

**Important macOS note:** `macos-latest` has pointed to arm64 (Apple Silicon) since macos-14 (2024). If you want to ship an Intel (x86_64) macOS binary, you must either use `macos-13` explicitly (still x86_64) or cross-compile from arm64. For an Apple Silicon binary, `macos-latest` is correct.

### Rust target triples

| Binary target | `runs-on` | Rust target triple |
|---------------|-----------|-------------------|
| Linux x86_64 | `ubuntu-latest` | `x86_64-unknown-linux-gnu` |
| Windows x86_64 | `windows-latest` | `x86_64-pc-windows-msvc` |
| macOS Apple Silicon | `macos-latest` (arm64) | `aarch64-apple-darwin` |
| macOS Intel (optional) | `macos-13` | `x86_64-apple-darwin` |

---

## 4. Linux: System Dependencies for iced

Linux is the only platform that requires installing system packages before `cargo build`. iced needs display-server and keyboard libraries to compile (not just to run — the build scripts link against them).

**From iced's own workflows** (`build.yml`, `test.yml`):

```yaml
- name: Install Linux dependencies
  if: runner.os == 'Linux'
  run: |
    export DEBIAN_FRONTEND=noninteractive
    sudo apt-get -qq update
    sudo apt-get install -y libxkbcommon-dev libgtk-3-dev
```

**What each package provides:**

| Package | Why needed |
|---------|-----------|
| `libxkbcommon-dev` | Keyboard handling; required by winit (iced's windowing layer) |
| `libgtk-3-dev` | Required if using any GTK-backed iced features (file dialogs, tray, etc.) |

**Do you need a display server for building (not running)?** No. `cargo build` compiles to a binary — it does not execute the GUI. `xvfb` or a virtual framebuffer is only needed if you run the app or run GUI tests. For a release build job that just compiles and packages, no display setup is needed.

**Is `pkg-config` needed?** It is pre-installed on `ubuntu-latest`. No explicit install step is required unless a dependency fails with a missing `pkg-config` error (rare on GitHub-hosted runners).

**Vulkan/wgpu on Linux:** wgpu (iced's default renderer) uses a software fallback (llvmpipe) for rendering. The build itself does not need a GPU or Vulkan ICD installed — that is a runtime concern. No extra GPU-related apt packages are needed for compilation.

---

## 5. Windows: Key Gotchas

### Console window appearing

By default, a Rust binary on Windows opens a console window alongside the GUI. To suppress it, add this to `src/main.rs`:

```rust
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
```

The `cfg_attr` form is better than the unconditional `#![windows_subsystem = "windows"]` because it doesn't break compilation on Linux/macOS. Do this in source, not in the workflow.

**iced's own build.yml does it differently** — it uses `sed` to inject `#![windows_subsystem = "windows"]` at CI time. That works but is fragile; the `cfg_attr` approach in source is cleaner.

### Static CRT linkage (optional but recommended)

iced's official build does this for Windows:

```yaml
- name: Enable static CRT linkage
  if: runner.os == 'Windows'
  shell: bash
  run: |
    mkdir -p .cargo
    echo '[target.x86_64-pc-windows-msvc]' >> .cargo/config.toml
    echo 'rustflags = ["-Ctarget-feature=+crt-static"]' >> .cargo/config.toml
```

This makes the binary self-contained — it does not require the Visual C++ redistributable to be installed on the user's machine. For a GUI desktop app distributed to end users this is strongly recommended.

**Note:** The file is `.cargo/config.toml` (not `config` without extension — the old name still works but `.toml` is canonical).

### No extra apt/package installs

Windows runners have the MSVC toolchain pre-installed. No extra setup beyond the toolchain action is needed.

---

## 6. macOS: Key Notes

- No system package installs needed. macOS runners have all required frameworks.
- Set `MACOSX_DEPLOYMENT_TARGET` if you want to support older macOS versions:

  ```yaml
  env:
    MACOSX_DEPLOYMENT_TARGET: "11.0"   # Minimum macOS 11 Big Sur
  ```

  iced 0.14 with wgpu requires Metal, which is macOS 10.13+. A deployment target of `11.0` is a reasonable floor for modern iced apps.

- After building, mark the binary executable (iced's workflow does this):

  ```yaml
  - run: chmod +x target/release/your-app
  ```

---

## 7. Uploading Release Artifacts

Two well-maintained approaches:

### Approach A: taiki-e/upload-rust-binary-action (recommended for release tags)

This action combines building + archiving + uploading into one step. It works well with `taiki-e/create-gh-release-action`.

```yaml
- uses: taiki-e/upload-rust-binary-action@v1
  with:
    bin: your-app-name          # binary name without extension
    target: ${{ matrix.target }}
    tar: unix                   # .tar.gz for Linux and macOS
    zip: windows                # .zip for Windows
    checksum: sha256
```

### Approach B: actions/upload-artifact + softprops/action-gh-release (more control)

Better when you need custom packaging steps (e.g., bundling assets alongside the binary).

```yaml
# Build step produces the binary, then:
- uses: actions/upload-artifact@v4
  with:
    name: ${{ matrix.target }}
    path: target/release/your-app   # or .exe on Windows

# In a separate gather job that runs after all build jobs:
- uses: actions/download-artifact@v4
  with:
    path: artifacts/
- uses: softprops/action-gh-release@v2
  with:
    files: artifacts/**/*
```

---

## 8. Complete Annotated Release Workflow

This is a working skeleton for the tracker app. Replace `tracker` with the actual binary name.

```yaml
name: Release

# Trigger on version tags: v0.1.0, v1.2.3, etc.
on:
  push:
    tags:
      - "v[0-9]+.*"

permissions:
  contents: write  # Required to create and upload to GitHub Releases

jobs:
  # ─── Create the GitHub Release ─────────────────────────────────────────────
  create-release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/create-gh-release-action@v1
        with:
          # Optional: reads release notes from CHANGELOG.md if you have one
          # changelog: CHANGELOG.md
          token: ${{ secrets.GITHUB_TOKEN }}

  # ─── Build and Upload Binaries ─────────────────────────────────────────────
  upload-assets:
    needs: create-release
    strategy:
      fail-fast: false   # Don't cancel other platform builds if one fails
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-apple-darwin
            os: macos-latest            # arm64 Apple Silicon since macos-14
          - target: x86_64-apple-darwin
            os: macos-13                # last Intel macOS runner; optional
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      # ── Toolchain ─────────────────────────────────────────────────────────
      - uses: dtolnay/rust-toolchain@stable

      # ── Cargo cache (speeds up repeat builds significantly) ───────────────
      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}

      # ── Linux: install system libraries required to compile iced/winit ────
      - name: Install Linux dependencies
        if: matrix.os == 'ubuntu-latest'
        run: |
          export DEBIAN_FRONTEND=noninteractive
          sudo apt-get -qq update
          sudo apt-get install -y libxkbcommon-dev libgtk-3-dev

      # ── Windows: static CRT so the binary needs no MSVC redistributable ──
      - name: Enable static CRT linkage (Windows)
        if: matrix.os == 'windows-latest'
        shell: bash
        run: |
          mkdir -p .cargo
          printf '[target.x86_64-pc-windows-msvc]\nrustflags = ["-Ctarget-feature=+crt-static"]\n' \
            >> .cargo/config.toml

      # ── macOS: set minimum deployment target ─────────────────────────────
      # (iced uses Metal which requires 10.13; 11.0 is a reasonable floor)
      - name: Set macOS deployment target
        if: runner.os == 'macOS'
        run: echo "MACOSX_DEPLOYMENT_TARGET=11.0" >> $GITHUB_ENV

      # ── Build and upload to the release ───────────────────────────────────
      - uses: taiki-e/upload-rust-binary-action@v1
        with:
          bin: tracker
          target: ${{ matrix.target }}
          tar: unix       # .tar.gz on Linux and macOS
          zip: windows    # .zip on Windows
          checksum: sha256
          token: ${{ secrets.GITHUB_TOKEN }}
```

---

## 9. CI Check Workflow (Separate from Release)

Keep a separate workflow for PR/push checks that runs on every commit. This avoids confusing compilation checks with the release process.

```yaml
name: CI

on:
  push:
    branches: ["master", "main"]
  pull_request:

jobs:
  check:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install Linux dependencies
        if: runner.os == 'Linux'
        run: |
          export DEBIAN_FRONTEND=noninteractive
          sudo apt-get -qq update
          sudo apt-get install -y libxkbcommon-dev libgtk-3-dev

      - name: cargo check
        run: cargo check --all-features

      - name: cargo clippy
        run: cargo clippy -- -D warnings
```

---

## 10. Summary of Key Decisions

| Question | Answer |
|----------|--------|
| `actions-rs/toolchain` still usable? | No — deprecated Oct 2023, broken on current runners |
| Best toolchain action? | `dtolnay/rust-toolchain@stable` |
| Best cache action? | `Swatinem/rust-cache@v2` |
| `macos-latest` architecture? | arm64 (Apple Silicon) since macos-14; use `macos-13` for Intel |
| Linux display server needed for `cargo build`? | No — only needed if running/testing the GUI |
| Linux apt packages for iced? | `libxkbcommon-dev libgtk-3-dev` (from iced's own CI) |
| Windows console window? | Suppress in source with `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]` |
| Windows CRT static linkage? | Yes — add `-Ctarget-feature=+crt-static` in `.cargo/config.toml` |
| Best release upload action? | `taiki-e/upload-rust-binary-action@v1` (simplest) or `softprops/action-gh-release@v2` (more control) |

---

## Sources Consulted

- iced-rs/iced `build.yml` and `test.yml` — actual workflows from the iced repo itself
- `taiki-e/upload-rust-binary-action` README — multi-platform release workflow patterns
- `dtolnay/rust-toolchain` README — current canonical usage
- `Swatinem/rust-cache` — Rust community standard caching
- GitHub Actions runner images issue #12520 — macos-latest architecture timeline
- Rust forum thread on actions-rs deprecation
