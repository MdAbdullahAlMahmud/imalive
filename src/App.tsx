import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

type Status = {
  active: boolean;
  display: boolean;
  idle: boolean;
  started_at_ms: number | null;
};

const ZERO: Status = {
  active: false,
  display: true,
  idle: true,
  started_at_ms: null,
};

function formatElapsed(ms: number): string {
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
}

function App() {
  const [status, setStatus] = useState<Status>(ZERO);
  const [elapsed, setElapsed] = useState("00:00:00");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const timer = useRef<number | null>(null);

  // Load initial status (in case the engine was already running).
  useEffect(() => {
    invoke<Status>("get_status").then(setStatus).catch(() => {});
  }, []);

  // Stay in sync when the menu-bar (tray) toggles keep-awake.
  useEffect(() => {
    const unlisten = listen<Status>("status-changed", (e) =>
      setStatus(e.payload)
    );
    return () => {
      unlisten.then((off) => off());
    };
  }, []);

  // Tick the elapsed clock while active.
  useEffect(() => {
    if (status.active && status.started_at_ms != null) {
      const start = status.started_at_ms;
      const tick = () => setElapsed(formatElapsed(Date.now() - start));
      tick();
      timer.current = window.setInterval(tick, 1000);
    } else {
      setElapsed("00:00:00");
    }
    return () => {
      if (timer.current != null) {
        window.clearInterval(timer.current);
        timer.current = null;
      }
    };
  }, [status.active, status.started_at_ms]);

  async function toggle() {
    setBusy(true);
    setError(null);
    try {
      const next = status.active
        ? await invoke<Status>("stop_keep_awake")
        : await invoke<Status>("start_keep_awake", {
            display: status.display,
            idle: status.idle,
          });
      setStatus(next);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  function setOption(key: "display" | "idle", value: boolean) {
    // Options only editable while inactive.
    if (status.active) return;
    const next = { ...status, [key]: value };
    setStatus(next);
    // Persist so the menu-bar toggle uses the same options.
    invoke("set_options", {
      display: next.display,
      idle: next.idle,
    }).catch(() => {});
  }

  const active = status.active;

  return (
    <main className="app">
      <header className="header">
        <div className={`pulse ${active ? "on" : ""}`} />
        <h1>ImAlive</h1>
        <p className="subtitle">Keep this machine awake</p>
      </header>

      <button
        className={`power ${active ? "active" : ""}`}
        onClick={toggle}
        disabled={busy}
        aria-pressed={active}
      >
        <span className="power-icon">⏻</span>
        <span className="power-label">
          {busy ? "…" : active ? "Awake — ON" : "Turn On"}
        </span>
      </button>

      <div className="clock">
        <span className="clock-label">{active ? "Active for" : "Idle"}</span>
        <span className="clock-value">{elapsed}</span>
      </div>

      <section className="options">
        <label className={`opt ${active ? "locked" : ""}`}>
          <span className="opt-icon">🖥️</span>
          <span className="opt-text">
            <span className="opt-title">Block display sleep</span>
            <span className="opt-desc">Keep the screen on</span>
          </span>
          <input
            type="checkbox"
            className="switch"
            checked={status.display}
            disabled={active}
            onChange={(e) => setOption("display", e.currentTarget.checked)}
          />
        </label>
        <label className={`opt ${active ? "locked" : ""}`}>
          <span className="opt-icon">☕</span>
          <span className="opt-text">
            <span className="opt-title">Block system sleep</span>
            <span className="opt-desc">Stay fully awake</span>
          </span>
          <input
            type="checkbox"
            className="switch"
            checked={status.idle}
            disabled={active}
            onChange={(e) => setOption("idle", e.currentTarget.checked)}
          />
        </label>
      </section>

      {error && <div className="error">{error}</div>}

      <footer className="footer">
        {active ? "Assertions held by the OS" : "Choose options, then turn on"}
      </footer>
    </main>
  );
}

export default App;
