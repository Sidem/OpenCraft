//! Terraforming sites (`factory/sites.rs`): marking and removing them (queued actions), the list for
//! overlays and the planner's survey (loaded chunks only).

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::factory::{survey_site, Job};
use crate::Game;

/// Numbers per site in `sites`: id, lo x, lo z, hi x, hi z, level, job (0 dig, 1 fill, 2 flatten), cells.
const SITE_FIELDS: usize = 8;

#[wasm_bindgen]
impl Game {
    /// Marks a site over the columns between (ax, az) and (bx, bz): `job` (0 dig, 1 fill, 2 flatten)
    /// to `level`. The core refuses one that doesn't fit (too big, overlapping, too many).
    pub fn mark_site(&mut self, ax: i32, az: i32, bx: i32, bz: i32, level: i32, job: u8) {
        if let Some(job) = Job::from_byte(job) {
            self.act(Action::MarkSite { a: (ax, az), b: (bx, bz), level, job });
        }
    }

    pub fn remove_site(&mut self, id: u32) {
        self.act(Action::RemoveSite { id });
    }

    pub fn site_fields(&self) -> usize {
        SITE_FIELDS
    }

    /// Every site, `site_fields` numbers each.
    pub fn sites(&self) -> Vec<i32> {
        let list = &self.sim.factory.sites.list;
        list.iter()
            .flat_map(|s| [s.id as i32, s.lo.0, s.lo.1, s.hi.0, s.hi.1, s.level, s.job as i32, s.cells() as i32])
            .collect()
    }

    /// What marking would move, among loaded chunks: cut, fill, ore, trees, water, unseen columns and
    /// the spoil to carry away (negative: ground to bring in). Empty for an area too big to be a site.
    pub fn site_survey(&self, ax: i32, az: i32, bx: i32, bz: i32, level: i32, job: u8) -> Vec<i32> {
        let Some(job) = Job::from_byte(job) else { return Vec::new() };
        let Some(s) = survey_site(&self.sim.world, (ax, az), (bx, bz), level, job) else { return Vec::new() };
        [s.cut, s.fill, s.ore, s.trees, s.water, s.unseen].iter().map(|&n| n as i32).chain([s.spoil()]).collect()
    }
}
