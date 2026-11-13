# Tracker

A local-first desktop task and time tracker written in Rust with [Iced](https://iced.rs/). It combines task prioritization with a lightweight daily time log.

## Features

- Organize tasks by urgency and importance using an Eisenhower-style priority order.
- Start, pause, resume, and stop a timer for a task.
- Add notes to a running session and record task status when stopping it.
- Review entries by day, including gaps between tracked sessions; edit times and notes.
- Store task and time-entry data locally as JSON in the current user's `home directory under `.tracker/data.json.

## Build and run

Requires a stable Rust toolchain with Cargo.

```sh
cargo run
```

To build a release binary:

```sh
cargo build --release
```

The release workflow builds Linux, Windows, and macOS binaries when a version tag is pushed. See [Releases](https://github.com/Rodhor/Tracker/releases) for published builds.
