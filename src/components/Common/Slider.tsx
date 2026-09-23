import { Component } from 'solid-js';

interface SliderProps {
  label: string;
  sub?: string;
  min: number;
  max: number;
  step?: number;
  value: number;
  /**
   * Display unit. `'%'` treats the value as a 0–1 fraction (0.5 → "50%"); for values that are
   * already percentages pass `format` instead.
   */
  unit?: string;
  format?: (v: number) => string;
  /** Greys the control out (e.g. a parameter another setting currently overrides). */
  disabled?: boolean;
  onChange: (val: number) => void;
}

export const Slider: Component<SliderProps> = (props) => {
  const step = () => {
    if (props.step !== undefined) return props.step;
    const diff = props.max - props.min;
    if (diff <= 1) return 0.01;
    if (diff <= 10) return 0.1;
    return 1;
  };

  const safeVal = () => {
    const v = Number(props.value);
    return isNaN(v) ? props.min : v;
  };

  const displayVal = () => {
    const val = safeVal();
    if (props.format) return props.format(val);
    if (props.unit === '%') return `${Math.round(val * 100)}%`;
    if (props.unit === 'px') return `${Math.round(val * 10) / 10}px`;
    if (props.unit === 'x') return `${val.toFixed(2)}x`;
    if (props.unit) return `${val}${props.unit}`;
    return Number.isInteger(val) ? String(val) : val.toFixed(2);
  };

  // `input` fires on every drag step and once more on release, so `change` is not needed
  // (listening to both sent the final value to the backend twice).
  const handleInput = (e: Event & { currentTarget: HTMLInputElement }) => {
    const val = parseFloat(e.currentTarget.value);
    if (!isNaN(val)) {
      props.onChange(val);
    }
  };

  return (
    <div class="control-row" style={props.disabled ? 'opacity: 0.45;' : undefined}>
      <div class="control-label">
        <span>{props.label}</span>
        {props.sub && <span class="control-sub">{props.sub}</span>}
      </div>
      <div class="slider-container">
        <input
          type="range"
          class="custom-slider"
          aria-label={props.label}
          aria-valuetext={displayVal()}
          min={props.min}
          max={props.max}
          step={step()}
          value={safeVal()}
          disabled={props.disabled}
          onInput={handleInput}
        />
        <span class="slider-val">{displayVal()}</span>
      </div>
    </div>
  );
};
