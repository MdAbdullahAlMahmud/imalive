# ImAlive

> Keep your machine awake — a small, cross-platform desktop app that prevents your display and system from sleeping while you're away.

Hand a long-running task to an AI agent (e.g. Claude Code) and step away, and your machine dims, locks, and sleeps after the idle timeout — interrupting work, pausing network activity, or forcing a re-auth when you return. **ImAlive** keeps the machine awake on demand, toggled from a clean GUI or the system tray.

Built with [Tauri](https://tauri.app/) (Rust backend + React/TypeScript frontend) — a ~5–10 MB bundle with native performance.

## Download

Grab the latest build for your OS from the [**Releases page**](https://github.com/MdAbdullahAlMahmud/imalive/releases/latest). All builds are **unsigned** open-source binaries, so each OS shows a one-time warning on first launch — the per-platform steps below clear it.

### 🍎 macOS

Download `ImAlive_<version>_universal.dmg` — a **universal binary** that runs natively on Apple Silicon (M-series) **and** Intel Macs. Open the DMG and drag **ImAlive** into **Applications**.

**First launch (Gatekeeper).** Do **one** of these, **once**:
- Right-click (Control-click) **ImAlive** in Applications → **Open** → **Open**, **or**
- Run: `xattr -dr com.apple.quarantine /Applications/ImAlive.app`

### 🐧 Linux

Two formats are provided:

- **AppImage** (any distro, no install): download `ImAlive_<version>_amd64.AppImage`, then:
  ```bash
  chmod +x ImAlive_*_amd64.AppImage
  ./ImAlive_*_amd64.AppImage
  ```
- **Debian/Ubuntu (.deb)**:
  ```bash
  sudo apt install ./ImAlive_<version>_amd64.deb
  ```
- **Fedora/RHEL (.rpm)**:
  ```bash
  sudo dnf install ./ImAlive-<version>-1.x86_64.rpm
  ```

> **Tray note:** the menu-bar/tray icon needs an AppIndicator host. On stock **GNOME**, install the [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/) to see it. On Linux, closing the window quits the app (rather than hiding to a tray that may not exist).

### 🪟 Windows

Download and run `ImAlive_<version>_x64-setup.exe` (or the `.msi`). Windows SmartScreen may show *"Windows protected your PC"* — click **More info → Run anyway** (one-time, because the app isn't code-signed).

> Want the warnings gone entirely? That requires paid signing certificates (Apple Developer ID for macOS, an Authenticode cert for Windows). The project ships unsigned for now.

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

### Build from source

Build a release bundle for your current platform:

```bash
pnpm tauri build
```

The installer/bundle is written to `src-tauri/target/release/bundle/`.

#### Universal macOS DMG

To produce a single `.dmg` that runs on both Apple Silicon and Intel Macs (what the Releases use):

```bash
rustup target add x86_64-apple-darwin   # one-time, adds the Intel target
pnpm tauri build --target universal-apple-darwin
```

The DMG lands in `src-tauri/target/universal-apple-darwin/release/bundle/dmg/`.

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
