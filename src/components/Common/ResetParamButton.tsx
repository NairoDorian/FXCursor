import { Component, Show } from 'solid-js';

interface ResetParamButtonProps {
  /** The value this parameter ships with. */
  defaultValue: number;
  /** The parameter's current value; the button is inert while they match. */
  value: number;
  label: string;
  disabled?: boolean;
  onReset: (value: number) => void;
}

/**
 * The circular-arrow "revert this parameter" control.
 *
 * It stays dimmed and disabled while the value already equals the default, so it never implies
 * there is a change to undo — only a modified parameter offers one. The comparison is tolerant
 * because a slider drag can land on 0.04999999 rather than 0.05 and an exact compare would
 * leave the arrow permanently lit.
 */
export const ResetParamButton: Component<ResetParamButtonProps> = (props) => {
  const isModified = () => Math.abs(props.value - props.defaultValue) > 1e-6;

  return (
    <Show when={!Number.isNaN(props.defaultValue)}>
      <button
        type="button"
        class={`reset-param ${isModified() ? 'active' : ''}`}
        disabled={props.disabled || !isModified()}
        title={`Reset "${props.label}" to its default (${props.defaultValue})`}
        aria-label={`Reset ${props.label} to default`}
        onClick={() => props.onReset(props.defaultValue)}
      >
        <svg viewBox="0 0 24 24" width="13" height="13" aria-hidden="true">
          {/* Counter-clockwise arrow in a circle: the universal "revert" glyph. */}
          <path
            d="M12 5V2L7.5 6.5 12 11V8a5.5 5.5 0 1 1-5.5 5.5"
            fill="none"
            stroke="currentColor"
            stroke-width="2.1"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </Show>
  );
};
