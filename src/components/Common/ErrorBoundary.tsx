import { Component, ParentProps, Errored } from 'solid-js';
import { toast } from '../../lib/toast';

export const ErrorBoundary: Component<ParentProps> = (props) => {
  return (
    <Errored
      fallback={(err) => {
        const errorObj = typeof err === 'function' ? err() : err;
        const errorMsg =
          errorObj instanceof Error
            ? `${errorObj.name}: ${errorObj.message}\n${errorObj.stack || ''}`
            : String(errorObj);

        const copyError = () => {
          navigator.clipboard.writeText(errorMsg).then(() => {
            toast.info('Error stack trace copied to clipboard');
          });
        };

        return (
          <div
            style="
              padding: 32px;
              color: #ef4444;
              background: #08080a;
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
              height: 100vh;
              display: flex;
              flex-direction: column;
              gap: 16px;
              box-sizing: border-box;
            "
          >
            <div style="display: flex; align-items: center; justify-content: space-between;">
              <h2 style="margin: 0; font-size: 18px; font-weight: 700; color: #ff5555;">
                ⚠️ Application Component Error
              </h2>
              <button class="tab-btn active" style="padding: 6px 14px;" onClick={copyError}>
                Copy Crash Log
              </button>
            </div>
            <p style="color: #888; font-size: 13px; margin: 0;">
              An unexpected error occurred inside a SolidJS component. You can copy the stack trace
              below for diagnostics.
            </p>
            <pre
              style="
                flex: 1;
                white-space: pre-wrap;
                background: #121318;
                padding: 16px;
                border-radius: 8px;
                border: 1px solid rgba(239, 68, 68, 0.2);
                overflow-y: auto;
                font-size: 12px;
                color: #ff8888;
                font-family: monospace;
              "
            >
              {errorMsg}
            </pre>
          </div>
        );
      }}
    >
      {props.children}
    </Errored>
  );
};
