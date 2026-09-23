import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import type { TrailConfig } from '../../lib/presets';

interface TrailTabProps {
  trail: TrailConfig;
  onChange: (newTrail: TrailConfig) => void;
}

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
        desc="Windhawk spring-damper pulling the leading node toward the pointer (or the Lazy Brush); the switch turns the whole trail on or off"
        headerRight={
          <Toggle
            ariaLabel="Enable ribbon trail"
            checked={props.trail.enabled}
            onChange={(v) => update({ enabled: v })}
          />
        }
      >
        <Slider
          label="Head Spring Strength"
          sub="Spring constant ÷1000 per 1/120 s (1 = loose and lagging, 500 = glued to the pointer)"
          min={1}
          max={500}
          step={1}
          value={props.trail.head_spring}
          onChange={(v) => update({ head_spring: v })}
        />
        <Slider
          label="Head Friction"
          sub="Velocity lost per 1/120 s in percent (0 = springy overshoot, 99 = heavy damping)"
          min={0}
          max={99}
          step={1}
          value={props.trail.head_damping}
          onChange={(v) => update({ head_damping: v })}
        />
      </SectionCard>

      {/* LazyBrush dead-zone pointer smoother */}
      <SectionCard
        title="Lazy Brush"
        desc="Dead-zone smoother between the raw pointer and the ribbon: micro-jitter never reaches the trail"
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
          onChange={(v) => update({ lazy_radius: v })}
        />
        <Slider
          label="Brush Friction"
          sub="How slowly the brush catches up once outside the dead zone (0 = instant, 0.99 = frozen)"
          min={0}
          max={0.99}
          step={0.01}
          value={props.trail.lazy_friction ?? 0.4}
          onChange={(v) => update({ lazy_friction: v })}
        />
      </SectionCard>

      {/* Trail Body Chain Physics */}
      <SectionCard
        title="Trail Body Spring Chain Physics"
        desc="Windhawk spring-damper chain with 0.3× second-neighbour coupling (a 512 px gap guard only catches teleports)"
      >
        <Slider
          label="Trail Segment Spring Strength"
          sub="Spring constant ÷1000 between consecutive nodes (1 = long lazy tail, 500 = short stiff tail)"
          min={1}
          max={500}
          step={1}
          value={props.trail.spring}
          onChange={(v) => update({ spring: v })}
        />
        <Slider
          label="Trail Segment Friction / Damping"
          sub="Velocity lost per 1/120 s in percent along the body (0 = wobbly, 99 = sluggish)"
          min={0}
          max={99}
          step={1}
          value={props.trail.damping}
          onChange={(v) => update({ damping: v })}
        />
        <Slider
          label="Trail Node Count (Length)"
          sub="Nodes in the chain (4 - 150); the fade and width taper run over the node index"
          min={4}
          max={150}
          step={1}
          value={props.trail.length}
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
          onChange={(v) => update({ min_width: v })}
        />
        <Slider
          label="Curve Smoothness (Spline Steps)"
          sub={
            props.trail.adaptive_quality
              ? 'Ignored while Adaptive Quality picks the sample density from the curvature'
              : 'Fixed Catmull-Rom sub-samples per node segment (1 - 10)'
          }
          min={1}
          max={10}
          step={1}
          disabled={props.trail.adaptive_quality}
          value={props.trail.interpolation_steps}
          onChange={(v) => update({ interpolation_steps: v })}
        />
      </SectionCard>

      {/* Dynamic Velocity & Fade Response */}
      <SectionCard
        title="Dynamic Velocity & Fade Profiles"
        desc="Speed-based ribbon expansion, brightness amplification, and mathematical decay curves"
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
          sub="Extra width at full speed (reached at 2400 px/s, as in Windhawk)"
          min={0}
          max={5.0}
          step={0.05}
          unit="x"
          value={props.trail.velocity_width_mult}
          onChange={(v) => update({ velocity_width_mult: v })}
        />
        <Slider
          label="Velocity Alpha Multiplier"
          sub="Extra opacity at full speed (same speed scale as the width boost)"
          min={0}
          max={5.0}
          step={0.05}
          unit="x"
          value={props.trail.velocity_alpha_mult}
          onChange={(v) => update({ velocity_alpha_mult: v })}
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
