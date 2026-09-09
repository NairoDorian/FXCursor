export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  id: string;
  level: LogLevel;
  message: string;
  timestamp: string;
  target?: string;
}

type LogListener = (logs: LogEntry[]) => void;

let logs: LogEntry[] = [];
const listeners = new Set<LogListener>();

function notify() {
  const snapshot = [...logs];
  listeners.forEach((listener) => listener(snapshot));
}

export function pushLog(level: LogLevel, message: string, target?: string) {
  const id = Math.random().toString(36).substring(2, 9);
  const timestamp = new Date().toLocaleTimeString();
  const entry: LogEntry = { id, level, message, timestamp, target };
  logs.push(entry);
  if (logs.length > 500) logs.shift();
  notify();
}

export function clearLogs() {
  logs = [];
  notify();
}

export function subscribeLogs(listener: LogListener): () => void {
  listeners.add(listener);
  listener([...logs]);
  return () => {
    listeners.delete(listener);
  };
}

// Intercept browser console methods
if (typeof window !== 'undefined') {
  const origLog = console.log;
  const origWarn = console.warn;
  const origError = console.error;

  console.log = (...args: any[]) => {
    origLog(...args);
    pushLog(
      'info',
      args.map((a) => (typeof a === 'object' ? JSON.stringify(a) : String(a))).join(' ')
    );
  };
  console.warn = (...args: any[]) => {
    origWarn(...args);
    pushLog(
      'warn',
      args.map((a) => (typeof a === 'object' ? JSON.stringify(a) : String(a))).join(' ')
    );
  };
  console.error = (...args: any[]) => {
    origError(...args);
    pushLog(
      'error',
      args.map((a) => (typeof a === 'object' ? JSON.stringify(a) : String(a))).join(' ')
    );
  };
}

// ---------------------------------------------------------------------------------------------
// Backend (Rust) log stream
// ---------------------------------------------------------------------------------------------

let backendAttached = false;

/**
 * Pulls the backend's recent log history and subscribes to the `rust-log` event so Rust
 * `log::info!`/`warn!`/`error!` records show up in the Dev Console next to browser output.
 * Safe to call more than once; only the first call attaches.
 */
export function attachBackendLogs() {
  if (backendAttached) return;
  backendAttached = true;

  const toLevel = (raw: string): LogLevel =>
    raw === 'error' || raw === 'warn' || raw === 'info' || raw === 'debug' || raw === 'trace'
      ? raw
      : 'info';

  const shortTarget = (target: string) => target.replace(/^fxcursor_lib::?/, 'rust::');

  void (async () => {
    try {
      const [{ commands }, { listen }] = await Promise.all([
        import('./bindings'),
        import('@tauri-apps/api/event'),
      ]);
      const history = await commands.getRecentLogs();
      for (const rec of history) {
        pushLog(toLevel(rec.level), rec.message, shortTarget(rec.target));
      }
      await listen<{ level: string; target: string; message: string }>('rust-log', (event) => {
        const rec = event.payload;
        pushLog(toLevel(rec.level), rec.message, shortTarget(rec.target));
      });
    } catch (e) {
      pushLog('warn', `Backend log stream unavailable: ${e}`, 'console');
    }
  })();
}
