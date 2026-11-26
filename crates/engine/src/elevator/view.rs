//! Drawing the elevator's tether (presentation only; the climb is core state: `mod.rs`). A few tall boxes over the
//! anchor, appended to the frame's box instances: the steel core of the sections built with four rails round it, a red
//! lamp collar at each section's top (lamps glow, so the tower stands lit at night), and, while a section climbs, a
//! see-through scaffold growing with its order and the climber car riding on top of it. Lit as open sky, since most of
//! it stands above every loaded cell. To change the look: the constants and `write_elevator`.

use crate::block::{tex, ELEVATOR_ANCHOR};
use crate::factory::{light_boxes, push_box};
use crate::math::Vec3;
use crate::settlements::{KINDS, STAGE_NAMES};
use crate::world::DAYLIGHT;
use crate::Game;

use super::{Unfit, SECTION_HEIGHT};

/// Width of the tether's core and how far its rails stand from the middle, in blocks.
const CORE: f32 = 1.2;
const RAIL: f64 = 1.1;
/// The anchor's height: the tether starts on its top.
const ANCHOR_TOP: f64 = 2.0;

impl Game {
    /// Where the held anchor is aimed: the town it would stand by, or why it can't stand there; `None` when no anchor
    /// is held and aimed. The core asks `elevator_site` again when it is placed.
    pub(crate) fn anchor_site(&self) -> Option<Result<u16, Unfit>> {
        let (block, anchor, _, _) = self.footprint_ghost()?;
        (block == ELEVATOR_ANCHOR).then(|| self.sim.elevator_site(anchor))
    }

    /// The placement label's line for the held anchor: "Stands by your basalt forge (a city)", or why it can't.
    pub(crate) fn anchor_line(&self) -> Option<(String, bool)> {
        Some(match self.anchor_site()? {
            Ok(i) => {
                let t = &self.sim.settlements.towns[i as usize];
                let kind = KINDS[t.kind as usize].name.to_lowercase();
                (format!("Stands by your {kind} (a {})", STAGE_NAMES[t.built as usize]), true)
            }
            Err(why) => (why.text().to_string(), false),
        })
    }

    /// Appends the tether over the elevator's anchor to the box instances (after they are lit: it lights itself).
    pub(crate) fn write_elevator(&mut self, eye: Vec3) {
        let Some(e) = &self.sim.elevator else { return };
        let from = self.instances.len();
        let base = self.sim.factory.base_middle(e.pos) + Vec3::new(0.0, ANCHOR_TOP, 0.0) - eye;
        let out = &mut self.instances;
        let built = f64::from(e.built) * f64::from(SECTION_HEIGHT);
        if built > 0.0 {
            let mid = base + Vec3::new(0.0, built / 2.0, 0.0);
            push_box(out, mid, 0.0, [CORE, built as f32, CORE], 0.0, [tex::STEEL; 3], true);
            for (x, z) in [(RAIL, 0.0), (-RAIL, 0.0), (0.0, RAIL), (0.0, -RAIL)] {
                let at = mid + Vec3::new(x, 0.0, z);
                push_box(out, at, 0.0, [0.3, built as f32, 0.3], 0.0, [tex::STEEL_BEAM; 3], true);
            }
            for s in 1..=e.built {
                let top = base + Vec3::new(0.0, f64::from(s) * f64::from(SECTION_HEIGHT), 0.0);
                push_box(out, top, 0.0, [3.0, 0.5, 3.0], 0.0, [tex::STEEL, tex::LAMP_RED, tex::STEEL], false);
            }
        }
        if !e.done() && e.progress() > 0.0 {
            let rise = f64::from(e.progress()) * f64::from(SECTION_HEIGHT);
            let mid = base + Vec3::new(0.0, built + rise / 2.0, 0.0);
            push_box(out, mid, 0.0, [2.4, rise as f32, 2.4], 0.0, [tex::FRAME; 3], true);
            let car = base + Vec3::new(0.0, built + rise + 0.8, 0.0);
            push_box(out, car, 0.0, [2.2, 1.6, 2.2], 0.0, [tex::LAMP_BLUE, tex::STEEL, tex::STEEL], false);
        }
        light_boxes(&mut self.instances[from..], eye, |_| DAYLIGHT);
    }
}
