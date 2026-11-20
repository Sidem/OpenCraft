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

    /// The footprint outline's colour: green when the held long-distance pole links (or standalone), red when it won't, else 0 (amber).
    pub(crate) fn long_pole_outline(&self) -> i32 {
        if self.body().input.crouch && self.holds_long_pole() {
            return GHOST_GREEN;
        }
        match self.long_pole_link() {
            Some((_, LongLink::Links { .. })) => GHOST_GREEN,
            Some(_) => BLOCKED_RED,
            None => 0,
        }
    }

    /// An outline box (7 numbers) on the pole the held long-distance pole links to (blue), or on the long-distance pole
    /// it is measured from (red); empty otherwise.
    pub(crate) fn long_pole_boxes(&self) -> Vec<i32> {
        if self.body().input.crouch {
            return Vec::new();
        }
        let Some((to, colour)) = self.long_pole_link().and_then(|(_, link)| far_pole(link)) else { return Vec::new() };
        let cells = self.sim.factory.machine_cells(to);
        let lo = cells.iter().fold(to, |m, c| IVec3::new(m.x.min(c.x), m.y.min(c.y), m.z.min(c.z)));
        let hi = cells.iter().fold(to, |m, c| IVec3::new(m.x.max(c.x), m.y.max(c.y), m.z.max(c.z)));
        vec![lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, colour]
    }

    /// The label's status and controls for a long-distance pole ("" when none is held and aimed).
    pub(crate) fn long_pole_line(&self, blocked: usize) -> String {
        let Some((anchor, link)) = self.long_pole_link() else { return String::new() };
        let block_note = match blocked {
            0 => String::new(),
            1 => " · 1 cell in the way (red)".to_string(),
            n => format!(" · {n} cells in the way (red)"),
        };
        if self.body().input.crouch {
            let keys = if blocked == 0 {
                factory::hud_keys(&[("RMB", "Place"), ("Release Ctrl", "Auto-wire")])
            } else {
                factory::hud_keys(&[("Release Ctrl", "Auto-wire")])
            };
            return format!("Standalone · not wired to any grid{block_note}\n{keys}");
        }
        let away = |p: IVec3| far(p, anchor);
        let (status, linked) = match link {
            LongLink::Links { to, span, .. } => {
                let what = if span == LONG_STATS.link { "long-distance pole" } else { "power pole" };
                let d = away(to).floor() as i32;
                (format!("Links to {what} · {d} / {span} blocks ({} to spare)", span - d), true)
            }
            LongLink::TooFar(p) => (
                format!("Out of reach · nearest is {} blocks away (max {})", away(p).ceil() as i32, LONG_STATS.link),
                false,
            ),
            LongLink::Full(p) => (
                format!(
                    "Won't link · pole {} blocks away is full ({}/{})",
                    away(p).round() as i32,
                    LONG_STATS.slots,
                    LONG_STATS.slots
                ),
                false,
            ),
            LongLink::Unpowered(p) => {
                (format!("Won't link · pole {} blocks away has no power", away(p).round() as i32), false)
            }
            LongLink::Alone => (format!("Won't link · no powered pole within {LONG_LOCAL_LINK} blocks"), false),
        };
        let keys = match (blocked == 0, linked) {
            (true, true) => factory::hud_keys(&[("RMB", "Place"), ("Hold Ctrl", "Unconnected")]),
            (true, false) => factory::hud_keys(&[("RMB", "Place")]),
            (false, _) => String::new(),
        };
        if keys.is_empty() {
            format!("{status}{block_note}")
        } else {
            format!("{status}{block_note}\n{keys}")
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
        if self.body().input.crouch {
            return;
        }
        let Some((to, _)) = far_pole(link) else { return };
        let Some(far_mid) = self.sim.factory.pole_top(to) else { return };
        let top_mid = factory::long_top(anchor);
        // The wire runs from an arm tip of one tower to an arm tip of the other (`pole_hang.rs`); the beacon stays on top.
        let Some(from) = self.sim.factory.pole_hang(to, top_mid) else { return };
        let top = factory::ghost_hang(anchor, LONG_TIER, 0, far_mid);
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
            let centre = far_mid + Vec3::new(0.0, 0.3 + f64::from(BEACON_HEIGHT) / 2.0, 0.0) - eye;
            let width = ((far_mid - eye).length() * BEACON_WIDTH).max(BEACON_MIN_WIDTH) as f32;
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
