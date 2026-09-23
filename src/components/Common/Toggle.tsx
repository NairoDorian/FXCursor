import { Component } from 'solid-js';

interface ToggleProps {
  label?: string;
  sub?: string;
  /** Accessible name when the toggle has no visible `label` (e.g. a card-header switch). */
  ariaLabel?: string;
  checked: boolean;
  onChange: (val: boolean) => void;
}

export const Toggle: Component<ToggleProps> = (props) => {
  return (
    <div class="card-header">
      {props.label && (
        <div>
          <div class="card-title">{props.label}</div>
          {props.sub && <div class="card-desc">{props.sub}</div>}
        </div>
      )}
      <label class="switch">
        <input
          type="checkbox"
          aria-label={props.ariaLabel ?? props.label}
          checked={props.checked}
          onChange={(e) => props.onChange(e.currentTarget.checked)}
        />
        <span class="slider-round" />
      </label>
    </div>
  );
};
