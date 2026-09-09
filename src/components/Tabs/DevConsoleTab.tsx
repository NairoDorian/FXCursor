import { Component, createSignal, onSettled, For, Show } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { LogEntry, subscribeLogs, clearLogs, LogLevel } from '../../lib/console';

export const DevConsoleTab: Component = () => {
  const [logs, setLogs] = createSignal<LogEntry[]>([]);
  const [filterLevel, setFilterLevel] = createSignal<LogLevel | 'all'>('all');
  const [searchQuery, setSearchQuery] = createSignal('');
  const [isPaused, setIsPaused] = createSignal(false);

  onSettled(() => {
    return subscribeLogs((updated) => {
      if (!isPaused()) {
        setLogs(updated);
      }
    });
  });

  const filteredLogs = () => {
    return logs().filter((log) => {
      const matchLevel = filterLevel() === 'all' || log.level === filterLevel();
      const matchSearch =
        !searchQuery() || log.message.toLowerCase().includes(searchQuery().toLowerCase());
      return matchLevel && matchSearch;
    });
  };

  const levelColor = (lvl: LogLevel) => {
    switch (lvl) {
      case 'error':
        return '#ef4444';
      case 'warn':
        return '#f97316';
      case 'info':
        return 'var(--accent-primary)';
      case 'debug':
        return '#a855f7';
      default:
        return '#888888';
    }
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Live Diagnostic Dev Console"
        desc="Real-time log streamer capturing IPC messages, shader compilation events, and render telemetry"
        headerRight={
          <div style="display: flex; gap: 8px;">
            <button
              class={`tab-btn ${isPaused() ? 'active' : ''}`}
              style="padding: 4px 10px; font-size: 11px;"
              onClick={() => setIsPaused(!isPaused())}
            >
              {isPaused() ? 'Resume Stream' : 'Pause'}
            </button>
            <button
              class="tab-btn"
              style="padding: 4px 10px; font-size: 11px;"
              onClick={() => clearLogs()}
            >
              Clear Logs
            </button>
          </div>
        }
      >
        {/* Controls bar */}
        <div style="display: flex; gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap;">
          <input
            type="text"
            placeholder="Filter logs by keyword..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            style="
              flex: 1;
              min-width: 200px;
              background: #121318;
              border: 1px solid rgba(255,255,255,0.1);
              color: #ffffff;
              padding: 6px 12px;
              border-radius: 6px;
              font-size: 12px;
              outline: none;
            "
          />

          <div style="display: flex; gap: 4px;">
            {(['all', 'info', 'warn', 'error'] as const).map((lvl) => (
              <button
                class={`tab-btn ${filterLevel() === lvl ? 'active' : ''}`}
                style="padding: 4px 8px; font-size: 11px; text-transform: uppercase;"
                onClick={() => setFilterLevel(lvl)}
              >
                {lvl}
              </button>
            ))}
          </div>
        </div>

        {/* Log Viewer Container */}
        <div
          style="
            height: 360px;
            overflow-y: auto;
            background: #08080a;
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 8px;
            padding: 10px;
            font-family: monospace;
            font-size: 11px;
            display: flex;
            flex-direction: column;
            gap: 4px;
          "
        >
          <Show
            when={filteredLogs().length > 0}
            fallback={
              <div style="color: #666; text-align: center; padding: 40px;">
                No log entries matching active filter criteria
              </div>
            }
          >
            <For each={filteredLogs()}>
              {(log) => (
                <div
                  style="
                    display: flex;
                    gap: 8px;
                    padding: 2px 4px;
                    border-radius: 4px;
                    background: rgba(255,255,255,0.02);
                  "
                >
                  <span style="color: #666;">[{log.timestamp}]</span>
                  <span
                    style={`
                      color: ${levelColor(log.level)};
                      font-weight: 600;
                      text-transform: uppercase;
                      width: 44px;
                      flex-shrink: 0;
                    `}
                  >
                    {log.level}
                  </span>
                  <Show when={log.target}>
                    <span
                      style={`color: ${log.target?.startsWith('rust') ? '#f97316' : '#4facfe'}; flex-shrink: 0; opacity: 0.85;`}
                      title={log.target}
                    >
                      {log.target?.startsWith('rust') ? 'rust' : 'web'}
                    </span>
                  </Show>
                  <span style="color: #e0e0e0; word-break: break-all; flex: 1;">{log.message}</span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </SectionCard>
    </div>
  );
};
