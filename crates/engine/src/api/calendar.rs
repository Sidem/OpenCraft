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

    /// Packed 2-bit seasonal role per texture layer (128 words = 32 `uvec4`s, same layout as `texture_turns`):
    /// 0 none, 1 grass/turf, 2 woodland ferns/reeds, 3 seasonal wildflowers (`mesher/foliage/season.rs`).
    pub fn season_layers(&self) -> Vec<u32> {
        crate::mesher::season_layers()
    }

    /// This frame's seasonal foliage uniforms (`mesher::FOLIAGE_FLOATS` floats = 30 `vec4`s: `leafA[10]`, `leafB[10]`,
    /// `litter[10]`) evaluated from the current `year_fraction`.
    pub fn season_foliage(&self) -> Vec<f32> {
        crate::mesher::pack_foliage(daytime::year_fraction(self.sim.tick) as f32).to_vec()
    }

    /// This frame's seasonal ground uniforms (`mesher::GROUND_FLOATS` floats = 3 `vec4`s: grass tint, woodland tint,
    /// budding + flower cover) evaluated from the current `year_fraction`.
    pub fn season_ground(&self) -> Vec<f32> {
        crate::mesher::pack_ground(daytime::year_fraction(self.sim.tick) as f32).to_vec()
    }
}
