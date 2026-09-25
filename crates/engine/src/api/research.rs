//! The research screen: the tech tree (`research::TECHS`), the world's progress through it, and
//! choosing what every lab researches.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::research::{TechState, TECHS};
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

    /// Lab seconds per unit at full power.
    pub fn tech_seconds(&self, t: u32) -> f64 {
        TECHS.get(t as usize).map_or(0.0, |x| x.seconds)
    }

    /// Items whose recipes it unlocks.
    pub fn tech_unlocks(&self, t: u32) -> Vec<u16> {
        TECHS.get(t as usize).map_or_else(Vec::new, |x| x.unlocks.iter().map(|i| i.0).collect())
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

    /// Chooses what labs research (-1 to stop), at the next tick. Only an available tech is taken.
    pub fn set_research(&mut self, t: i32) {
        let tech = u8::try_from(t).unwrap_or(u8::MAX);
        self.act(Action::SetResearch { tech });
    }
}

impl Game {
    fn tech_state_of(&self, t: u32) -> TechState {
        self.sim.factory.research.state(t.min(u8::MAX as u32) as u8)
    }
}
