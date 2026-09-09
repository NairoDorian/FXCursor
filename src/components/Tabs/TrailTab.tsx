import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import { LayerConfig } from '../../lib/presets';

interface TrailConfig {
  enabled: boolean;
  length: number;
  spring: number;
  damping: number;
  head_spring: number;
  head_damping: number;
  lead_nodes?: number;
  cursor_size: number;
  min_width: number;
  velocity_width_mult: number;
  velocity_alpha_mult: number;
  interpolation_steps: number;
  fade_mode: number;
  enable_gradient: boolean;
  adaptive_quality: boolean;
  layers: [LayerConfig, LayerConfig, LayerConfig, LayerConfig];
}

interface TrailTabProps {
  trail: TrailConfig;
  onChange: (newTrail: TrailConfig) => void;
}

export const TrailTab: Component<TrailTabProps> = (props) => {
  const update = (patch: Partial<TrailConfig>) => {
    props.onChange({ ...props.trail, ...patch });
  };

  const fadeModes = ['Linear', 'Ease Out', 'Exponential', 'Sigmoid', 'Smoothstep'];

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      {/* Head Kinematics */}
      <SectionCard
        title="Head Point Kinematics"
        desc="Spring force and velocity friction pulling the leading ribbon head toward the hardware cursor"
        headerRight={
          <Toggle checked={props.trail.enabled} onChange={(v) => update({ enabled: v })} />
        }
      >
        <Slider
          label="Head Follow Strength"
          sub="How tightly the ribbon head tracks the pointer; it follows without overshooting (5 = loose, 98 = glued)"
          min={5}
          max={98}
          step={1}
          value={props.trail.head_spring ?? props.trail.spring}
          onChange={(v) => update({ head_spring: v })}
        />
        <Slider
          label="Head Smoothing"
          sub="Jitter filter applied to the raw cursor before the head follows it (0 = raw, 95 = very smooth)"
          min={0}
          max={95}
          step={1}
          value={props.trail.head_damping ?? props.trail.damping}
          onChange={(v) => update({ head_damping: v })}
        />
        <Slider
          label="Lead Nodes"
          sub="Leading nodes that follow the pointer without whipping before the spring chain takes over (1 - 12)"
          min={1}
          max={12}
          step={1}
          value={props.trail.lead_nodes ?? 4}
          onChange={(v) => update({ lead_nodes: v })}
        />
      </SectionCard>

      {/* Trail Body Chain Physics */}
      <SectionCard
        title="Trail Body Spring Chain Physics"
        desc="D3D11 multi-segment spring-damper chain with second-order neighbor curvature stabilization"
      >
        <Slider
          label="Trail Segment Spring Strength"
          sub="Elastic tension between consecutive trail nodes (1 - 500)"
          min={1}
          max={500}
          step={1}
          value={props.trail.spring}
          onChange={(v) => update({ spring: v })}
        />
        <Slider
          label="Trail Segment Friction / Damping"
          sub="Fluid viscosity and drag along the ribbon body (0 - 99)"
          min={0}
          max={99}
          step={1}
          value={props.trail.damping}
          onChange={(v) => update({ damping: v })}
        />
        <Slider
          label="Trail Node Count (Length)"
          sub="Total discrete physical links simulated in the chain (4 - 150)"
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
          sub="Catmull-Rom sub-samples per segment for silky curves (1 - 10)"
          min={1}
          max={10}
          step={1}
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
          sub="Dynamic ribbon widening during rapid flicks (0.0x - 5.0x)"
          min={0}
          max={5.0}
          step={0.05}
          unit="x"
          value={props.trail.velocity_width_mult}
          onChange={(v) => update({ velocity_width_mult: v })}
        />
        <Slider
          label="Velocity Alpha Multiplier"
          sub="Brightness amplification during rapid flicks (0.0x - 5.0x)"
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
              Optimizes spline sub-steps during ultra-fast flicks for 0ms frame lag
            </span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
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
