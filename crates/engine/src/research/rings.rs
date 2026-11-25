//! Frontier-map structure (`docs/TECH_TREE.md` section 10): the six [`Ring`]s, twelve [`Wedge`]s, red-ring [`Deed`]s,
//! and the ring-threshold & pack-milestone rules that replaced the four Science gate techs.
//!
//! - **Rings** open by milestone: Red from the start; Green, Blue, Violet and Gold when their first science pack is
//!   crafted/produced; Sky on the first rocket launch.
//! - **Pack recipes** unlock once at least half of the preceding ring is researched (`done * 2 >= total`) and the
//!   pack's ingredient techs are done.
//! - **Deeds** (Red ring) finish by doing the job (`BELT_DEED_TARGET` belts laid, or a connected belt climb of
//!   `CLIMB_DEED_STEPS` vertical blocks) and cannot be lab-researched.

use std::collections::{HashSet, VecDeque};

use crate::block::BELT;
use crate::item::{ItemId, BLUE_PACK, GOLD_PACK, GREEN_PACK, VIOLET_PACK};
use crate::math::IVec3;
use crate::recipes::{BLUE_RECIPES, GOLD_RECIPES, VIOLET_RECIPES};
use crate::sim::SimEvent;

use super::ids::*;
use super::{Research, TechState, Unlock, TECHS};

/// Belts laid to complete the `Belt Routing` deed.
pub const BELT_DEED_TARGET: u32 = 40;
/// Vertical blocks climbed by one connected belt line to complete the `Belt Lifts` deed.
pub const CLIMB_DEED_STEPS: i32 = 3;

/// Era rings from centre outward (`docs/TECH_TREE.md` section 10).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum Ring {
    Red = 0,
    Green = 1,
    Blue = 2,
    Violet = 3,
    Gold = 4,
    Sky = 5,
}

pub const RING_COUNT: usize = 6;

impl Ring {
    pub const fn bit(self) -> u8 {
        1 << (self as u8)
    }

    pub const fn from_u8(v: u8) -> Option<Ring> {
        match v {
            0 => Some(Ring::Red),
            1 => Some(Ring::Green),
            2 => Some(Ring::Blue),
            3 => Some(Ring::Violet),
            4 => Some(Ring::Gold),
            5 => Some(Ring::Sky),
            _ => None,
        }
    }

    pub const fn prev(self) -> Option<Ring> {
        match self {
            Ring::Red => None,
            Ring::Green => Some(Ring::Red),
            Ring::Blue => Some(Ring::Green),
            Ring::Violet => Some(Ring::Blue),
            Ring::Gold => Some(Ring::Violet),
            Ring::Sky => Some(Ring::Gold),
        }
    }

    /// What the research screen calls the ring.
    pub const fn name(self) -> &'static str {
        match self {
            Ring::Red => "Red",
            Ring::Green => "Green",
            Ring::Blue => "Blue",
            Ring::Violet => "Violet",
            Ring::Gold => "Gold",
            Ring::Sky => "Sky",
        }
    }

    /// The science pack whose first craft/production opens this ring (`None` for Red and Sky).
    pub const fn milestone_pack(self) -> Option<ItemId> {
        match self {
            Ring::Green => Some(GREEN_PACK),
            Ring::Blue => Some(BLUE_PACK),
            Ring::Violet => Some(VIOLET_PACK),
            Ring::Gold => Some(GOLD_PACK),
            Ring::Red | Ring::Sky => None,
        }
    }
}

/// Twelve disciplines clockwise from the top (`docs/TECH_TREE.md` section 10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Wedge {
    Logistics = 0,
    Workshop = 1,
    OresMetals = 2,
    Chemistry = 3,
    Power = 4,
    Computing = 5,
    RobotsDrones = 6,
    YourGear = 7,
    Survey = 8,
    #[allow(dead_code)]
    Sea = 9,
    TownsLife = 10,
    Building = 11,
}

/// A tech completed by doing something in the world instead of lab research.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Deed {
    /// Lay `target` belts in the world.
    LayBelts(u32),
    /// Run a connected belt line up at least `steps` vertical blocks.
    ClimbBelt(i32),
}

impl Deed {
    /// The job in a few words, for the research screen ("Lay 40 belts").
    pub fn goal(self) -> String {
        match self {
            Deed::LayBelts(n) => format!("Lay {n} belts"),
            Deed::ClimbBelt(n) => format!("Run one belt line up {n} blocks"),
        }
    }
}

/// Which ring's threshold gates `unlock` (the four pack recipes that replaced the Science techs).
pub fn pack_recipe_ring(unlock: Unlock) -> Option<Ring> {
    match unlock {
        Unlock::Recipe(GREEN_PACK) => Some(Ring::Green),
        Unlock::MachineRecipe(r) if r == BLUE_RECIPES[0] => Some(Ring::Blue),
        Unlock::MachineRecipe(r) if r == VIOLET_RECIPES[0] => Some(Ring::Violet),
        Unlock::MachineRecipe(r) if r == GOLD_RECIPES[0] => Some(Ring::Gold),
        _ => None,
    }
}

/// The techs that must be done for a ring's pack ingredients to be known.
pub const fn pack_part_techs(ring: Ring) -> &'static [u8] {
    match ring {
        Ring::Green => &[], // belts and screws are available from the start
        Ring::Blue => &[ASSEMBLY, STEELMAKING],
        Ring::Violet => &[ELECTRONICS, ASSEMBLY, STEELMAKING],
        Ring::Gold => &[PLASTICS, BAUXITE_PROCESSING, PROCESSORS],
        Ring::Red | Ring::Sky => &[],
    }
}

/// Whether `tech` is completed by a deed rather than labs.
pub fn is_deed(tech: u8) -> bool {
    TECHS.get(tech as usize).is_some_and(|t| t.deed.is_some())
}

impl Research {
    /// Whether `ring` is open (Red is always open).
    pub fn ring_open(&self, ring: Ring) -> bool {
        ring == Ring::Red || (self.open_rings & ring.bit()) != 0
    }

    /// Opens `ring` (and every earlier ring, so saves and tests stay monotone).
    pub fn open_ring(&mut self, ring: Ring) {
        let mut r = Some(ring);
        let before = self.open_rings;
        while let Some(cur) = r {
            self.open_rings |= cur.bit();
            r = cur.prev();
        }
        if self.open_rings != before {
            self.refresh();
        }
    }

    /// `(done, total)` non-endless techs in `ring`.
    pub fn ring_progress(&self, ring: Ring) -> (u32, u32) {
        let (mut done, mut total) = (0u32, 0u32);
        for (i, t) in TECHS.iter().enumerate() {
            if t.ring == ring && !super::is_bonus(i as u8) {
                total += 1;
                if self.progress[i] >= t.units {
                    done += 1;
                }
            }
        }
        (done, total)
    }

    /// Whether `ring`'s science pack recipe is unlocked: already open, OR at least half of the previous ring is done
    /// and the pack's ingredient techs are done.
    pub fn ring_pack_unlocked(&self, ring: Ring) -> bool {
        if self.ring_open(ring) {
            return true;
        }
        let Some(prev) = ring.prev() else { return true };
        let (done, total) = self.ring_progress(prev);
        let half = total > 0 && done * 2 >= total;
        half && pack_part_techs(ring).iter().all(|&t| self.state(t) == TechState::Done)
    }

    /// Progress toward a deed tech (`belts_laid` for `LayBelts`, 0/1 for `ClimbBelt`).
    pub fn deed_progress(&self, tech: u8) -> u32 {
        match TECHS.get(tech as usize).and_then(|t| t.deed) {
            Some(Deed::LayBelts(n)) => self.belts_laid.min(n),
            Some(Deed::ClimbBelt(_)) => u32::from(self.progress(tech) >= TECHS[tech as usize].units),
            None => 0,
        }
    }

    /// Completes a deed tech immediately (idempotent).
    pub fn complete_deed(&mut self, tech: u8) {
        let i = tech as usize;
        if i < TECHS.len() && self.progress[i] < TECHS[i].units {
            self.progress[i] = TECHS[i].units;
            self.refresh();
        }
    }

    /// Watches this tick's core events to complete red deeds and open ring milestones.
    pub fn note(&mut self, events: &[SimEvent], belt_positions: &[IVec3]) {
        for &ev in events {
            match ev {
                SimEvent::Crafted { item, .. } | SimEvent::Produced { item, .. } | SimEvent::Gained { item, .. } => {
                    for ring in [Ring::Green, Ring::Blue, Ring::Violet, Ring::Gold] {
                        if ring.milestone_pack() == Some(item) && !self.ring_open(ring) {
                            self.open_ring(ring);
                        }
                    }
                }
                SimEvent::Launched { .. } => {
                    if !self.ring_open(Ring::Sky) {
                        self.open_ring(Ring::Sky);
                    }
                }
                SimEvent::BlockPlaced { pos, block, .. } if block == BELT => {
                    if self.state(BELT_ROUTING) != TechState::Done {
                        self.belts_laid = (self.belts_laid + 1).min(BELT_DEED_TARGET);
                        if self.belts_laid >= BELT_DEED_TARGET {
                            self.complete_deed(BELT_ROUTING);
                        }
                    }
                    if self.state(BELT_LIFTS) != TechState::Done
                        && belt_climb_span(pos, belt_positions) >= CLIMB_DEED_STEPS
                    {
                        self.complete_deed(BELT_LIFTS);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Vertical span (`max_y - min_y`) of the connected belt component containing `start` (horizontal or ±1 y neighbours).
/// A set lookup per neighbour, so a big base costs one pass over its belts, not one per belt.
fn belt_climb_span(start: IVec3, belts: &[IVec3]) -> i32 {
    let mut left: HashSet<IVec3> = belts.iter().copied().collect();
    if !left.remove(&start) {
        return 0;
    }
    let mut q = VecDeque::from([start]);
    let (mut min_y, mut max_y) = (start.y, start.y);
    while let Some(p) = q.pop_front() {
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
        if max_y - min_y >= CLIMB_DEED_STEPS {
            break;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for dy in -1..=1 {
                let n = p + IVec3::new(dx, dy, dz);
                if left.remove(&n) {
                    q.push_back(n);
                }
            }
        }
    }
    max_y - min_y
}
