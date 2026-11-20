//! X-ray goggles' view: what the player built, drawn through the ground. Presentation only: wearing the goggles is core
//! state (`Inventory::has_xray_goggles`, the Head slot), but whether the view is switched on is a local setting the host
//! sends (`set_xray_enabled`), never saved and never in the state hash.
//!
//! Each frame, while the view is active, the factory's box records (the span `frame.rs` wrote between the avatars and the
//! wires, so dropped items, players and previews are never in it) within [`RADIUS`] of the camera are copied into
//! `Xray::boxes` and its round bodies into `Xray::round`, capped at [`MAX_RECORDS`]. The host draws them after the
//! terrain and before the solid boxes, with the depth test set to *greater*, so only what the terrain hides shows
//! (`web/src/render/boxes.ts` `drawXray`). Nothing here touches chunks or meshes, and with the goggles off no work is
//! done. The view is off in the strategy view, whose record list is compacted (`strategy.rs`).
//! To change the look: the radius and fade start here, the tint and flat shading in `render/boxes.ts` and `shaders.ts`.

use std::ops::Range;

use crate::factory::{copy_near, INSTANCE_FLOATS};
use crate::Game;

/// Blocks around the camera within which buried machines show.
pub const RADIUS: f32 = 28.0;
/// The share of [`RADIUS`] where the view starts to fade out, so the edge is soft, not a wall.
pub const FADE_FROM: f32 = 0.6;
/// The most records one frame copies (a record is 64 bytes): a dense base near the camera must not cost a frame.
const MAX_RECORDS: usize = 24_000;

pub(crate) struct Xray {
    /// The player's choice (default on, so goggles just put on work at once).
    pub enabled: bool,
    pub boxes: Vec<f32>,
    pub round: Vec<f32>,
    /// Whether the last frame hit [`MAX_RECORDS`] (a test and the bench read it).
    pub capped: bool,
}

impl Default for Xray {
    fn default() -> Self {
        Xray { enabled: true, boxes: Vec::new(), round: Vec::new(), capped: false }
    }
}

impl Game {
    /// Fills `Xray::boxes` and `round` from `factory_span` of the frame's dynamic instances plus cached static chunks
    /// near `eye`, or empties them when inactive. Call after `light_boxes` and before anything reorders `instances`.
    pub(crate) fn write_xray(&mut self, factory_span: Range<usize>, eye: crate::math::Vec3) {
        if !self.xray_active() {
            self.xray.boxes.clear();
            self.xray.round.clear();
            self.xray.capped = false;
            return;
        }
        let span = &self.instances[factory_span.start * INSTANCE_FLOATS..factory_span.end * INSTANCE_FLOATS];
        let x = &mut self.xray;
        x.capped = copy_near(span, RADIUS, MAX_RECORDS, &mut x.boxes, &mut x.round);
        if !x.capped {
            x.capped = self.static_boxes.copy_xray_near(eye, RADIUS, MAX_RECORDS, &mut x.boxes, &mut x.round);
        }
    }
}

#[cfg(test)]
mod tests;
