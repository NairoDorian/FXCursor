import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import { ColorPicker } from '../Common/ColorPicker';

interface SatelliteConfig {
  enabled: boolean;
  count: number;
  orbit_diameter: number;
  size: number;
  speed: number;
  dual_ring: boolean;
  show_orbit_ring: boolean;
  orbit_ring_thickness: number;
  color: [number, number, number, number];
}

interface SatellitesTabProps {
  satellites: SatelliteConfig;
  onChange: (newSatellites: SatelliteConfig) => void;
}

export const SatellitesTab: Component<SatellitesTabProps> = (props) => {
  const update = (patch: Partial<SatelliteConfig>) => {
    props.onChange({ ...props.satellites, ...patch });
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Orbit Satellites (Celestial Bodies)"
        desc="SDF celestial orbs rotating smoothly around the mouse pointer"
        headerRight={
          <Toggle checked={props.satellites.enabled} onChange={(v) => update({ enabled: v })} />
        }
      >
        <ColorPicker
          label="Satellite Color"
          sub="Celestial body and orbit glow tint"
          color={props.satellites.color}
          onChange={(c) => update({ color: c })}
        />
        <Slider
          label="Satellite Count"
          sub="Number of revolving bodies (1 - 16)"
          min={1}
          max={16}
          step={1}
          value={props.satellites.count}
          onChange={(v) => update({ count: v })}
        />
        <Slider
          label="Orbit Diameter"
          sub="Distance from mouse pointer (10 - 250 px)"
          min={10}
          max={250}
          step={1}
          unit="px"
          value={props.satellites.orbit_diameter}
          onChange={(v) => update({ orbit_diameter: v })}
        />
        <Slider
          label="Satellite Size"
          sub="Orb diameter (1 - 30 px)"
          min={1}
          max={30}
          step={0.5}
          unit="px"
          value={props.satellites.size}
          onChange={(v) => update({ size: v })}
        />
        <Slider
          label="Revolution Speed"
          sub="Angular rotation velocity (-20 to +20 rad/s)"
          min={-20.0}
          max={20.0}
          step={0.1}
          unit=" rad/s"
          value={props.satellites.speed}
          onChange={(v) => update({ speed: v })}
        />

        <div class="control-row">
          <div class="control-label">
            <span>Mirrored Dual-Ring Orbit</span>
            <span class="control-sub">Secondary counter-rotating celestial ring</span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              checked={props.satellites.dual_ring}
              onChange={(e) => update({ dual_ring: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>

        <div class="control-row">
          <div class="control-label">
            <span>Show Orbit Track Ring</span>
            <span class="control-sub">Faint luminous circle along the orbital path</span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              checked={props.satellites.show_orbit_ring}
              onChange={(e) => update({ show_orbit_ring: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>

        {props.satellites.show_orbit_ring && (
          <Slider
            label="Orbit Ring Thickness"
            sub="Width of celestial track ring (0.2 - 8 px)"
            min={0.2}
            max={8.0}
            step={0.1}
            unit="px"
            value={props.satellites.orbit_ring_thickness}
            onChange={(v) => update({ orbit_ring_thickness: v })}
          />
        )}
      </SectionCard>
    </div>
  );
};
