//! Where items go. `Slot` names a machine, `Link` is where a belt or miner delivers, `Sinks` borrows
//! the machines that take items and `deliver` hands one over. `relink` rebuilds the derived data after
//! any machine is added or removed: belt outputs and corner curves, the outputs of every other machine,
//! and the downstream-first belt update order. Nothing here is saved; links are a pure function of the
//! machines and their positions. A new machine that takes items: an arm in `Slot::is_sink` and in
//! `Sinks`.

use crate::item::ItemId;
use crate::math::IVec3;

use super::belt::Belt;
use super::constructor::Constructor;
use super::smelter::Smelter;
use super::storage::Storage;
use super::{opposite, Factory, Kind, DIRS, FACES};

/// Where a belt, miner or box delivers to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Link {
    None,
    /// `mid`: joining from the side, so the item enters halfway along the target belt.
    Belt {
        belt: u32,
        mid: bool,
    },
    /// A machine that takes items (`Slot::is_sink`).
    Machine(Slot),
}

/// What occupies a position: an index into the matching machine `Vec`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Slot {
    Belt(u32),
    Miner(u32),
    Storage(u32),
    Smelter(u32),
    Constructor(u32),
}

impl Slot {
    /// Whether belts and miners can deliver into it (belts are linked separately).
    pub(super) fn is_sink(self) -> bool {
        match self {
            Slot::Storage(_) | Slot::Smelter(_) | Slot::Constructor(_) => true,
            Slot::Belt(_) | Slot::Miner(_) => false,
        }
    }

    pub(super) fn kind(self) -> Kind {
        match self {
            Slot::Belt(_) => Kind::Belt,
            Slot::Miner(_) => Kind::Miner,
            Slot::Storage(_) => Kind::Storage,
            Slot::Smelter(_) => Kind::Smelter,
            Slot::Constructor(_) => Kind::Constructor,
        }
    }
}

/// The machines items can be delivered into, borrowed apart from the belts and miners.
pub(crate) struct Sinks<'a> {
    pub(super) storages: &'a mut [Storage],
    pub(super) smelters: &'a mut [Smelter],
    pub(super) constructors: &'a mut [Constructor],
}

impl Sinks<'_> {
    /// Whether the sink at `slot` takes one `item` now.
    pub(super) fn can_accept(&self, slot: Slot, item: ItemId) -> bool {
        match slot {
            Slot::Storage(i) => self.storages[i as usize].buf.can_accept(item),
            Slot::Smelter(i) => self.smelters[i as usize].can_accept(item),
            Slot::Constructor(i) => self.constructors[i as usize].room_for(item) > 0,
            Slot::Belt(_) | Slot::Miner(_) => false,
        }
    }

    /// Hands one `item` to the sink at `slot`; false if it doesn't take it.
    pub(super) fn accept(&mut self, slot: Slot, item: ItemId) -> bool {
        match slot {
            Slot::Storage(i) => self.storages[i as usize].buf.add(item, 1) == 0,
            Slot::Smelter(i) => self.smelters[i as usize].accept(item),
            Slot::Constructor(i) => self.constructors[i as usize].accept(item),
            Slot::Belt(_) | Slot::Miner(_) => false,
        }
    }
}

/// Hands one item to a link. `overflow` is how far past the end of the source belt it already is.
pub(super) fn deliver(belts: &mut [Belt], sinks: &mut Sinks, link: Link, item: ItemId, overflow: f32) -> bool {
    match link {
        Link::None => false,
        Link::Belt { belt, mid } => belts[belt as usize].accept(item, mid, overflow),
        Link::Machine(slot) => sinks.accept(slot, item),
    }
}

impl Factory {
    /// Recomputes belt links, curves, machine outputs and the belt update order.
    pub(super) fn relink(&mut self) {
        self.dirty = false;
        let at = &self.at;
        let belts = &self.belts;
        let belt_at = |q: IVec3| match at.get(&q) {
            Some(Slot::Belt(j)) => Some(*j),
            _ => None,
        };

        let curves: Vec<Option<u8>> = belts
            .iter()
            .map(|b| {
                let back = b.pos - DIRS[b.dir as usize];
                let (mut back_fed, mut sides, mut side) = (false, 0, None);
                for s in 0..4u8 {
                    if s == b.dir {
                        continue;
                    }
                    let q = b.pos + DIRS[s as usize];
                    let feeds = match at.get(&q) {
                        Some(Slot::Belt(j)) => {
                            let o = &belts[*j as usize];
                            o.pos + DIRS[o.dir as usize] == b.pos
                        }
                        // Machines only feed belts whose back faces them.
                        Some(_) => q == back,
                        None => false,
                    };
                    if feeds && q == back {
                        back_fed = true;
                    } else if feeds {
                        sides += 1;
                        side = Some(s);
                    }
                }
                if !back_fed && sides == 1 {
                    side
                } else {
                    None
                }
            })
            .collect();

        let outs: Vec<Link> = belts
            .iter()
            .map(|b| {
                let t = b.pos + DIRS[b.dir as usize];
                match at.get(&t) {
                    Some(Slot::Belt(j)) => {
                        let o = &belts[*j as usize];
                        if o.dir == opposite(b.dir) {
                            Link::None
                        } else {
                            let from_back = o.pos - DIRS[o.dir as usize] == b.pos;
                            let start = from_back || curves[*j as usize] == Some(opposite(b.dir));
                            Link::Belt { belt: *j, mid: !start }
                        }
                    }
                    Some(&s) if s.is_sink() => Link::Machine(s),
                    _ => Link::None,
                }
            })
            .collect();

        // Machines feed belts that lead away from them.
        let feeds = |pos: IVec3| -> Vec<u32> {
            (0..4u8).filter_map(|s| belt_at(pos + DIRS[s as usize]).filter(|&j| belts[j as usize].dir == s)).collect()
        };
        let miner_outs: Vec<Vec<Link>> = self
            .miners
            .iter()
            .map(|m| {
                let mut v: Vec<Link> = feeds(m.pos).into_iter().map(|belt| Link::Belt { belt, mid: false }).collect();
                for (f, &n) in FACES.iter().enumerate() {
                    if f as u8 == m.drill {
                        continue;
                    }
                    if let Some(&s) = at.get(&(m.pos + n)).filter(|s| s.is_sink()) {
                        v.push(Link::Machine(s));
                    }
                }
                v
            })
            .collect();
        let storage_outs: Vec<Vec<u32>> = self.storages.iter().map(|s| feeds(s.pos)).collect();
        let smelter_outs: Vec<Vec<u32>> = self.smelters.iter().map(|s| feeds(s.pos)).collect();
        let constructor_outs: Vec<Vec<u32>> = self.constructors.iter().map(|c| feeds(c.pos)).collect();

        for ((b, c), o) in self.belts.iter_mut().zip(curves).zip(outs) {
            b.curve_from = c;
            b.out = o;
        }
        for (m, o) in self.miners.iter_mut().zip(miner_outs) {
            m.outs = o;
            m.next_out %= m.outs.len().max(1);
        }
        for (s, o) in self.storages.iter_mut().zip(storage_outs) {
            s.outs = o;
            s.next_out %= s.outs.len().max(1);
        }
        for (s, o) in self.smelters.iter_mut().zip(smelter_outs) {
            s.outs = o;
            s.next_out %= s.outs.len().max(1);
        }
        for (c, o) in self.constructors.iter_mut().zip(constructor_outs) {
            c.outs = o;
            c.next_out %= c.outs.len().max(1);
        }

        // Each belt has at most one belt downstream, so walking the chain from every unvisited belt
        // and appending it reversed puts every belt after the one it feeds.
        let n = self.belts.len();
        let mut visited = vec![false; n];
        let mut path = Vec::new();
        self.order.clear();
        for start in 0..n {
            let mut cur = start;
            path.clear();
            while !visited[cur] {
                visited[cur] = true;
                path.push(cur as u32);
                match self.belts[cur].out {
                    Link::Belt { belt, .. } => cur = belt as usize,
                    _ => break,
                }
            }
            self.order.extend(path.iter().rev());
        }
    }
}
