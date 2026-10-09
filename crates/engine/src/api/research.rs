//! The research screen: the tech tree (`research::TECHS`), the world's progress through it, and
//! choosing what every lab researches.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::item::ItemId;
use crate::research::{is_bonus, TechState, MAX_QUEUE, TECHS};
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
