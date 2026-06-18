# ImAlive — Task List

Tracking the build against the milestones in [PRD.md](./PRD.md).

## M1 — Core Engine ✅ DONE
- [x] Install Rust toolchain (rustup + stable rustc/cargo 1.96)
- [x] Scaffold Tauri v2 + React/TypeScript/Vite project
- [x] Rust keep-awake core engine — worker thread owns the platform guard via
      the `keepawake` crate (IOKit / SetThreadExecutionState / D-Bus inhibit);
      Tauri `start_keep_awake` / `stop_keep_awake` / `get_status` commands
- [x] Minimal toggle UI — power toggle wired to Tauri commands, live elapsed
      clock, display/idle option checkboxes
- [x] Build & verify app runs — built clean (0 warnings); verified a real OS
      assertion appears in `pmset -g assertions` while active and releases on
      stop; app binary boots without panic

Run it: `pnpm tauri dev` (or build the release bundle with `pnpm tauri build`).

## M2 — Polished single-screen app
- [ ] Status tab, tray/menu-bar icon, duration timer, light/dark theme, persistence

## M3 — Mouse nudge
- [ ] Optional nudge mode + interval (with Wayland limitation note)

## M4 — Full-featured
- [ ] Profiles, Schedules, launch-at-login

## M5 — Auto-detect activity
- [ ] Keep awake while a chosen process runs

## M6 — Packaging
- [ ] Per-OS installers, icons, (later) signing/notarization
