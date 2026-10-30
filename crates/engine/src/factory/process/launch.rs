//! The launch site (Milestone 12, the Rocketry tech): a 7×7×6 pad and gantry, an ordinary recipe machine
//! (`Pick::ByInput`, `Category::Launch`, 2 MW). Belts bring the five kinds of rocket part and fuel, and a satellite, in at
//! the back hatches; once it holds a rocket's worth (`recipes/orbit.rs`: 4 light frames, 2 engines, 2 fuel tanks, an
//! avionics, 8 fuel canisters, the satellite) a 60 s batch builds it, and when the batch ends the rocket is up:
//! `launches` counts them (saved for this block only), `lifted` hands the satellite to `Factory::update`, which puts it in
//! orbit (`satellites.rs`), and the empty canisters come out of the front. A half-fed pad names what it lacks
//! (`half_fed_text`); its readout also says how many of each kind are up and over the pad (`sky_line`).
//!
//! To make another launch: a `Category::Launch` recipe row whose last input is the satellite item (a `satellites::KINDS` row).

use crate::block::{tex, LAUNCH_SITE};
use crate::factory::Factory;
use crate::item::ItemId;
use crate::recipes::Category;
use crate::satellites::KINDS;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::{Energy, Extra, Pick, ProcessSpec, ProcessTier, Processor, Status};

#[cfg(test)]
mod tests;

pub const LAUNCH_SPEC: ProcessSpec = ProcessSpec {
    block: LAUNCH_SITE,
    categories: &[Category::Launch],
    pick: Pick::ByInput,
    // Light frames, engines, fuel tanks, avionics, fuel canisters and the satellite: one slot for each.
    buffers: [6, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 2000 }],
    footprint: Footprint {
        size: [7, 7, 6],
        ports: &[
            Port { side: Side::Back, role: Role::In, cell: Which::All },
            Port { side: Side::Front, role: Role::Out, cell: Which::All },
        ],
    },
    verb: "Building a rocket",
    products: "empty canisters",
    waiting: "Waiting for light frames, rocket engines, fuel tanks, avionics, rocket fuel canisters and a satellite",
    map_colour: 0xe8ecf4,
    compute: 0,
    parts: &LAUNCH_PARTS,
};

/// How many input slots a saved processor of `spec` wrote: launch sites made before save 45 had no slot for a satellite.
pub(super) fn saved_input_slots(spec: &ProcessSpec, version: u32) -> usize {
    let old_pad = spec.block == LAUNCH_SITE && version < 45;
    spec.buffers[0] - usize::from(old_pad)
}

impl Processor {
    /// The launch site's own words: how far along the rocket is while it builds. `None` for another machine or for any
    /// other state (a half-fed pad is `half_fed_text`'s).
    pub(super) fn launch_text(&self) -> Option<String> {
        if !self.is_launch() || self.status != Status::Working {
            return None;
        }
        let permille = self.batch_recipe().map_or(0, |r| self.progress / super::super::ticks(r.seconds));
        Some(format!("Building a rocket: {}% (launches when done)", permille.min(1000) / 10))
    }

    /// A readout line with the rockets this pad has sent up, once there are any.
    pub(super) fn launch_line(&self) -> Option<String> {
        match self.extra {
            Extra::Launch { launches, .. } if launches > 0 => Some(format!("{launches} in orbit")),
            _ => None,
        }
    }

    /// What is up in the sky, one line for each kind that has a satellite: how many, and how many cover this pad right now
    /// (starts with a newline); empty for another machine or a sky with nothing in it.
    pub(super) fn sky_line(&self, f: &Factory) -> String {
        if !self.is_launch() {
            return String::new();
        }
        let mut text = String::new();
        for (kind, def) in KINDS.iter().enumerate() {
            let up = f.orbit.count(kind as u8);
            if up > 0 {
                let over = f.orbit.covering(kind as u8, self.pos.x, self.pos.z, f.orbit.now);
                text += &format!("\n{} satellites: {} up, {} over this pad", def.name, up, over);
            }
        }
        text
    }

    /// The satellite its rocket carried up this tick, for Factory::update to put in orbit (once).
    pub(in crate::factory) fn take_lifted(&mut self) -> Option<ItemId> {
        match &mut self.extra {
            Extra::Launch { lifted, .. } => lifted.take(),
            _ => None,
        }
    }

    /// Whether this is a launch site (the one block that keeps a launch count).
    pub(super) fn is_launch(&self) -> bool {
        self.spec.block == LAUNCH_SITE
    }
}

const SLAB: Look = Look::Tex([tex::LAUNCH_TOP, tex::LAUNCH_SIDE, tex::FRAME]);
const GANTRY: Look = Look::Tex([tex::FRAME, tex::LAUNCH_SIDE, tex::FRAME]);

/// Across and deep 7, 6 high (±3.5, ±3.5, ±3) but for the tower: a concrete slab, a mount, a launch tower on the left with
/// two catch arms and a quick-disconnect arm reaching for the stack, and, while a batch runs, the rocket on the mount
/// (`rocket.rs`, standing far above the site); the status lamp sits at a corner.
const LAUNCH_PARTS: [Part; 9] = [
    part([0.0, -2.85, 0.0], [6.96, 0.3, 6.96], SLAB),
    part([0.0, -2.6, 0.0], [2.4, 0.2, 2.4], GANTRY),
    part([-2.9, 5.05, -0.4], [0.8, 15.5, 0.8], GANTRY),
    part([-1.85, 7.0, -0.95], [1.9, 0.26, 0.24], GANTRY),
    part([-1.85, 7.0, 0.15], [1.9, 0.26, 0.24], GANTRY),
    part([-1.9, 3.9, -0.4], [1.7, 0.22, 0.34], GANTRY),
    part([-2.9, 13.3, -0.4], [0.3, 1.0, 0.3], GANTRY),
    part([0.0, -2.5, 0.0], [1.3, 0.0, 1.3], Look::Rocket),
    part([3.2, -2.55, 3.2], [0.2, 0.2, 0.2], Look::Lamp),
];
