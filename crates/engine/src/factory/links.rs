//! Where items go. `Slot` names a machine, `Link` is where a belt or miner delivers, `Sinks` borrows
//! the machines that take items and `deliver` hands one over. `relink` rebuilds the derived data after
//! any machine is added or removed: belt outputs (for every belt shape) and corner curves, the outputs of every other machine,
//! and the downstream-first belt update order, and the power grids (`power.rs`). Nothing here is saved; links are a pure function of the
//! machines and their positions. A new machine that takes items: an arm in `Slot::is_sink` and in
//! `Sinks`.

use crate::item::ItemId;
use crate::math::IVec3;

use super::belt::Belt;
use super::belt_shape::{Shape, UNDERPASS_RANGE};
use super::constructor::Constructor;
use super::generator::Generator;
use super::lab::Lab;
use super::power::Power;
use super::router::Router;
use super::smelter::Smelter;
use super::storage::Storage;
use super::{opposite, Factory, Kind, DIRS, FACES};

const UP: IVec3 = IVec3::new(0, 1, 0);

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
    Router(u32),
    Generator(u32),
    Pole(u32),
    Lab(u32),
}

impl Slot {
    /// Whether belts and miners can deliver into it (belts are linked separately).
    pub(super) fn is_sink(self) -> bool {
        match self {
            Slot::Storage(_)
            | Slot::Smelter(_)
            | Slot::Constructor(_)
            | Slot::Router(_)
            | Slot::Generator(_)
            | Slot::Lab(_) => true,
            Slot::Belt(_) | Slot::Miner(_) | Slot::Pole(_) => false,
        }
    }

    pub(super) fn kind(self) -> Kind {
        match self {
            Slot::Belt(_) => Kind::Belt,
            Slot::Miner(_) => Kind::Miner,
            Slot::Storage(_) => Kind::Storage,
            Slot::Smelter(_) => Kind::Smelter,
            Slot::Constructor(_) => Kind::Constructor,
            Slot::Router(_) => Kind::Router,
            Slot::Generator(_) => Kind::Generator,
            Slot::Pole(_) => Kind::Pole,
            Slot::Lab(_) => Kind::Lab,
        }
    }
}

/// The machines items can be delivered into, borrowed apart from the belts and miners.
pub(crate) struct Sinks<'a> {
    pub(super) storages: &'a mut [Storage],
    pub(super) smelters: &'a mut [Smelter],
    pub(super) constructors: &'a mut [Constructor],
    pub(super) routers: &'a mut [Router],
    pub(super) generators: &'a mut [Generator],
    pub(super) labs: &'a mut [Lab],
}

impl Sinks<'_> {
    /// Whether the sink at `slot` takes one `item` now.
    pub(super) fn can_accept(&self, slot: Slot, item: ItemId) -> bool {
        match slot {
            Slot::Storage(i) => self.storages[i as usize].buf.can_accept(item),
            Slot::Smelter(i) => self.smelters[i as usize].can_accept(item),
            Slot::Constructor(i) => self.constructors[i as usize].room_for(item) > 0,
            Slot::Router(i) => self.routers[i as usize].can_accept(),
            Slot::Generator(i) => self.generators[i as usize].room_for(item) > 0,
            Slot::Lab(i) => self.labs[i as usize].room_for(item) > 0,
            Slot::Belt(_) | Slot::Miner(_) | Slot::Pole(_) => false,
        }
    }

    /// Hands one `item` to the sink at `slot`; false if it doesn't take it.
    pub(super) fn accept(&mut self, slot: Slot, item: ItemId) -> bool {
        match slot {
            Slot::Storage(i) => self.storages[i as usize].buf.add(item, 1) == 0,
            Slot::Smelter(i) => self.smelters[i as usize].accept(item),
            Slot::Constructor(i) => self.constructors[i as usize].accept(item),
            Slot::Router(i) => self.routers[i as usize].accept(item),
            Slot::Generator(i) => self.generators[i as usize].accept(item),
            Slot::Lab(i) => self.labs[i as usize].accept(item),
            Slot::Belt(_) | Slot::Miner(_) | Slot::Pole(_) => false,
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
                if b.shape != Shape::Flat {
                    return None;
                }
                for s in 0..4u8 {
                    if s == b.dir {
                        continue;
                    }
                    let q = b.pos + DIRS[s as usize];
                    let feeds = match at.get(&q) {
                        Some(Slot::Belt(j)) => {
                            let o = &belts[*j as usize];
                            o.shape.flat_out() && o.pos + DIRS[o.dir as usize] == b.pos
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

        // A belt of `shape` facing `dir` stands at `pos`.
        let shaped = |pos: IVec3, shape: Shape, dir: u8| {
            belt_at(pos).filter(|&j| belts[j as usize].shape == shape && belts[j as usize].dir == dir)
        };
        let start = |belt: u32| Link::Belt { belt, mid: false };
        // Where an item travelling `dir` goes when it reaches cell `t` from the cell behind, at `t`'s
        // level: onto a belt (its start, or its middle from the side), into a machine, or onto the high
        // end of a down ramp below an empty `t`.
        let into = |t: IVec3, dir: u8| match at.get(&t) {
            Some(Slot::Belt(j)) => {
                let o = &belts[*j as usize];
                if o.shape != Shape::Flat {
                    if o.dir == dir && o.shape.fed_from_back() {
                        start(*j)
                    } else {
                        Link::None
                    }
                } else if o.dir == opposite(dir) {
                    Link::None
                } else {
                    let from_back = o.dir == dir || curves[*j as usize] == Some(opposite(dir));
                    Link::Belt { belt: *j, mid: !from_back }
                }
            }
            Some(&s) if s.is_sink() => Link::Machine(s),
            Some(_) => Link::None,
            None => shaped(t - UP, Shape::Down, dir).map_or(Link::None, start),
        };
        let lifts: Vec<(bool, bool)> = belts
            .iter()
            .map(|b| {
                let lift = |pos| b.shape == Shape::Lift && shaped(pos, Shape::Lift, b.dir).is_some();
                (lift(b.pos - UP), lift(b.pos + UP))
            })
            .collect();
        let outs: Vec<Link> = belts
            .iter()
            .zip(&lifts)
            .map(|(b, &(_, above))| {
                let d = DIRS[b.dir as usize];
                match b.shape {
                    Shape::Flat | Shape::Down | Shape::Exit => into(b.pos + d, b.dir),
                    Shape::Up => into(b.pos + d + UP, b.dir),
                    Shape::Lift if above => shaped(b.pos + UP, Shape::Lift, b.dir).map_or(Link::None, start),
                    Shape::Lift => into(b.pos + d + UP, b.dir),
                    Shape::Entry => (1..=UNDERPASS_RANGE)
                        .find_map(|k| shaped(b.pos + IVec3::new(d.x * k, 0, d.z * k), Shape::Exit, b.dir))
                        .map_or(Link::None, start),
                }
            })
            .collect();

        // Machines feed belts that lead away from them (not down ramps or underpass exits).
        let lead_away = |pos: IVec3, s: u8| {
            let o = belt_at(pos + DIRS[s as usize]);
            o.filter(|&j| belts[j as usize].dir == s && belts[j as usize].shape.fed_from_back())
        };
        let feeds = |pos: IVec3| -> Vec<u32> { (0..4u8).filter_map(|s| lead_away(pos, s)).collect() };
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
        let router_outs: Vec<[Option<u32>; 3]> =
            self.routers.iter().map(|r| r.out_dirs().map(|s| lead_away(r.pos, s))).collect();

        for (((b, c), o), l) in self.belts.iter_mut().zip(curves).zip(outs).zip(lifts) {
            b.curve_from = c;
            b.out = o;
            (b.lift_below, b.lift_above) = l;
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
        for (r, o) in self.routers.iter_mut().zip(router_outs) {
            r.outs = o;
        }

        self.power =
            Power::rebuild(&self.poles, &self.generators, &self.miners, &self.constructors, &self.routers, &self.labs);

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
