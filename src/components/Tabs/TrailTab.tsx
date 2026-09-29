import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import type { TrailConfig } from '../../lib/presets';
import { getDefaultConfig } from '../../lib/presets';
import { isModified, resetFields } from '../../lib/reset';

interface TrailTabProps {
  trail: TrailConfig;
  onChange: (newTrail: TrailConfig) => void;
}

const DEFAULTS = getDefaultConfig().trail;

/** Which fields each card owns, so one card's reset never touches its neighbour's sliders. */
const HEAD_KEYS = ['head_spring', 'head_damping'] as const;
const LAZY_KEYS = ['lazy_enabled', 'lazy_radius', 'lazy_friction'] as const;
const BODY_KEYS = [
  'spring',
  'damping',
  'length',
  'cursor_size',
  'min_width',
  'interpolation_steps',
] as const;
const RESPONSE_KEYS = [
  'fade_mode',
  'velocity_width_mult',
  'velocity_alpha_mult',
  'velocity_reference_speed',
  'enable_gradient',
  'adaptive_quality',
] as const;

export const TrailTab: Component<TrailTabProps> = (props) => {
  const update = (patch: Partial<TrailConfig>) => {
    props.onChange({ ...props.trail, ...patch });
  };

  // Index = `fade_mode` (`apply_fade_curve` in renderer.rs / `fadeCurve` in lib/trail.ts).
  const fadeModes = ['Linear', 'Ease Out', 'Exponential', 'Sigmoid', 'Smoothstep'];

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      {/* Head Kinematics (the header switch is the master toggle for the whole ribbon) */}
      <SectionCard
        title="Ribbon Trail · Head Kinematics"
        desc="Spring-damper pulling the leading node toward the pointer (or the Lazy Brush); the switch turns the whole trail on or off"
        modified={isModified(props.trail, DEFAULTS, HEAD_KEYS)}
        onReset={() => props.onChange(resetFields(props.trail, DEFAULTS, HEAD_KEYS))}
        headerRight={
          <Toggle
            ariaLabel="Enable ribbon trail"
            checked={props.trail.enabled}
            onChange={(v) => update({ enabled: v })}
          />
        }
      >
        <Slider
          label="Head Spring"
          sub="Pull of the ribbon head toward the pointer, as a fraction of the gap (0 = free drift, 1 = snaps shut)"
          min={0}
          max={1}
          step={0.005}
          value={props.trail.head_spring}
          defaultValue={DEFAULTS.head_spring}
          onChange={(v) => update({ head_spring: v })}
        />
        <Slider
          label="Head Friction"
          sub="Fraction of velocity the head keeps per frame (0 = stops dead, 1 = frictionless)"
          min={0}
          max={1}
          step={0.005}
          value={props.trail.head_damping}
          defaultValue={DEFAULTS.head_damping}
          onChange={(v) => update({ head_damping: v })}
        />
      </SectionCard>

      {/* LazyBrush dead-zone pointer smoother */}
      <SectionCard
        title="Lazy Brush"
        desc="Dead-zone smoother between the raw pointer and the ribbon: micro-jitter never reaches the trail"
        modified={isModified(props.trail, DEFAULTS, LAZY_KEYS)}
        onReset={() => props.onChange(resetFields(props.trail, DEFAULTS, LAZY_KEYS))}
      >
        <div class="control-row">
          <div class="control-label">
            <span>Enable Lazy Brush</span>
            <span class="control-sub">Hold the brush still until the pointer leaves the dead zone</span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              aria-label="Enable Lazy Brush"
              checked={props.trail.lazy_enabled ?? false}
              onChange={(e) => update({ lazy_enabled: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>
        <Slider
          label="Dead-Zone Radius"
          sub="Distance the pointer must travel before the brush starts moving (5 - 150 px)"
          min={5}
          max={150}
          step={1}
          unit="px"
          value={props.trail.lazy_radius ?? 30}
          defaultValue={DEFAULTS.lazy_radius}
          onChange={(v) => update({ lazy_radius: v })}
        />
        <Slider
          label="Brush Friction"
          sub="How slowly the brush catches up once outside the dead zone (0 = instant, 0.99 = frozen)"
          min={0}
          max={0.99}
          step={0.01}
          value={props.trail.lazy_friction ?? 0.4}
          defaultValue={DEFAULTS.lazy_friction}
          onChange={(v) => update({ lazy_friction: v })}
        />
      </SectionCard>

      {/* Trail Body Chain Physics */}
      <SectionCard
        title="Trail Body Spring Chain Physics"
        desc="Spring-damper chain with 0.3× second-neighbour coupling (a 512 px gap guard only catches teleports)"
        modified={isModified(props.trail, DEFAULTS, BODY_KEYS)}
        onReset={() => props.onChange(resetFields(props.trail, DEFAULTS, BODY_KEYS))}
      >
        <Slider
          label="Body Spring"
          sub="Pull of each node toward the one ahead, as a fraction of the gap (0 = no pull, 1 = stiff)"
          min={0}
          max={1}
          step={0.005}
          value={props.trail.spring}
          defaultValue={DEFAULTS.spring}
          onChange={(v) => update({ spring: v })}
        />
        <Slider
          label="Body Friction"
          sub="Fraction of velocity each body node keeps per frame (0 = stops dead, 1 = frictionless)"
          min={0}
          max={1}
          step={0.005}
          value={props.trail.damping}
          defaultValue={DEFAULTS.damping}
          onChange={(v) => update({ damping: v })}
        />
        <Slider
          label="Trail Node Count (Length)"
          sub="Nodes in the chain (4 - 500); the fade and width taper run over the node index"
          min={4}
          max={500}
          step={1}
          value={props.trail.length}
          defaultValue={DEFAULTS.length}
          onChange={(v) => update({ length: v })}
        />
        <Slider
          label="Base Cursor Reference Size"
          sub="Master pixel width scaled by all 4 layers (5 - 200 px)"
          min={5}
          max={200}
          step={1}
          unit="px"
          value={props.trail.cursor_size}
          defaultValue={DEFAULTS.cursor_size}
          onChange={(v) => update({ cursor_size: v })}
        />
        <Slider
          label="Minimum Tail Width"
          sub="Minimum pixel thickness floor at ribbon tail (0 - 30 px)"
          min={0}
          max={30}
          step={0.5}
          unit="px"
          value={props.trail.min_width ?? 2.0}
          defaultValue={DEFAULTS.min_width}
          onChange={(v) => update({ min_width: v })}
        />
        <Slider
          label="Curve Smoothness (Spline Steps)"
          sub={
            props.trail.adaptive_quality
              ? 'Ignored while Adaptive Quality picks the sample density from the curvature'
              : 'Fixed Catmull-Rom sub-samples per node segment (1 - 32)'
          }
          min={1}
          max={32}
          step={1}
          disabled={props.trail.adaptive_quality}
          value={props.trail.interpolation_steps}
          defaultValue={DEFAULTS.interpolation_steps}
          onChange={(v) => update({ interpolation_steps: v })}
        />
      </SectionCard>

      {/* Dynamic Velocity & Fade Response */}
      <SectionCard
        title="Dynamic Velocity & Fade Profiles"
        desc="Speed-based ribbon expansion, brightness amplification, and mathematical decay curves"
        modified={isModified(props.trail, DEFAULTS, RESPONSE_KEYS)}
        onReset={() => props.onChange(resetFields(props.trail, DEFAULTS, RESPONSE_KEYS))}
      >
        <div class="control-row">
          <div class="control-label">
            <span>Fade Profile Curve</span>
            <span class="control-sub">Mathematical decay function from head to tail</span>
          </div>
          <div style="display: flex; gap: 6px; flex-wrap: wrap;">
            {fadeModes.map((modeName, idx) => (
              <button
                class={`tab-btn ${props.trail.fade_mode === idx ? 'active' : ''}`}
                style="padding: 4px 10px; font-size: 11px;"
                onClick={() => update({ fade_mode: idx })}
              >
                {modeName}
              </button>
            ))}
          </div>
        </div>

        <Slider
          label="Velocity Width Multiplier"
          sub="Extra width at the reference speed below"
          min={0}
          max={5.0}
          step={0.05}
          unit="x"
          value={props.trail.velocity_width_mult}
          defaultValue={DEFAULTS.velocity_width_mult}
          onChange={(v) => update({ velocity_width_mult: v })}
        />
        <Slider
          label="Velocity Alpha Multiplier"
          sub="Extra opacity at the reference speed below"
          min={0}
          max={5.0}
          step={0.05}
          unit="x"
          value={props.trail.velocity_alpha_mult}
          defaultValue={DEFAULTS.velocity_alpha_mult}
          onChange={(v) => update({ velocity_alpha_mult: v })}
        />
        <Slider
          label="Velocity Reference Speed"
          sub="Speed (px per 1/120 s) at which the two boosts above reach 100% — 20 = 2400 px/s"
          min={1}
          max={200}
          step={1}
          unit="px"
          value={props.trail.velocity_reference_speed ?? 20}
          defaultValue={DEFAULTS.velocity_reference_speed}
          onChange={(v) => update({ velocity_reference_speed: v })}
        />

        <div class="control-row">
          <div class="control-label">
            <span>Enable Color Gradient Blending</span>
            <span class="control-sub">Smooth transition from layer Start Color to End Color</span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              aria-label="Enable color gradient blending"
              checked={props.trail.enable_gradient}
              onChange={(e) => update({ enable_gradient: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>

        <div class="control-row">
          <div class="control-label">
            <span>Adaptive Quality Scaling</span>
            <span class="control-sub">
              Dense samples in bends, sparse on straight runs (3 - 24 px spacing); fewer capsules
              for the same smoothness
            </span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              aria-label="Adaptive quality scaling"
              checked={props.trail.adaptive_quality}
              onChange={(e) => update({ adaptive_quality: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>
      </SectionCard>
    </div>
  );
};
