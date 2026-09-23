//! Prints a reference trace of [`TrailChain`] + [`build_samples`] as JSON.
//!
//! `bun run fixtures` writes it to `test/fixtures/trail_trace.json`; `test/trail-parity.test.ts`
//! replays the same scripted pointer path through the TypeScript mirror (`src/lib/trail.ts`,
//! which drives the Studio live preview) and requires the same node positions and sample
//! progress. A physics change on one side without the other therefore fails the Bun tests.

use fxcursor_protocol::AppConfig;
use fxcursor_render::{build_samples, TrailChain};
use serde_json::{json, Value};

/// Scripted pointer: a fast sweep with a turn, a hard reversal, then a full stop.
fn pointer(frame: usize) -> (f32, f32) {
    let f = frame as f32;
    match frame {
        0..60 => (100.0 + f * 12.0, 300.0 + (f * 0.15).sin() * 60.0),
        60..90 => {
            let (x0, y0) = pointer(59);
            (x0 - (f - 59.0) * 9.0, y0 + (f - 59.0) * 4.0)
        }
        _ => pointer(89),
    }
}

fn round(v: f32) -> f64 {
    (f64::from(v) * 1000.0).round() / 1000.0
}

fn run(name: &str, config: &AppConfig, fps: f32, frames: usize) -> Value {
    let mut chain = TrailChain::with_capacity(config.trail.length as usize);
    let mut checkpoints = Vec::new();
    for frame in 0..frames {
        let (x, y) = pointer(frame);
        chain.resize(config.trail.length as usize, x, y);
        chain.advance(x, y, 1.0 / fps, config);
        if frame % 30 == 29 || frame + 1 == frames {
            let nodes: Vec<(f32, f32, f32)> = chain.nodes().collect();
            let mut samples = Vec::new();
            build_samples(&nodes, config, &mut samples);
            checkpoints.push(json!({
                "frame": frame,
                "brush": [round(chain.brush().0), round(chain.brush().1)],
                "nodes": nodes.iter().map(|n| [round(n.0), round(n.1), round(n.2)]).collect::<Vec<_>>(),
                "samples": samples.iter().map(|s| [round(s.x), round(s.y), round(s.progress)]).collect::<Vec<_>>(),
                "moving": chain.is_moving(),
            }));
        }
    }
    json!({ "name": name, "fps": fps, "frames": frames, "checkpoints": checkpoints })
}

fn main() {
    let base = AppConfig::default();
    let mut lazy = AppConfig::default();
    lazy.trail.lazy_enabled = true;
    let mut short = AppConfig::default();
    short.trail.length = 24;
    short.trail.adaptive_quality = false;
    short.trail.interpolation_steps = 3;

    let trace = json!({
        "config_overrides": {
            "default": {},
            "lazy": { "lazy_enabled": true },
            "short_fixed_steps": { "length": 24, "adaptive_quality": false, "interpolation_steps": 3 },
        },
        "runs": [
            run("default", &base, 60.0, 150),
            run("default_144hz", &base, 144.0, 360),
            run("lazy", &lazy, 60.0, 150),
            run("short_fixed_steps", &short, 60.0, 150),
        ],
    });
    println!("{}", serde_json::to_string(&trace).expect("serialize trace"));
}
