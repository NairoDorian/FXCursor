import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import { ColorPicker } from '../Common/ColorPicker';

interface ParticleConfig {
  enabled: boolean;
  count_per_click: number;
  duration_ms: number;
  base_speed: number;
  gravity: number;
  friction: number;
  size: number;
  color: [number, number, number, number];
}

interface ParticlesTabProps {
  particles: ParticleConfig;
  onChange: (newParticles: ParticleConfig) => void;
}

export const ParticlesTab: Component<ParticlesTabProps> = (props) => {
  const update = (patch: Partial<ParticleConfig>) => {
    props.onChange({ ...props.particles, ...patch });
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Kinematic Particle Bursts"
        desc="Physics-driven particles with gravity vector, air resistance, and alpha decay"
        headerRight={
          <Toggle checked={props.particles.enabled} onChange={(v) => update({ enabled: v })} />
        }
      >
        <ColorPicker
          label="Particle Spark Color"
          sub="Primary luminous tint of sparks"
          color={props.particles.color}
          onChange={(c) => update({ color: c })}
        />
        <Slider
          label="Particles per Click"
          sub="Burst quantity emitted on mouse press (2 - 100)"
          min={2}
          max={100}
          step={1}
          value={props.particles.count_per_click}
          onChange={(v) => update({ count_per_click: v })}
        />
        <Slider
          label="Particle Size"
          sub="Radius of particle billboards (0.5 - 20 px)"
          min={0.5}
          max={20.0}
          step={0.5}
          unit="px"
          value={props.particles.size}
          onChange={(v) => update({ size: v })}
        />
        <Slider
          label="Ejection Velocity"
          sub="Initial burst launch speed (25 - 1500 px/s)"
          min={25}
          max={1500}
          step={25}
          unit="px/s"
          value={props.particles.base_speed}
          onChange={(v) => update({ base_speed: v })}
        />
        <Slider
          label="Downward Gravity"
          sub="Vertical acceleration pulling particles down (-500 to 1000 px/s²)"
          min={-500}
          max={1000}
          step={20}
          unit="px/s²"
          value={props.particles.gravity}
          onChange={(v) => update({ gravity: v })}
        />
        <Slider
          label="Air Drag Friction"
          sub="Velocity decay factor per second (0.50 - 0.99)"
          min={0.5}
          max={0.99}
          step={0.01}
          value={props.particles.friction}
          onChange={(v) => update({ friction: v })}
        />
        <Slider
          label="Lifespan Duration"
          sub="Particle time to live before fade (100 - 2500 ms)"
          min={100}
          max={2500}
          step={25}
          unit="ms"
          value={props.particles.duration_ms}
          onChange={(v) => update({ duration_ms: v })}
        />
      </SectionCard>
    </div>
  );
};
