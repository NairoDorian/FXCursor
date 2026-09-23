//! `fxcursor-render` — the single renderer shared by the Tauri Studio (`src-tauri`) and the
//! experimental headless daemon (`crates/fxcursor-daemon`).
//!
//! Responsibilities:
//! * CPU physics: spring-damper trail chain ([`TrailChain`], mirrored in TypeScript by
//!   `src/lib/trail.ts` and checked against `test/fixtures/trail_trace.json`), squishy head,
//!   ripples, particles, satellites.
//! * Geometry: Catmull-Rom resampling of the chain, then one round capsule per sample pair and
//!   layer; the union of capsules is resolved on the GPU with a depth-based max-coverage test
//!   (`shaders/render.wgsl`), so joins and caps are round and overlaps never double-blend.
//! * One wgpu render pass (depth pre-pass + colour pass per layer, then SDF billboards) with
//!   pre-multiplied alpha.
//!
//! The host owns the window, surface, input and frame pacing; it feeds [`OverlayRenderer`] with
//! click events and cursor positions and asks it to render into a texture view.
//!
//! `shaders/physics.wgsl` is a prototype compute pass (spring chain + particles) that is **not**
//! dispatched yet; it is kept here as the starting point for GPU-side physics.

pub mod cursor;
pub mod renderer;

pub use cursor::{
    cursor_quad_vertices, CursorShape, CursorVertex, GpuCursorState,
};
pub use renderer::{
    build_layer_capsules, build_samples, CapsuleInstance, CircleInstance, ModeMask,
    OverlayRenderer, Sample, TrailChain, DEPTH_FORMAT,
};
