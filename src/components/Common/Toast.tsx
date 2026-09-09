import { Component, createSignal, onSettled, For } from 'solid-js';
import { ToastItem, subscribeToasts, removeToast } from '../../lib/toast';

export const ToastContainer: Component = () => {
  const [items, setItems] = createSignal<ToastItem[]>([]);

  onSettled(() => {
    return subscribeToasts((updated) => setItems(updated));
  });

  return (
    <div
      style="position: fixed; bottom: 20px; right: 20px; z-index: 9999; display: flex; flex-direction: column; gap: 8px; pointer-events: none;"
      aria-live="polite"
    >
      <For each={items()}>
        {(item) => {
          const typeColors = {
            info: {
              border: 'var(--accent-glow)',
              bg: 'rgba(10, 16, 26, 0.95)',
              text: 'var(--accent-primary)',
            },
            success: {
              border: 'rgba(16, 185, 129, 0.4)',
              bg: 'rgba(10, 24, 18, 0.95)',
              text: '#10b981',
            },
            warning: {
              border: 'rgba(249, 115, 22, 0.4)',
              bg: 'rgba(26, 16, 10, 0.95)',
              text: '#f97316',
            },
            error: {
              border: 'rgba(239, 68, 68, 0.4)',
              bg: 'rgba(26, 10, 10, 0.95)',
              text: '#ef4444',
            },
          };
          const style = typeColors[item.type] || typeColors.info;

          return (
            <div
              style={`
                pointer-events: auto;
                display: flex;
                align-items: center;
                gap: 10px;
                padding: 10px 16px;
                border-radius: 8px;
                border: 1px solid ${style.border};
                background: ${style.bg};
                backdrop-filter: blur(12px);
                color: #ffffff;
                font-size: 13px;
                box-shadow: 0 8px 32px rgba(0,0,0,0.5);
                animation: toast-in 0.2s ease-out;
              `}
            >
              <div
                style={`width: 8px; height: 8px; border-radius: 50%; background: ${style.text};`}
              />
              <span>{item.message}</span>
              <button
                style="background: transparent; border: none; color: #888; cursor: pointer; padding: 2px 6px; margin-left: 8px;"
                onClick={() => removeToast(item.id)}
              >
                ✕
              </button>
            </div>
          );
        }}
      </For>
    </div>
  );
};
