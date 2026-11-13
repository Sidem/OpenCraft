//! What the personal drone (`helpers/drone/`) can reach: the items in storage boxes and in what machines have made
//! (their output and byproduct buffers, a miner's ore) near a point, the boxes that could take an item, and taking
//! items out of any of them. Core reads; `take_stock` is the only change, made by the drone's tick.
//!
//! Which boxes it may fill (`homes_near`), so it never clogs a line: a box on a production line (a belt, miner or
//! quarry delivers into it, or it feeds a belt) never; a box whose label the player picked, or that a drone port's pad
//! touches, only with more of what it already holds; any other box also when empty. A box holding other things never,
//! except one labelled `box_label::MISC`: the catch-all, filled last with anything that fits, kinds mixed.
//! It fetches from storage first, then from line boxes, then from machines.
//!
//! Invariants: lists come back in that order, nearest first within it, ties in the factory's own list order (boxes,
//! then miners, then machines), so every peer plans the same trip. A machine's input and fuel are never taken: only
//! what it made. To let the drone reach another kind of holder: a branch in `holders` and in `take_stock`.

use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::{sort_small_by_key, IVec3};

use super::links::Link;
use super::{box_label, Factory, Slot};

/// A place holding some of an item: where, how many, its squared distance and whether it is a machine (not a box).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Source {
    pub pos: IVec3,
    pub count: u32,
    pub d2: i64,
    pub machine: bool,
}

/// How much of every item lies within reach: in boxes and in machines.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stocked {
    pub item: ItemId,
    pub boxes: u32,
    pub machines: u32,
}

/// How the drone may use a box (`roles`): never, only as the home of what it holds, also when empty, or for anything.
const LINE: u8 = 0;
const HOME_ONLY: u8 = 1;
const FREE: u8 = 2;
const MISC: u8 = 3;

/// A box the drone may fill with an item (`homes_near`): where, how many fit, and whether it is a catch-all (which may
/// take several kinds on one trip).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Home {
    pub at: IVec3,
    pub room: u32,
    pub misc: bool,
}

impl Factory {
    /// Every box and machine within `range` blocks of `from` holding `item`: storage, line boxes, then machines,
    /// nearest first within each.
    pub fn stock_near(&self, from: IVec3, item: ItemId, range: i32) -> Vec<Source> {
        let mut out = Vec::new();
        self.holders(from, range, |pos, d2, class, stacks| {
            let count: u32 = stacks.iter().filter(|s| s.item == item).map(|s| s.count).sum();
            if count > 0 {
                out.push((Source { pos, count, d2, machine: class == 2 }, class));
            }
        });
        sort_small_by_key(&mut out, |s| (s.1, s.0.d2));
        out.into_iter().map(|s| s.0).collect()
    }

    /// How many of `item` lie within `range` blocks of `from`, in boxes and machines together.
    pub fn stock_count(&self, from: IVec3, item: ItemId, range: i32) -> u32 {
        self.stock_near(from, item, range).iter().map(|s| s.count).sum()
    }

    /// Every item within `range` blocks of `from`, in the order first met, with how many boxes and machines hold.
    pub fn stock_totals(&self, from: IVec3, range: i32) -> Vec<Stocked> {
        let mut out: Vec<Stocked> = Vec::new();
        self.holders(from, range, |_, _, class, stacks| {
            for s in stacks.iter().filter(|s| !s.is_empty()) {
                let i = match out.iter().position(|o| o.item == s.item) {
                    Some(i) => i,
                    None => {
                        out.push(Stocked { item: s.item, boxes: 0, machines: 0 });
                        out.len() - 1
                    }
                };
                if class == 2 {
                    out[i].machines += s.count;
                } else {
                    out[i].boxes += s.count;
                }
            }
        });
        out
    }

    /// The boxes within `range` of `from` the drone may fill with `item` (see the header for which), in this order,
    /// nearest first within each: those already holding some (its home), empty free ones, catch-alls holding some,
    /// then catch-alls with room.
    pub fn homes_near(&self, from: IVec3, item: ItemId, range: i32) -> Vec<Home> {
        let (reach, roles) = (i64::from(range).pow(2), self.box_roles());
        let mut out: Vec<(Home, u8, i64)> = Vec::new();
        for (s, &role) in self.storages.iter().zip(&roles) {
            let (d2, room, home) = (dist2(s.pos, from), s.buf.space_for(item), s.buf.count(item) > 0);
            let tier = match role {
                MISC => 2 + u8::from(!home),
                LINE => continue,
                _ if home => 0,
                FREE if s.buf.total() == 0 => 1,
                _ => continue,
            };
            if d2 <= reach && room > 0 {
                out.push((Home { at: s.pos, room, misc: role == MISC }, tier, d2));
            }
        }
        sort_small_by_key(&mut out, |h| (h.1, h.2));
        out.into_iter().map(|h| h.0).collect()
    }

    /// Room left for `item` in the box at `at` once `planned` (stacks already on their way there) are in.
    pub fn room_after(&self, at: IVec3, planned: impl Iterator<Item = Stack>, item: ItemId) -> u32 {
        let Some(&Slot::Storage(i)) = self.at.get(&at) else { return 0 };
        let mut buf = super::buffer::Buffer { slots: self.storages[i as usize].buf.slots.clone() };
        planned.for_each(|s| _ = buf.add(s.item, s.count));
        buf.space_for(item)
    }

    /// Whether some box within `range` of `from`, off the production lines, already holds `item` and has room.
    pub fn has_home_near(&self, from: IVec3, item: ItemId, range: i32) -> bool {
        let (reach, roles) = (i64::from(range).pow(2), self.box_roles());
        self.storages.iter().zip(&roles).any(|(s, &role)| {
            role != LINE && dist2(s.pos, from) <= reach && s.buf.count(item) > 0 && s.buf.space_for(item) > 0
        })
    }

    /// Takes up to `n` of `item` from the box or machine at `pos` (a machine's output first, then its byproducts);
    /// how many came out.
    pub fn take_stock(&mut self, pos: IVec3, item: ItemId, n: u32) -> u32 {
        let take = |buf: &mut super::buffer::Buffer, n: u32| {
            let took = buf.count(item).min(n);
            buf.remove(item, took);
            took
        };
        match self.at.get(&pos).copied() {
            Some(Slot::Storage(i)) => take(&mut self.storages[i as usize].buf, n),
            Some(Slot::Miner(i)) => take(&mut self.miners[i as usize].out, n),
            Some(Slot::Process(i)) => {
                let p = &mut self.processors[i as usize];
                let first = take(&mut p.out, n);
                first + take(&mut p.side, n - first)
            }
            _ => 0,
        }
    }

    /// Each box's role for the drone, in list order (see the header). Links are derived (`relink`), so a line built
    /// this tick counts from the next factory update.
    fn box_roles(&self) -> Vec<u8> {
        let mut roles: Vec<u8> = self
            .storages
            .iter()
            .map(|s| {
                if !s.outs.is_empty() {
                    LINE
                } else if s.label == box_label::MISC {
                    MISC
                } else if s.label != box_label::AUTO {
                    HOME_ONLY
                } else {
                    FREE
                }
            })
            .collect();
        let fed = |l: &Link| match *l {
            Link::Machine(Slot::Storage(i)) => Some(i as usize),
            _ => None,
        };
        let belts = self.belts.iter().filter_map(|b| fed(&b.out));
        let miners = self.miners.iter().flat_map(|m| &m.outs).filter_map(fed);
        for i in belts.chain(miners).chain(self.quarries.iter().flat_map(|q| &q.outs).filter_map(fed)) {
            roles[i] = LINE;
        }
        for port in self.ports() {
            for pos in self.port_boxes(port.anchor) {
                if let Some(&Slot::Storage(i)) = self.at.get(&pos) {
                    roles[i as usize] = roles[i as usize].min(HOME_ONLY);
                }
            }
        }
        roles
    }

    /// Calls `each(pos, squared distance, class, stacks)` for every box and machine within `range` of `from`, in list
    /// order; the class is 0 for a storage box, 1 for a box on a line, 2 for a machine.
    fn holders(&self, from: IVec3, range: i32, mut each: impl FnMut(IVec3, i64, u8, &[Stack])) {
        let (reach, roles) = (i64::from(range).pow(2), self.box_roles());
        for (s, &role) in self.storages.iter().zip(&roles) {
            let d2 = dist2(s.pos, from);
            if d2 <= reach {
                each(s.pos, d2, u8::from(role == LINE), &s.buf.slots);
            }
        }
        for m in &self.miners {
            let d2 = dist2(m.pos, from);
            if d2 <= reach {
                each(m.pos, d2, 2, &m.out.slots);
            }
        }
        for p in &self.processors {
            let d2 = dist2(p.pos, from);
            if d2 <= reach {
                each(p.pos, d2, 2, &p.out.slots);
                each(p.pos, d2, 2, &p.side.slots);
            }
        }
    }
}

fn dist2(a: IVec3, b: IVec3) -> i64 {
    let d = a - b;
    i64::from(d.x).pow(2) + i64::from(d.y).pow(2) + i64::from(d.z).pow(2)
}
