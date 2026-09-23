//! Every built-in preset must be able to show what it configures: an effect (or ribbon layer)
//! that is enabled in the preset but masked off by its `effect_mode` is dead configuration —
//! `particle_firestorm` once shipped with `ParticlesOnly`, which hid its own ember trail.

use fxcursor_protocol::get_builtin_presets;
use fxcursor_render::ModeMask;

#[test]
fn no_builtin_preset_masks_its_own_effects() {
    for preset in get_builtin_presets() {
        let c = &preset.config;
        let mask = ModeMask::from_mode(c.effect_mode);
        let id = &preset.id;
        if c.trail.enabled {
            assert!(mask.trail, "{id}: trail enabled but masked by {:?}", c.effect_mode);
            for (i, layer) in c.trail.layers.iter().enumerate() {
                assert!(!layer.enabled || mask.layers[i], "{id}: layer {i} enabled but masked");
            }
        }
        assert!(!c.head.enabled || mask.head, "{id}: head enabled but masked");
        assert!(!c.ripple.enabled || mask.ripples, "{id}: ripples enabled but masked");
        assert!(!c.particles.enabled || mask.particles, "{id}: particles enabled but masked");
        assert!(!c.satellites.enabled || mask.satellites, "{id}: satellites enabled but masked");
    }
}
