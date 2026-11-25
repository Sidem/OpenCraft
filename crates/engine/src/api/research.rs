//! The research screen: the tech tree (`research::TECHS`), the world's progress through it, and
//! choosing what every lab researches.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::item::ItemId;
use crate::item::RED_PACK;
use crate::research::{is_bonus, is_deed, pack_part_techs, Deed, Ring, TechState, MAX_QUEUE, RING_COUNT, TECHS};
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn tech_count(&self) -> u32 {
        TECHS.len() as u32
    }

    pub fn tech_name(&self, t: u32) -> String {
        TECHS.get(t as usize).map_or_else(String::new, |x| x.name.to_string())
    }

    pub fn tech_blurb(&self, t: u32) -> String {
        TECHS.get(t as usize).map_or_else(String::new, |x| x.blurb.to_string())
    }

    /// Techs that must be done first.
    pub fn tech_needs(&self, t: u32) -> Vec<u8> {
        TECHS.get(t as usize).map_or_else(Vec::new, |x| x.needs.to_vec())
    }

    /// Science packs each unit uses (one of each).
    pub fn tech_packs(&self, t: u32) -> Vec<u16> {
        TECHS.get(t as usize).map_or_else(Vec::new, |x| x.packs.iter().map(|p| p.0).collect())
    }

    pub fn tech_units(&self, t: u32) -> u32 {
        TECHS.get(t as usize).map_or(0, |x| x.units)
    }

    /// Lab seconds the next unit takes at full power (an endless tech's grows with its level).
    pub fn tech_seconds(&self, t: u32) -> f64 {
        let tech = t.min(u8::MAX as u32) as u8;
        if (tech as usize) < TECHS.len() {
            self.sim.factory.research.unit_seconds(tech)
        } else {
            0.0
        }
    }

    /// Whether it is an endless bonus tech: progress is a level, and there are no packs.
    pub fn tech_endless(&self, t: u32) -> bool {
        is_bonus(t.min(u8::MAX as u32) as u8)
    }

    /// Era ring index (0 Red .. 5 Sky) of tech `t`.
    pub fn tech_ring(&self, t: u32) -> u8 {
        TECHS.get(t as usize).map_or(0, |x| x.ring as u8)
    }

    /// Discipline wedge index (0 Logistics .. 11 Building) of tech `t`.
    pub fn tech_wedge(&self, t: u32) -> u8 {
        TECHS.get(t as usize).map_or(0, |x| x.wedge as u8)
    }

    /// Whether tech `t` is a deed (done by doing, not lab-researched).
    pub fn tech_deed(&self, t: u32) -> bool {
        is_deed(t.min(u8::MAX as u32) as u8)
    }

    /// Progress toward a deed tech (e.g. belts laid so far).
    pub fn tech_deed_progress(&self, t: u32) -> u32 {
        self.sim.factory.research.deed_progress(t.min(u8::MAX as u32) as u8)
    }

    /// Target count for a deed tech (0 if not a deed).
    pub fn tech_deed_target(&self, t: u32) -> u32 {
        match TECHS.get(t as usize).and_then(|x| x.deed) {
            Some(Deed::LayBelts(n)) => n,
            Some(Deed::ClimbBelt(_)) => 1,
            None => 0,
        }
    }

    /// Whether ring `r` (0 Red .. 5 Sky) is open.
    pub fn ring_open(&self, r: u8) -> bool {
        Ring::from_u8(r).is_some_and(|ring| self.sim.factory.research.ring_open(ring))
    }

    /// Whether ring `r`'s science pack recipe is unlocked.
    pub fn ring_pack_unlocked(&self, r: u8) -> bool {
        Ring::from_u8(r).is_some_and(|ring| self.sim.factory.research.ring_pack_unlocked(ring))
    }

    /// How many rings there are (the research screen's bands).
    pub fn ring_count(&self) -> u32 {
        RING_COUNT as u32
    }

    /// Ring `r`'s name ("Green"), or empty.
    pub fn ring_name(&self, r: u8) -> String {
        Ring::from_u8(r).map_or_else(String::new, |ring| ring.name().to_string())
    }

    /// The science pack of ring `r`, which colours its band: the red pack for Red, the pack whose first craft opens
    /// Green to Gold; -1 for Sky (it opens with the first rocket launch) or none.
    pub fn ring_pack(&self, r: u8) -> i32 {
        match Ring::from_u8(r) {
            Some(Ring::Red) => i32::from(RED_PACK.0),
            Some(ring) => ring.milestone_pack().map_or(-1, |p| i32::from(p.0)),
            None => -1,
        }
    }

    /// Techs of the ring before `r` still to finish before `r`'s pack recipe unlocks (half that ring, rounded up).
    pub fn ring_threshold_left(&self, r: u8) -> u32 {
        let Some(prev) = Ring::from_u8(r).and_then(Ring::prev) else { return 0 };
        let (done, total) = self.sim.factory.research.ring_progress(prev);
        total.div_ceil(2).saturating_sub(done)
    }

    /// The techs that make ring `r`'s pack parts and are not done yet (its pack recipe waits for them too).
    pub fn ring_parts_left(&self, r: u8) -> Vec<u32> {
        let Some(ring) = Ring::from_u8(r) else { return Vec::new() };
        let research = &self.sim.factory.research;
        pack_part_techs(ring).iter().filter(|&&t| research.state(t) != TechState::Done).map(|&t| u32::from(t)).collect()
    }

    /// What deed tech `t` asks of the player ("Lay 40 belts"), or empty for a lab tech.
    pub fn tech_deed_goal(&self, t: u32) -> String {
        TECHS.get(t as usize).and_then(|x| x.deed).map_or_else(String::new, Deed::goal)
    }

    /// Completed non-endless techs in ring `r`.
    pub fn ring_done(&self, r: u8) -> u32 {
        Ring::from_u8(r).map_or(0, |ring| self.sim.factory.research.ring_progress(ring).0)
    }

    /// Total non-endless techs in ring `r`.
    pub fn ring_total(&self, r: u8) -> u32 {
        Ring::from_u8(r).map_or(0, |ring| self.sim.factory.research.ring_progress(ring).1)
    }

    /// The items of what it unlocks (hand recipes' and machine recipes' products).
    pub fn tech_unlocks(&self, t: u32) -> Vec<u16> {
        let items = TECHS.get(t as usize).map(|x| x.unlocks.iter().map(|u| u.item()));
        items.map_or_else(Vec::new, |i| i.filter(|&i| i != ItemId::NONE).map(|i| i.0).collect())
    }

    /// Units done.
    pub fn tech_progress(&self, t: u32) -> u32 {
        self.sim.factory.research.progress(t.min(u8::MAX as u32) as u8)
    }

    pub fn tech_done(&self, t: u32) -> bool {
        self.tech_state_of(t) == TechState::Done
    }

    /// Every prerequisite is done and it isn't.
    pub fn tech_available(&self, t: u32) -> bool {
        self.tech_state_of(t) == TechState::Available
    }

    /// The tech labs work on, or -1 for none.
    pub fn current_research(&self) -> i32 {
        self.sim.factory.research.current.map_or(-1, i32::from)
    }

    /// Chooses what labs research now (-1 gives the current one up and moves on to the next queued), at the next
    /// tick. Only an available tech is taken; the tech it replaces goes to the front of the queue.
    pub fn set_research(&mut self, t: i32) {
        let tech = u8::try_from(t).unwrap_or(u8::MAX);
        self.act(Action::SetResearch { tech });
    }

    /// How many techs wait behind the current one.
    pub fn research_queue_len(&self) -> u32 {
        self.sim.factory.research.queue().len() as u32
    }

    /// The tech at place `i` of the queue (0 is next), or -1.
    pub fn research_queue_at(&self, i: u32) -> i32 {
        self.sim.factory.research.queue().get(i as usize).map_or(-1, |&t| i32::from(t))
    }

    /// The most techs the queue holds.
    pub fn research_queue_max(&self) -> u32 {
        MAX_QUEUE as u32
    }

    /// Adds a tech, with the prerequisites it is missing, to the end of the queue, at the next tick.
    pub fn queue_research(&mut self, t: u32) {
        self.act(Action::QueueResearch { tech: t.min(u8::MAX as u32) as u8 });
    }

    /// Takes a tech, and the queued techs that need it, out of the queue, at the next tick.
    pub fn unqueue_research(&mut self, t: u32) {
        self.act(Action::UnqueueResearch { tech: t.min(u8::MAX as u32) as u8 });
    }
}

impl Game {
    fn tech_state_of(&self, t: u32) -> TechState {
        self.sim.factory.research.state(t.min(u8::MAX as u32) as u8)
    }
}
