//! The calendar the host shows (`web/src/ui/calendar.ts`, the HUD's season chip): the day, the season, the year's
//! daylight and growth, all read from `daytime.rs`; queries only. The clock and the sky's sun are in `render.rs`.

use wasm_bindgen::prelude::*;

use crate::daytime::{self, Season};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The day the world is on, counting from 1.
    pub fn day_number(&self) -> u32 {
        daytime::day_number(self.sim.tick) as u32
    }

    /// The season: 0 spring, 1 summer, 2 autumn, 3 winter.
    pub fn season(&self) -> u32 {
        daytime::season(self.sim.tick) as u32
    }

    /// Which day of its season it is, counting from 1 (`days_per_season` days each).
    pub fn day_of_season(&self) -> u32 {
        daytime::day_of_season(self.sim.tick) as u32
    }

    /// Days in each season.
    pub fn days_per_season(&self) -> u32 {
        daytime::DAYS_PER_SEASON as u32
    }

    /// How far through the year it is, 0 to 1 from the start of spring.
    pub fn year_fraction(&self) -> f64 {
        daytime::year_fraction(self.sim.tick)
    }

    /// The share of day `day` of the year (from 0) the sun is up, 0 to 1.
    pub fn daylight_share(&self, day: u32) -> f64 {
        daytime::daylight_share(u64::from(day))
    }

    /// How fast plants grow in `season` (0 spring to 3 winter), as a percentage of the year-round rate.
    pub fn growth_percent(&self, season: u32) -> u32 {
        let s = [Season::Spring, Season::Summer, Season::Autumn, Season::Winter];
        daytime::growth_percent(s[season as usize % 4])
    }
}
