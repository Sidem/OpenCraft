//! Earthworks (Milestone 8): drones work terraforming sites (`factory/sites.rs`) the way they work ghosts. A
//! site is a list of cells in a fixed order (`Site::cell`): cut layers from the top down, then fill layers from
//! the bottom up. A port picks the first cell in its reach that needs work and that no drone is on:
//!
//! - a **cut** cell (above the site's level, or any cell of a tunnel) holds a block the cut takes (`cut_takes`): the
//!   drone breaks it by hand and the drops go into the boxes touching the pad, which must have room (no spilling
//!   mountains). A tunnel cell next to water waits (`Need::Later`), so the bore never lets water in;
//! - a **fill** cell (at or below the level) is free (air or water), has a free cell above it up to the level and
//!   solid ground or an earlier fill below it: the drone brings one block from a box (dirt on top, else stone,
//!   dirt, sand, grass) and places it. So dug ground fills other sites from the same boxes.
//!
//! `Site::done` caches how many cells from the start need nothing; it is a speed-up only (never saved, no
//! effect on what is picked). When every cell is done (rechecked from the start once) the site is removed.
//! To change the fill materials: [`FILL_TOP`] and [`FILL_BELOW`].

use crate::block::{self, AIR, DIRT, GRASS, SAND, STONE};
use crate::factory::{cut_takes, touches_water, Site};
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::Sim;

use super::{target_centre, Drone, Job, PortInfo};

/// Fill blocks by preference, for the top layer and for the layers below it.
const FILL_TOP: [block::BlockId; 4] = [DIRT, STONE, SAND, GRASS];
const FILL_BELOW: [block::BlockId; 4] = [STONE, DIRT, SAND, GRASS];

#[derive(PartialEq, Eq)]
enum Need {
    /// Nothing to do in this cell.
    Nothing,
    /// A fill that has to wait for the cell below it.
    Later,
    Cut,
    Fill,
}

impl Sim {
    /// Whether the drones' work at `target` is breaking (a tear-down mark, or a site's cut cell).
    pub(super) fn breaks(&self, target: IVec3) -> bool {
        match self.ghosts.covering(target) {
            Some(g) => g.block == AIR,
            None => self.factory.sites.list.iter().any(|s| s.covers(target) && s.cuts_at(target)),
        }
    }

    /// The end of a drone's work on a site cell: break it, or place the block it brought.
    pub(super) fn finish_site_cell(&mut self, d: &mut Drone) {
        let Some(s) = self.factory.sites.list.iter().find(|s| s.covers(d.target)).copied() else { return };
        if s.cuts_at(d.target) {
            self.break_into_boxes(d.port, d.target);
        } else if d.load.count > 0 && self.put_block(super::CREDIT, d.target, d.load.item, 0, d.target) {
            d.load = Default::default();
        }
    }

    /// The first cell of a site in `port`'s reach that can be worked now, if it is nearer than `closer_than`
    /// (squared blocks from the pad). Removes sites that are finished.
    pub(super) fn site_job(&mut self, port: &PortInfo, boxes: &[IVec3], closer_than: Option<f64>) -> Option<Job> {
        let mut best: Option<(f64, Job)> = None;
        let reach2 = port.reach * port.reach;
        let mut finished = Vec::new();
        for i in 0..self.factory.sites.list.len() {
            let s = self.factory.sites.list[i];
            let Some(start) = self.first_pending(&s) else {
                finished.push(s.id);
                continue;
            };
            self.factory.sites.list[i].done = start;
            for k in start..s.cells() {
                let pos = s.cell(k);
                let to = target_centre(pos) - port.centre;
                let dist2 = to.x * to.x + to.y * to.y + to.z * to.z;
                let beaten = [closer_than, best.as_ref().map(|b| b.0)].into_iter().flatten().any(|b| b <= dist2);
                if dist2 > reach2 || beaten || self.drones.list.iter().any(|d| d.target == pos) {
                    continue;
                }
                if let Some(job) = self.workable(&s, pos, boxes) {
                    best = Some((dist2, job));
                    break;
                }
            }
        }
        for id in finished {
            self.factory.sites.remove(id);
        }
        best.map(|b| b.1)
    }

    /// The first cell of `s` that is not done (from its cache; once from the start if that finds none), or
    /// `None` when the whole site is done.
    fn first_pending(&mut self, s: &Site) -> Option<u32> {
        let total = s.cells();
        for from in [s.done.min(total), 0] {
            let mut k = from;
            while k < total && self.need(s, s.cell(k)) == Need::Nothing {
                k += 1;
            }
            if k < total {
                return Some(k);
            }
            if from == 0 {
                break;
            }
        }
        None
    }

    /// What is to be done in the site's cell at `pos`.
    fn need(&mut self, s: &Site, pos: IVec3) -> Need {
        let id = self.world.block_anywhere_or_generate(pos);
        if s.tunnel.is_some() {
            // Breaking into water waits until the water is gone.
            let world = &mut self.world;
            return match (cut_takes(id), touches_water(|p| Some(world.block_anywhere_or_generate(p)), pos)) {
                (false, _) => Need::Nothing,
                (true, true) => Need::Later,
                (true, false) => Need::Cut,
            };
        }
        if pos.y > s.level {
            return if cut_takes(id) { Need::Cut } else { Need::Nothing };
        }
        if !block::replaceable(id) {
            return Need::Nothing;
        }
        for y in pos.y + 1..=s.level {
            if !block::replaceable(self.world.block_anywhere_or_generate(IVec3::new(pos.x, y, pos.z))) {
                return Need::Nothing;
            }
        }
        if block::replaceable(self.world.block_anywhere_or_generate(pos - IVec3::new(0, 1, 0))) {
            Need::Later
        } else {
            Need::Fill
        }
    }

    /// The job for the cell at `pos`, if it needs work now and the port can supply it.
    fn workable(&mut self, s: &Site, pos: IVec3, boxes: &[IVec3]) -> Option<Job> {
        match self.need(s, pos) {
            Need::Cut => {
                let drop = block::def(self.world.block_anywhere_or_generate(pos)).drop;
                let room = drop == AIR || self.factory.boxes_have_room(boxes, ItemId::block(drop));
                room.then_some(Job { pos, item: ItemId::NONE, from: None })
            }
            Need::Fill => {
                let order = if pos.y == s.level { FILL_TOP } else { FILL_BELOW };
                order.iter().find_map(|&b| {
                    let item = ItemId::block(b);
                    let from = boxes.iter().copied().find(|&bx| self.factory.box_count(bx, item) > 0)?;
                    Some(Job { pos, item, from: Some(from) })
                })
            }
            Need::Nothing | Need::Later => None,
        }
    }
}
