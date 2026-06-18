# ImAlive

> Keep your machine awake — a small, cross-platform desktop app that prevents your display and system from sleeping while you're away.

Hand a long-running task to an AI agent (e.g. Claude Code) and step away, and your machine dims, locks, and sleeps after the idle timeout — interrupting work, pausing network activity, or forcing a re-auth when you return. **ImAlive** keeps the machine awake on demand, toggled from a clean GUI or the system tray.

Built with [Tauri](https://tauri.app/) (Rust backend + React/TypeScript frontend) — a ~5–10 MB bundle with native performance.

## Features

- **Toggle keep-awake** on/off from the main window or the tray icon
- **Prevent display sleep** and **prevent system idle sleep** — independently configurable
- **Live status** — active/inactive state and elapsed timer
- **System tray** presence for quick toggling without opening the window
- **Crash-safe** — power assertions are released on stop and on exit, restoring normal sleep behavior
- **No admin/root** required for core functionality

Under the hood, sleep prevention uses the native mechanism on each OS (IOKit power assertions on macOS, `SetThreadExecutionState` on Windows, and D-Bus/systemd inhibitors on Linux) via the [`keepawake`](https://crates.io/crates/keepawake) crate.

## Status

ImAlive is in early development. The core engine (keep-awake on macOS/Windows/Linux), status reporting, and tray integration are working. Roadmap items — optional mouse-nudge mode, duration timers, profiles, schedules, and launch-at-login — are tracked in [`PRD.md`](./PRD.md) and [`TASKS.md`](./TASKS.md).

## Getting started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+ and [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- Platform build dependencies for Tauri — see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

### Develop

```bash
pnpm install
pnpm tauri dev
```

### Build a release bundle

```bash
pnpm tauri build
```

The installer/bundle for your platform is written to `src-tauri/target/release/bundle/`.

## Project structure

```
src/                 React + TypeScript frontend (UI, status, controls)
src-tauri/           Rust backend (Tauri app)
  src/lib.rs         Keep-awake engine, Tauri commands, tray, status worker
  tauri.conf.json    App identifier, window, and bundle config
PRD.md               Product requirements & full roadmap
TASKS.md             Task breakdown
```

## Contributing

Issues and pull requests are welcome. For larger changes, please open an issue first to discuss the approach. Run `pnpm tauri dev` to develop locally, and `cargo test` inside `src-tauri/` to run the Rust tests.

## License

[MIT](./LICENSE) © Abdullah
