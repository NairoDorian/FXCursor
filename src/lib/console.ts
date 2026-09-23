import { commands } from './bindings';
import { listen } from '@tauri-apps/api/event';

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

/**
 * Renders one console argument for the Dev Console. Never throws: a logging call must not break
 * its caller (plain `JSON.stringify` threw on circular objects and printed `Error`s as "{}").
 */
export function formatLogArg(arg: unknown): string {
  if (arg instanceof Error) return arg.stack ?? `${arg.name}: ${arg.message}`;
  if (typeof arg !== 'object' || arg === null) return String(arg);
  try {
    return JSON.stringify(arg) ?? String(arg);
  } catch {
    return Object.prototype.toString.call(arg); // circular or otherwise unserialisable
  }
}

// Intercept browser console methods
if (typeof window !== 'undefined') {
  const wrap = (level: LogLevel, original: (...args: unknown[]) => void) =>
    (...args: unknown[]) => {
      original(...args);
      pushLog(level, args.map(formatLogArg).join(' '));
    };
  console.log = wrap('info', console.log.bind(console));
  console.warn = wrap('warn', console.warn.bind(console));
  console.error = wrap('error', console.error.bind(console));
}

// ---------------------------------------------------------------------------------------------
// Backend (Rust) log stream
// ---------------------------------------------------------------------------------------------

let backendAttached = false;

/**
 * Connects the frontend Dev Console to the Rust logging backend.
 *
 * Pulls the backend's recent log history via `commands.getRecentLogs()` and
 * subscribes to the `rust-log` event so that records emitted via Rust `log::info!`,
 * `warn!`, `debug!`, and `error!` are displayed in the Studio Dev Console alongside
 * browser console output.
 *
 * Idempotent: safe to call multiple times; only the first invocation initializes the listener.
 */
export function attachBackendLogs() {
  if (backendAttached) return;
  backendAttached = true;

  const toLevel = (raw: string): LogLevel =>
    raw === 'error' || raw === 'warn' || raw === 'info' || raw === 'debug' || raw === 'trace'
      ? raw
      : 'info';

  // Every record on this stream is from the backend: tag all of them "rust" (fxcursor_render,
  // wgpu_*, tauri targets included), shortening the app crate's own prefix.
  const shortTarget = (target: string) => `rust::${target.replace(/^fxcursor_lib(::)?/, '')}`;

  void (async () => {
    try {
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

