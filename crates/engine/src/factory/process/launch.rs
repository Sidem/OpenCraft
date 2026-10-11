//! The launch site (Milestone 12, the Rocketry tech): a 7×7×6 pad and gantry, an ordinary recipe machine
//! (`Pick::ByInput`, `Category::Launch`, 2 MW). Belts bring the five kinds of rocket part and fuel in at the back hatches;
//! once it holds a rocket's worth (`recipes/orbit.rs`: 4 light frames, 2 engines, 2 fuel tanks, an avionics, 8 fuel
//! canisters) a 60 s batch builds it, and when the batch ends the rocket is in orbit: `launches` counts them (saved for this
//! block only) and the empty canisters come out of the front. A half-fed pad names what it lacks (`half_fed_text`).
//!
//! Orbit state beyond the count (what a rocket carries, where it is) arrives with the satellites (step 12.2).
//! To make another launch: a `Category::Launch` recipe row.

use crate::block::{tex, LAUNCH_SITE};
use crate::recipes::Category;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{drum_x, drum_z, part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

#[cfg(test)]
mod tests;

pub const LAUNCH_SPEC: ProcessSpec = ProcessSpec {
    block: LAUNCH_SITE,
    categories: &[Category::Launch],
    pick: Pick::ByInput,
    // Light frames, engines, fuel tanks, avionics and fuel canisters: one slot for each.
    buffers: [5, 0, 1],
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
    waiting: "Waiting for light frames, rocket engines, fuel tanks, avionics and rocket fuel canisters",
    map_colour: 0xe8ecf4,
    compute: 0,
    parts: &LAUNCH_PARTS,
};

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
        (self.is_launch() && self.launches > 0).then(|| format!("{} in orbit", self.launches))
    }

    /// Whether this is a launch site (the one block that keeps a launch count).
    pub(super) fn is_launch(&self) -> bool {
        self.spec.block == LAUNCH_SITE
    }
}

const SLAB: Look = Look::Tex([tex::LAUNCH_TOP, tex::LAUNCH_SIDE, tex::FRAME]);
const GANTRY: Look = Look::Tex([tex::FRAME, tex::LAUNCH_SIDE, tex::FRAME]);
const HULL: Look = Look::Rocket([tex::FRAME, tex::ROCKET_HULL, tex::FRAME]);
const RED: Look = Look::Rocket([tex::LAMP_RED; 3]);

/// Across and deep 7, 6 high (±3.5, ±3.5, ±3): a concrete slab, a mount, a gantry tower with two arms on the left and, while
/// a batch runs, the rocket (hull, nose, bell and fins) standing on the mount; the status lamp sits at a corner.
const LAUNCH_PARTS: [Part; 12] = [
    part([0.0, -2.85, 0.0], [6.96, 0.3, 6.96], SLAB),
    part([0.0, -2.6, 0.0], [2.4, 0.2, 2.4], GANTRY),
    part([-2.9, 0.0, -0.4], [0.6, 5.6, 0.6], GANTRY),
    part([-1.75, 1.3, -0.4], [2.1, 0.2, 0.3], GANTRY),
    part([-1.75, -0.7, -0.4], [2.1, 0.2, 0.3], GANTRY),
    drum_x([0.0, 0.1, 0.0], [1.2, 4.6, 1.2], HULL),
    drum_z([0.0, 0.1, 0.0], [1.2, 4.6, 1.2], HULL),
    part([0.0, 2.7, 0.0], [0.7, 0.6, 0.7], RED),
    part([0.0, -2.35, 0.0], [0.9, 0.3, 0.9], Look::Rocket([tex::FRAME, tex::ROCKET_ENGINE, tex::FRAME])),
    part([0.0, -1.7, 0.0], [2.2, 0.9, 0.1], RED),
    part([0.0, -1.7, 0.0], [0.1, 0.8, 2.2], RED),
    part([3.2, -2.55, 3.2], [0.2, 0.2, 0.2], Look::Lamp),
];
