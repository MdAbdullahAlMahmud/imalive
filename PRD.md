# PRD — ImAlive (Cross-Platform "Keep Awake" App)

## 1. Problem

When I hand a long-running task to an AI agent (e.g. Claude Code) and step
away, my machine dims and sleeps the screen after the idle timeout. Sleep can
interrupt the agent's work, pause network activity, or force a re-auth when I
return. I want a polished desktop app that keeps the machine awake — by
preventing idle sleep and, optionally, simulating mouse movement — that I can
start/stop and manage from a beautiful GUI on macOS, Linux, and Windows.

## 2. Goals

- Prevent display sleep / dimming and system idle sleep on demand.
- Cross-platform: macOS, Linux, Windows — one app, consistent UX.
- A beautiful, modern GUI to manage everything (not just a CLI).
- System tray / menu-bar presence for quick toggle without opening the window.
- Lightweight: small bundle, low RAM/CPU.
- Always restore normal power behavior on stop or crash.

## 3. Non-Goals

- Not an anti-AFK tool to deceive remote-work monitoring software.
- No cloud sync, accounts, or telemetry in v1.
- Not responsible for manual lid-close sleep on laptops.
- No mobile (iOS/Android) version.

## 4. Users

Primarily me — a developer running AI agents locally — plus anyone who wants a
clean cross-platform "keep awake" utility.

## 5. Tech Stack (decided)

- **Framework:** Tauri (Rust backend + web frontend).
- **Frontend:** React + TypeScript, styled for a modern look (Tailwind +
  shadcn/ui or similar), light/dark themes.
- **Backend:** Rust, with per-OS implementations behind a common trait.
- **Why:** ~5–10 MB bundle, ~50 MB RAM, native performance, full design
  freedom for a beautiful UI.

### Per-OS "keep awake" implementation

| OS | Prevent sleep | Mouse nudge |
|----|---------------|-------------|
| macOS | IOKit power assertions (`IOPMAssertionCreateWithName`, `PreventUserIdleDisplaySleep`) — same mechanism as `caffeinate` | CoreGraphics `CGEventCreateMouseEvent` / `CGWarpMouseCursorPosition` |
| Windows | `SetThreadExecutionState(ES_CONTINUOUS \| ES_DISPLAY_REQUIRED \| ES_SYSTEM_REQUIRED)` | `SendInput` (mouse_event) |
| Linux | `systemd-inhibit` / `org.freedesktop.ScreenSaver` D-Bus inhibit; fallback `xdg-screensaver` | X11: `XTestFakeMotionEvent`; Wayland: limited — rely on inhibit |

> Candidate crates to evaluate: `keepawake`, `nosleep`, or hand-rolled FFI per
> OS. Mouse simulation: `enigo` (cross-platform input). Wayland mouse movement
> is restricted by design — on Wayland we prefer the inhibit path and surface a
> note in the UI.

## 6. Requirements

### Functional

| ID | Requirement | Priority |
|----|-------------|----------|
| F1 | Toggle keep-awake on/off from the main window | Must |
| F2 | Prevent display sleep | Must |
| F3 | Prevent system idle sleep | Must |
| F4 | System tray / menu-bar icon with quick toggle + status | Must |
| F5 | Optional duration / timer ("keep awake for 2h, then stop") | Must |
| F6 | Optional mouse nudge mode (periodic 1px move) with interval | Must |
| F7 | Live status: state, elapsed time, active assertions, mode | Must |
| F8 | Profiles (named presets, e.g. "Agent run", "Presentation") | Should |
| F9 | Schedules (e.g. active 9am–6pm weekdays) | Should |
| F10 | Auto-detect activity — keep awake while a chosen process runs | Should |
| F11 | Launch at login (per-OS) | Should |
| F12 | Light/dark theme + clean, modern visual design | Must |
| F13 | Restore normal power behavior on quit/crash | Must |
| F14 | Persist settings/profiles locally (JSON in app config dir) | Must |

### Non-Functional

- Bundle ≤ ~15 MB per platform; RAM ≤ ~80 MB; idle CPU < 1%.
- No admin/root required on any OS for core functionality.
- Crash-safe: assertions/inhibitors released on unexpected exit.
- Accessible: keyboard-navigable, sensible contrast in both themes.
- Signed/notarized builds where feasible (macOS notarization, etc.) — later.

## 7. UX / Screens

Tabbed layout with a persistent status header.

- **Status** — big toggle, current mode, elapsed timer, active assertions,
  quick duration picker.
- **Schedule** — define time windows when keep-awake auto-activates.
- **Profiles** — named presets bundling mode + duration + nudge settings.
- **Settings** — theme, launch-at-login, default mode, tray behavior,
  per-OS notes (e.g. Wayland limitation).

Tray/menu-bar menu: Toggle, current status, pick profile, open window, quit.

```
Tabs: Status | Schedule | Profiles | Settings

[ Keep Awake ]  ● ON
Mode: ( ) Prevent sleep only   (•) Prevent sleep + nudge mouse
Nudge every: [ 60s ▾ ]   Duration: [ Until I stop ▾ ]
Elapsed: 01:23:45
[x] Display sleep blocked   [x] System sleep blocked
```

## 8. Architecture

- **Core (Rust):** `KeepAwake` trait with `start(mode)`, `stop()`,
  `status()`; platform modules (`macos.rs`, `windows.rs`, `linux.rs`) selected
  via `cfg`. A `Nudger` task (async timer) for mouse movement.
- **State manager:** owns current session, timer, schedule evaluation; emits
  events to the frontend via Tauri events.
- **Frontend (React):** subscribes to state events, renders tabs, invokes
  Tauri commands (`start`, `stop`, `set_profile`, `save_settings`, …).
- **Persistence:** settings + profiles in the OS app-config dir.

## 9. Milestones

1. **M1 — Core engine:** Rust keep-awake working on all 3 OSes (prevent sleep
   only), minimal toggle UI. ← proves the hard part.
2. **M2 — Polished single-screen app:** Status tab, tray icon, duration timer,
   theme, persistence.
3. **M3 — Mouse nudge:** optional nudge mode + interval (with Wayland note).
4. **M4 — Full-featured:** Profiles, Schedules, launch-at-login.
5. **M5 — Auto-detect activity:** keep awake while a chosen process runs.
6. **M6 — Packaging:** installers per OS, icons, (later) signing/notarization.

## 10. Success Criteria

- Screen/system does not sleep during a multi-hour agent run on each OS.
- Toggling off restores normal sleep within seconds.
- Survives an overnight run unattended; releases assertions on quit/crash.
- The UI is clean enough that I enjoy opening it.

## 11. Open Questions

1. Frontend styling kit — Tailwind + shadcn/ui (recommended), or another?
2. Build/test order — which OS is primary for first dev pass (macOS)?
3. Auto-detect (F10): match by process name, or watch CPU/network activity?
4. Do you want signed/notarized release builds in scope, or dev builds first?
```
