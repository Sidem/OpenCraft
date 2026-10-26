//! Placing a long-distance pole (the local player's, presentation only). It is placed like a multi-block machine
//! (`footprint_preview.rs`), and the core wires it to `Factory::auto_hook`'s pole; this shows beforehand where that
//! would be, from `Factory::long_link`:
//!
//! - the footprint outline is green when it links and red when it won't, and the pole it links to is boxed blue (the
//!   long-distance pole it measures from, red) (`long_pole_outline`, `long_pole_boxes`, both from `api/hud.rs`);
//! - the label's second line says to which pole, how far and how much reach is to spare, or why not (`long_pole_line`);
//! - a ghost tower with its wire in the grid's colour, or a red wire that stops at full reach, and a light pillar over
//!   the far long-distance pole that shows through the fog, as it is often out of sight (`write_long_pole_preview`);
//! - a power-up or power-down blip as the spot comes into reach or leaves it (`update_long_pole_cue`);
//! - while one is held, the maps ring every long-distance pole's reach (`holds_long_pole`, `minimap/marks.rs`).

use crate::block::{tex, LONG_POLE};
use crate::factory::{self, push_box, LongLink, LONG_LOCAL_LINK, LONG_STATS, LONG_TIER};
use crate::math::{IVec3, Vec3};
use crate::{sound, Game};

use super::{dist2, ANCHOR_BLUE, BLOCKED_RED, GHOST_GREEN};

/// The light pillar over the far long-distance pole: height in blocks, and width in blocks per block away (so it stays a
/// few pixels wide however far it is), at least `BEACON_MIN_WIDTH`.
const BEACON_HEIGHT: f32 = 48.0;
const BEACON_WIDTH: f64 = 0.008;
const BEACON_MIN_WIDTH: f64 = 0.3;
/// How loud the blip at the edge of reach is (0..1).
const CUE_VOLUME: f64 = 0.35;

impl Game {
    /// Whether a long-distance pole is in the selected slot.
    pub(crate) fn holds_long_pole(&self) -> bool {
        self.held_footprint().is_some_and(|f| f.0 == LONG_POLE)
    }

    /// The anchor of the held long-distance pole's spot and what it would be wired to there; `None` when none is held
    /// and aimed, and for the tick after a change (the grids are stale).
    pub(crate) fn long_pole_link(&self) -> Option<(IVec3, LongLink)> {
        let (block, anchor, _, _) = self.footprint_ghost()?;
        if block != LONG_POLE {
            return None;
        }
        Some((anchor, self.sim.factory.long_link(anchor)?))
    }

    /// The footprint outline's colour: green when the held long-distance pole links, red when it won't, else 0 (amber).
    pub(crate) fn long_pole_outline(&self) -> i32 {
        match self.long_pole_link() {
            Some((_, LongLink::Links { .. })) => GHOST_GREEN,
            Some(_) => BLOCKED_RED,
            None => 0,
        }
    }

    /// An outline box (7 numbers) on the pole the held long-distance pole links to (blue), or on the long-distance pole
    /// it is measured from (red); empty otherwise.
    pub(crate) fn long_pole_boxes(&self) -> Vec<i32> {
        let Some((to, colour)) = self.long_pole_link().and_then(|(_, link)| far_pole(link)) else { return Vec::new() };
        let cells = self.sim.factory.machine_cells(to);
        let lo = cells.iter().fold(to, |m, c| IVec3::new(m.x.min(c.x), m.y.min(c.y), m.z.min(c.z)));
        let hi = cells.iter().fold(to, |m, c| IVec3::new(m.x.max(c.x), m.y.max(c.y), m.z.max(c.z)));
        vec![lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, colour]
    }

    /// The label's line about linking ("" when no long-distance pole is held and aimed): "Links to the long-distance
    /// pole 132 blocks away · 28 to spare", or why it won't link.
    pub(crate) fn long_pole_line(&self) -> String {
        let Some((anchor, link)) = self.long_pole_link() else { return String::new() };
        let away = |p: IVec3| far(p, anchor);
        match link {
            LongLink::Links { to, span, .. } => {
                let what = if span == LONG_STATS.link { "long-distance pole" } else { "power pole" };
                let d = away(to).floor() as i32;
                format!("Links to the {what} {d} blocks away · {} to spare", span - d)
            }
            LongLink::TooFar(p) => format!(
                "Out of reach: the nearest long-distance pole is {} blocks away, {} at most",
                away(p).ceil() as i32,
                LONG_STATS.link
            ),
            LongLink::Full(p) => format!(
                "Won't link: the long-distance pole {} blocks away has all {} connections used",
                away(p).round() as i32,
                LONG_STATS.slots
            ),
            LongLink::Unpowered(p) => {
                format!("Won't link: the long-distance pole {} blocks away has no power yet", away(p).round() as i32)
            }
            LongLink::Alone => format!("Won't link: no powered pole within {LONG_LOCAL_LINK} blocks"),
        }
    }

    /// The ghost tower (when its cells are free), its wire and the light pillar over the far long-distance pole, drawn
    /// with the machines; `eye` is the camera.
    pub(crate) fn write_long_pole_preview(&mut self, eye: Vec3) {
        let Some((anchor, link)) = self.long_pole_link() else { return };
        let free = self.footprint_ghost().is_some_and(|g| g.3.iter().all(|c| !c.1));
        if free {
            factory::preview_pole(&mut self.instances, LONG_TIER, 0, anchor.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye);
        }
        let Some((to, _)) = far_pole(link) else { return };
        let Some(from) = self.sim.factory.pole_top(to) else { return };
        let top = factory::long_top(anchor);
        let layer = if let LongLink::Links { layer, .. } = link { layer } else { tex::LAMP_RED };
        // Too far: the wire runs out at full reach, short of the ghost. Full or unpowered: no wire.
        let end = match link {
            LongLink::Links { .. } => Some(top),
            LongLink::TooFar(_) => Some(from + (top - from) * (f64::from(LONG_STATS.link) / far(to, anchor))),
            _ => None,
        };
        if let Some(end) = end {
            factory::preview_wire(&mut self.instances, from - eye, end - eye, layer);
        }
        let long = !matches!(link, LongLink::Links { span, .. } if span != LONG_STATS.link);
        if long {
            let centre = from + Vec3::new(0.0, 0.3 + f64::from(BEACON_HEIGHT) / 2.0, 0.0) - eye;
            let width = ((from - eye).length() * BEACON_WIDTH).max(BEACON_MIN_WIDTH) as f32;
            let size = [width, BEACON_HEIGHT, width];
            push_box(&mut self.instances, centre, 0.0, size, 0.0, [layer; 3], false);
            factory::mark_beacon(&mut self.instances);
        }
    }

    /// Plays a blip when the held long-distance pole's spot comes into reach (power up) or leaves it (power down).
    pub(crate) fn update_long_pole_cue(&mut self) {
        let Some((_, anchor, _, _)) = self.footprint_ghost().filter(|g| g.0 == LONG_POLE) else {
            self.tools.long_cue = None;
            return;
        };
        // Stale grids for a tick: keep what was heard.
        let Some(link) = self.sim.factory.long_link(anchor) else { return };
        let links = matches!(link, LongLink::Links { .. });
        if self.tools.long_cue.is_some_and(|was| was != links) {
            let kind = if links { sound::POWER_UP } else { sound::POWER_DOWN };
            self.play(kind, 0, self.body().eye(), CUE_VOLUME);
        }
        self.tools.long_cue = Some(links);
    }
}

/// Blocks between two anchors.
fn far(a: IVec3, b: IVec3) -> f64 {
    f64::from(dist2(a, b)).sqrt()
}

/// The pole a link points at, with its box colour: the pole it links to (blue) or the long-distance pole it doesn't
/// (red); `None` when there is none.
fn far_pole(link: LongLink) -> Option<(IVec3, i32)> {
    match link {
        LongLink::Links { to, .. } => Some((to, ANCHOR_BLUE)),
        LongLink::TooFar(p) | LongLink::Full(p) | LongLink::Unpowered(p) => Some((p, BLOCKED_RED)),
        LongLink::Alone => None,
    }
}
