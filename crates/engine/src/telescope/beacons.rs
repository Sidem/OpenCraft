//! The pins as columns of light, drawn while the telescope is up: one tinted beacon box per pin (`factory::mark_beacon_tinted`,
//! lit by itself and not hidden by fog) standing on its ground. A column grows taller and wider with its distance so it
//! stays a few pixels wide, as the long-distance pole's pillar does (`power_tools/long_pole.rs`).
//!
//! Past `REACH_VIEWS` view distances a column would be cut off by the camera's far plane (`render/renderer.ts`: 1.8
//! view distances and a bit), so it is drawn on the same line of sight at that distance, its position and size scaled
//! together about the eye: it covers the same angle on the screen, whatever the real distance. Loaded terrain is always
//! nearer than that, so a column behind a hill is still hidden by it.
//! To tune the look: `HEIGHT_*`, `WIDTH_*`.

use crate::block::tex;
use crate::factory::{mark_beacon_tinted, push_box};
use crate::math::Vec3;
use crate::Game;

use super::ground::column_top;

/// The column's height in blocks: this plus a share of its distance, at most `HEIGHT_MAX`.
pub(super) const HEIGHT_MIN: f64 = 64.0;
pub(super) const HEIGHT_PER_BLOCK: f64 = 0.1;
pub(super) const HEIGHT_MAX: f64 = 400.0;
/// Its width in blocks: a share of its distance, at least `WIDTH_MIN`.
const WIDTH_PER_BLOCK: f64 = 0.004;
const WIDTH_MIN: f64 = 0.5;
/// Columns farther than this many view distances are drawn at this distance.
const REACH_VIEWS: f64 = 1.5;

impl Game {
    /// Writes every pin's column onto the box instances (`eye` is the camera); nothing unless the telescope is up.
    pub(crate) fn write_scope_beacons(&mut self, eye: Vec3) {
        if !self.scope_active() {
            return;
        }
        let reach = self.sim.world.view_distance() * REACH_VIEWS;
        for pin in &self.scope.pins {
            let top = pin.y.unwrap_or_else(|| column_top(&self.sim.world, pin.x, pin.z).0);
            let base = Vec3::new(f64::from(pin.x) + 0.5, f64::from(top + 1), f64::from(pin.z) + 0.5);
            push_beacon(&mut self.instances, base - eye, reach, pin.rgb);
        }
    }
}

/// One column standing at `rel` (its foot, relative to the eye) in colour `rgb`; `reach` is the farthest it is drawn at.
pub(super) fn push_beacon(out: &mut Vec<f32>, rel: Vec3, reach: f64, rgb: u32) {
    let d = rel.length();
    let width = (d * WIDTH_PER_BLOCK).max(WIDTH_MIN);
    let height = (HEIGHT_MIN + d * HEIGHT_PER_BLOCK).min(HEIGHT_MAX);
    let k = if d > reach { reach / d } else { 1.0 };
    let centre = (rel + Vec3::new(0.0, height / 2.0, 0.0)) * k;
    let size = [(width * k) as f32, (height * k) as f32, (width * k) as f32];
    push_box(out, centre, 0.0, size, 0.0, [tex::LAMP; 3], false);
    mark_beacon_tinted(out, rgb);
}
