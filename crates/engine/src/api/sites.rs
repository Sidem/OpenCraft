//! Terraforming sites (`factory/sites.rs`): marking and removing them (queued actions), the list for
//! overlays and the planner's survey (loaded chunks only).

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::factory::{survey_site, survey_tunnel, Job, SiteSurvey, Tunnel, SECTIONS};
use crate::math::IVec3;
use crate::Game;

/// Numbers per site in `sites`: id, lo x, lo z, hi x, hi z, level, job (0 dig, 1 fill, 2 flatten, 3 tunnel), cells,
/// section (a tunnel's index into `tunnel_sections`, else -1).
const SITE_FIELDS: usize = 9;

#[wasm_bindgen]
impl Game {
    /// Marks a site over the columns between (ax, az) and (bx, bz): `job` (0 dig, 1 fill, 2 flatten)
    /// to `level`. The core refuses one that doesn't fit (too big, overlapping, too many).
    pub fn mark_site(&mut self, ax: i32, az: i32, bx: i32, bz: i32, level: i32, job: u8) {
        if let Some(job) = Job::from_byte(job) {
            self.act(Action::MarkSite { a: (ax, az), b: (bx, bz), level, job });
        }
    }

    /// Marks a tunnel from the block `ends[0..3]` towards `ends[3..6]` (x, y, z each) with section `size` (an
    /// index into `tunnel_sections`). The core refuses one that is too long, too steep or in the way of another site.
    pub fn mark_tunnel(&mut self, ends: &[i32], size: u8) {
        if let Some((from, to)) = tunnel_ends(ends) {
            self.act(Action::MarkTunnel { from, to, size });
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
            .flat_map(|s| {
                let section = s.tunnel.map_or(-1, |t| t.size as i32);
                [s.id as i32, s.lo.0, s.lo.1, s.hi.0, s.hi.1, s.level, s.job as i32, s.cells() as i32, section]
            })
            .collect()
    }

    /// What marking would move, among loaded chunks: cut, fill, ore, trees, water, unseen columns and
    /// the spoil to carry away (negative: ground to bring in). Empty for an area too big to be a site.
    pub fn site_survey(&self, ax: i32, az: i32, bx: i32, bz: i32, level: i32, job: u8) -> Vec<i32> {
        let Some(job) = Job::from_byte(job) else { return Vec::new() };
        let Some(s) = survey_site(&self.sim.world, (ax, az), (bx, bz), level, job) else { return Vec::new() };
        survey_numbers(&s)
    }

    /// The same for a tunnel (`water` counts the cells next to water, which drones leave alone; `unseen` is cells
    /// not loaded). Empty for one the core would refuse.
    pub fn tunnel_survey(&self, ends: &[i32], size: u8) -> Vec<i32> {
        match tunnel_ends(ends).and_then(|(from, to)| Tunnel::new(from, to, size)) {
            Some(t) => survey_numbers(&survey_tunnel(&self.sim.world, &t)),
            None => Vec::new(),
        }
    }

    /// The tunnel sections as width, height pairs.
    pub fn tunnel_sections(&self) -> Vec<i32> {
        SECTIONS.iter().flat_map(|&(w, h)| [w, h]).collect()
    }
}

fn tunnel_ends(e: &[i32]) -> Option<(IVec3, IVec3)> {
    match *e {
        [ax, ay, az, bx, by, bz] => Some((IVec3::new(ax, ay, az), IVec3::new(bx, by, bz))),
        _ => None,
    }
}

fn survey_numbers(s: &SiteSurvey) -> Vec<i32> {
    [s.cut, s.fill, s.ore, s.trees, s.water, s.unseen].iter().map(|&n| n as i32).chain([s.spoil()]).collect()
}
