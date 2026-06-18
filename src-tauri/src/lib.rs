//! ImAlive — cross-platform "keep awake" engine.
//!
//! A dedicated worker thread owns the platform keep-awake guard
//! (IOKit assertion on macOS, SetThreadExecutionState on Windows,
//! D-Bus/systemd inhibit on Linux). Tauri commands talk to it over a
//! channel, so the (possibly `!Send`) guard never crosses threads.

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};

/// Snapshot of the engine, shared with the frontend.
#[derive(Clone, Serialize)]
struct Status {
    /// Whether keep-awake is currently active.
    active: bool,
    /// Block the display from sleeping/dimming.
    display: bool,
    /// Block system idle sleep.
    idle: bool,
    /// Epoch milliseconds when the current session started (None if inactive).
    started_at_ms: Option<u64>,
}

/// Inactive by default, but both sleep types are pre-selected — so a one-tap
/// start (window or menu bar) keeps the machine fully awake.
impl Default for Status {
    fn default() -> Self {
        Self {
            active: false,
            display: true,
            idle: true,
            started_at_ms: None,
        }
    }
}

/// Commands sent to the worker thread. Each carries a reply channel so the
/// invoking command can surface success/failure synchronously.
enum Cmd {
    Start {
        display: bool,
        idle: bool,
        reply: Sender<Result<Status, String>>,
    },
    Stop {
        reply: Sender<Result<Status, String>>,
    },
}

/// Managed Tauri state: a handle to the worker plus the shared status.
struct Engine {
    tx: Mutex<Sender<Cmd>>,
    status: Arc<Mutex<Status>>,
}

/// Handles to the menu-bar UI we mutate as state changes (so the tray's
/// label/tooltip/title track on/off without rebuilding the menu).
struct TrayUi {
    toggle: tauri::menu::MenuItem<tauri::Wry>,
    tray: tauri::tray::TrayIcon<tauri::Wry>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Spawn the worker thread that owns the keep-awake guard for its lifetime.
// `guard` is held only for its `Drop` (releasing the OS assertion on
// reassign/clear); it is intentionally never read.
#[allow(unused_assignments, unused_variables)]
fn spawn_worker(status: Arc<Mutex<Status>>) -> Sender<Cmd> {
    let (tx, rx) = channel::<Cmd>();

    thread::spawn(move || {
        // Holds the active assertion. We never read `guard` — it exists purely
        // for its `Drop`, which releases the OS assertion when reassigned/cleared.
        let mut guard: Option<keepawake::KeepAwake> = None;

        for cmd in rx {
            match cmd {
                Cmd::Start {
                    display,
                    idle,
                    reply,
                } => {
                    // Release any existing assertion before creating a new one.
                    guard = None;

                    let created = keepawake::Builder::default()
                        .display(display)
                        .idle(idle)
                        .reason("ImAlive is keeping this machine awake")
                        .app_name("ImAlive")
                        .app_reverse_domain("com.mkrlabs.imalive")
                        .create();

                    match created {
                        Ok(k) => {
                            guard = Some(k);
                            let snapshot = {
                                let mut s = status.lock().unwrap();
                                s.active = true;
                                s.display = display;
                                s.idle = idle;
                                s.started_at_ms = Some(now_ms());
                                s.clone()
                            };
                            let _ = reply.send(Ok(snapshot));
                        }
                        Err(e) => {
                            let _ = reply.send(Err(e.to_string()));
                        }
                    }
                }
                Cmd::Stop { reply } => {
                    // Drop the guard → assertion released.
                    guard = None;
                    let snapshot = {
                        let mut s = status.lock().unwrap();
                        s.active = false;
                        s.started_at_ms = None;
                        s.clone()
                    };
                    let _ = reply.send(Ok(snapshot));
                }
            }
        }
    });

    tx
}

/// Send a Start command to the worker and wait for the result.
fn engine_start(engine: &Engine, display: bool, idle: bool) -> Result<Status, String> {
    let (reply_tx, reply_rx) = channel();
    engine
        .tx
        .lock()
        .unwrap()
        .send(Cmd::Start {
            display,
            idle,
            reply: reply_tx,
        })
        .map_err(|e| e.to_string())?;
    reply_rx.recv().map_err(|e| e.to_string())?
}

/// Send a Stop command to the worker and wait for the result.
fn engine_stop(engine: &Engine) -> Result<Status, String> {
    let (reply_tx, reply_rx) = channel();
    engine
        .tx
        .lock()
        .unwrap()
        .send(Cmd::Stop { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.recv().map_err(|e| e.to_string())?
}

/// Push the latest status everywhere the UI lives: the menu-bar icon (toggle
/// label, tooltip, title) and the window (via a `status-changed` event), so
/// the two never disagree no matter which one initiated the change.
fn broadcast_status(app: &AppHandle, status: &Status) {
    if let Some(ui) = app.try_state::<TrayUi>() {
        let _ = ui
            .toggle
            .set_text(if status.active { "Turn Off" } else { "Turn On" });
        let _ = ui.tray.set_tooltip(Some(if status.active {
            "ImAlive — On"
        } else {
            "ImAlive — Off"
        }));
        let _ = ui
            .tray
            .set_title(if status.active { Some("ON") } else { None });
    }
    let _ = app.emit("status-changed", status.clone());
}

#[tauri::command]
fn start_keep_awake(
    display: bool,
    idle: bool,
    app: AppHandle,
    state: State<Engine>,
) -> Result<Status, String> {
    let status = engine_start(&state, display, idle)?;
    broadcast_status(&app, &status);
    Ok(status)
}

#[tauri::command]
fn stop_keep_awake(app: AppHandle, state: State<Engine>) -> Result<Status, String> {
    let status = engine_stop(&state)?;
    broadcast_status(&app, &status);
    Ok(status)
}

/// Persist sleep-blocking preferences while inactive, so the menu-bar toggle
/// starts with the same options the user picked in the window.
#[tauri::command]
fn set_options(display: bool, idle: bool, state: State<Engine>) -> Status {
    let mut s = state.status.lock().unwrap();
    if !s.active {
        s.display = display;
        s.idle = idle;
    }
    s.clone()
}

#[tauri::command]
fn get_status(state: State<Engine>) -> Status {
    state.status.lock().unwrap().clone()
}

/// Show, unhide, and focus the main window.
fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let status = Arc::new(Mutex::new(Status::default()));
    let tx = spawn_worker(status.clone());
    let engine = Engine {
        tx: Mutex::new(tx),
        status,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            start_keep_awake,
            stop_keep_awake,
            set_options,
            get_status
        ])
        .setup(|app| {
            // --- Menu-bar (status bar) icon + menu ---
            let toggle = MenuItem::with_id(app, "toggle", "Turn On", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show ImAlive", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit ImAlive", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&toggle, &show, &PredefinedMenuItem::separator(app)?, &quit],
            )?;

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(tauri::include_image!("icons/tray.png"))
                // macOS recolors a template icon for light/dark menu bars.
                .icon_as_template(true)
                .tooltip("ImAlive — Off")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => {
                        let state = app.state::<Engine>();
                        let current = state.status.lock().unwrap().clone();
                        let result = if current.active {
                            engine_stop(&state)
                        } else {
                            engine_start(&state, current.display, current.idle)
                        };
                        if let Ok(status) = result {
                            broadcast_status(app, &status);
                        }
                    }
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Left-click toggles the window's visibility.
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                show_main_window(app);
                            }
                        }
                    }
                })
                .build(app)?;

            app.manage(TrayUi { toggle, tray });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    // macOS/Windows have a reliable tray, so closing the window
                    // hides it there instead of quitting. Many Linux desktops
                    // (e.g. stock GNOME) show no tray — hiding would make the
                    // window unreachable, so there we let the close proceed.
                    #[cfg(not(target_os = "linux"))]
                    {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = &api;
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Clicking the Dock icon while the window is hidden reopens it.
            if let tauri::RunEvent::Reopen { .. } = event {
                show_main_window(app);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// True if an "ImAlive" power assertion is currently registered with the OS.
    #[cfg(target_os = "macos")]
    fn os_has_assertion() -> bool {
        std::process::Command::new("pmset")
            .args(["-g", "assertions"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("ImAlive"))
            .unwrap_or(false)
    }

    /// Drives the same worker code path the Tauri commands use: Start → Stop,
    /// asserting the status transitions and (on macOS) the real OS assertion
    /// being held while active and released after stop.
    #[test]
    fn start_then_stop_lifecycle() {
        let status = Arc::new(Mutex::new(Status::default()));
        let tx = spawn_worker(status.clone());

        // --- START (block both display + idle) ---
        let (reply_tx, reply_rx) = channel();
        tx.send(Cmd::Start {
            display: true,
            idle: true,
            reply: reply_tx,
        })
        .unwrap();
        let started = reply_rx.recv().unwrap().expect("start should succeed");
        assert!(started.active, "status should be active after start");
        assert!(started.display && started.idle, "both options recorded");
        assert!(started.started_at_ms.is_some(), "timestamp set on start");
        assert!(status.lock().unwrap().active, "shared status reflects active");

        #[cfg(target_os = "macos")]
        {
            std::thread::sleep(Duration::from_millis(300));
            assert!(
                os_has_assertion(),
                "OS should hold an ImAlive assertion while active"
            );
        }

        // --- STOP ---
        let (reply_tx, reply_rx) = channel();
        tx.send(Cmd::Stop { reply: reply_tx }).unwrap();
        let stopped = reply_rx.recv().unwrap().expect("stop should succeed");
        assert!(!stopped.active, "status inactive after stop");
        assert!(stopped.started_at_ms.is_none(), "timestamp cleared on stop");
        assert!(!status.lock().unwrap().active, "shared status reflects inactive");

        #[cfg(target_os = "macos")]
        {
            std::thread::sleep(Duration::from_millis(300));
            assert!(
                !os_has_assertion(),
                "OS assertion should be released after stop"
            );
        }
    }

    /// Restarting while already active should not error and should keep exactly
    /// one assertion (the old guard is dropped before the new one is created).
    #[test]
    fn restart_while_active_is_clean() {
        let status = Arc::new(Mutex::new(Status::default()));
        let tx = spawn_worker(status.clone());

        for _ in 0..2 {
            let (reply_tx, reply_rx) = channel();
            tx.send(Cmd::Start {
                display: true,
                idle: false,
                reply: reply_tx,
            })
            .unwrap();
            assert!(reply_rx.recv().unwrap().is_ok(), "repeated start ok");
        }
        assert!(status.lock().unwrap().active);

        let (reply_tx, reply_rx) = channel();
        tx.send(Cmd::Stop { reply: reply_tx }).unwrap();
        reply_rx.recv().unwrap().expect("stop ok");
    }
}
