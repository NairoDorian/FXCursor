//! Geometry dump for the open "crescent at the head" bug (docs/CRESCENT_AT_TRAIL_HEAD.md).
//!
//! Prints the numbers behind the diagnosis: how far the merged centreline folds in front of
//! the head, how much `progress` jumps between the first two drawn samples, and the resulting
//! per-capsule radii.
//!
//! ## This is NOT a renderer — never use it to judge a visual fix
//!
//! It draws nothing. It reads `build_samples` / `build_layer_capsules`, which are the same code
//! that fills the GPU instance buffer, so the *numbers* are trustworthy. But it cannot tell you
//! what the artifact looks like, because the real pipeline does things a geometry dump does not
//! model: `blur` feathers each capsule's edge across 39-50% of its radius, the depth pre-pass
//! resolves the union by that feathered alpha, and compositing is pre-multiplied.
//!
//! A previous fix was judged on a CPU rasterisation of exactly these numbers, looked clean, and
//! was still broken — and worse — in the real app. **Judge visual changes on a capture from
//! `bun run tauri dev`**; see docs/RUNNING_AND_DEBUGGING.md.
//!
//! Usage: `cargo run -p fxcursor-render --example dump_crescent`

use fxcursor_protocol::AppConfig;
use fxcursor_render::{build_layer_capsules, build_samples, Sample, TrailChain, UNBOUNDED_VIEWPORT};

const FRAME: f32 = 1.0 / 60.0;
const N: usize = 80;

fn main() {
    let config = AppConfig::default();
    let mut chain = TrailChain::with_capacity(N);
    let mut samples: Vec<Sample> = Vec::new();

    // Fast constant run to the right, then a dead stop — the reported situation.
    let stop_at = 40u32;
    let speed = 30.0f32;
    let pointer = |f: u32| (f.min(stop_at) as f32 * speed, 0.0f32);

    println!("defaults: length {}  spring {}  damping {}  cursor_size {}  min_width {}",
        config.trail.length, config.trail.spring, config.trail.damping,
        config.trail.cursor_size, config.trail.min_width);
    println!("head stroke half-width: {:.1} px\n",
        config.trail.cursor_size * config.trail.layers[0].width_factor * 0.5);

    println!("frame  headGap  samples  progress jump   layer0 radii (a->b)   nodes ahead of head");
    for frame in 0..=200u32 {
        let (x, y) = pointer(frame);
        chain.resize(N, x, y);
        chain.advance(x, y, FRAME, &config);
        if frame < stop_at || (frame - stop_at) % 30 != 0 {
            continue;
        }
        let nodes: Vec<(f32, f32, f32)> = chain.nodes().collect();
        build_samples(&nodes, &config, &mut samples);
        let head = nodes[0];

        // How far does progress move between the first two drawn samples?
        let jump = samples.get(1).map(|s| s.progress - samples[0].progress).unwrap_or(0.0);

        // Widest layer, first two capsules: the shoulder that reads as the crescent.
        let mut caps = Vec::new();
        build_layer_capsules(&samples, 0, &config.trail.layers[0], &config, 0.0,
                             UNBOUNDED_VIEWPORT, &mut caps);
        let radii = if caps.len() >= 2 {
            format!("{:.1}->{:.1}  {:.1}->{:.1}", caps[0].radii[0], caps[0].radii[1],
                    caps[1].radii[0], caps[1].radii[1])
        } else {
            "-".to_string()
        };

        // Nodes the merge keeps that sit in FRONT of the head (a fold in the centreline).
        let mut ahead = 0.0f32;
        let mut kept: Vec<(f32, f32)> = vec![(head.0, head.1)];
        for &(nx, ny, _) in &nodes {
            if let Some(l) = kept.last() {
                if ((nx - l.0).powi(2) + (ny - l.1).powi(2)).sqrt() < 0.25 { continue; }
            }
            kept.push((nx, ny));
            ahead = ahead.max(nx - head.0);
            if kept.len() >= 8 { break; }
        }

        println!("{frame:>5}  {:>7.2}  {:>7}  {:>14.3}  {:>21}  {:>8.2} px",
            (head.0 - x).abs(), samples.len(), jump, radii, ahead);
    }
}
