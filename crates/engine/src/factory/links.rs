//! Where items go. `Slot` names a machine, `Link` is where a belt or miner delivers, `Sinks` borrows
//! the machines that take items and `deliver` hands one over. `relink` rebuilds the derived data after
//! any machine is added or removed: belt outputs (for every belt shape) and corner curves, the outputs of every other machine,
//! and the downstream-first belt update order, and the power grids (`power.rs`). Nothing here is saved; links are a pure function of the
//! machines and their positions. A multi-block machine takes and gives only at its ports (`footprint/`).
//! A new machine that takes items: an arm in `Slot::is_sink` and in `Sinks`.

use crate::item::ItemId;
use crate::math::IVec3;

use super::belt::Belt;
use super::belt_shape::{derive_slopes, Shape, UNDERPASS_RANGE};
use super::generator::Generator;
use super::lab::Lab;
use super::power::Power;
use super::process::{self, Processor};
use super::router::Router;
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
    Process(u32),
    Router(u32),
    Generator(u32),
    Pole(u32),
    Lab(u32),
    Pipe(u32),
    Quarry(u32),
    Sensor(u32),
    Rail(u32),
}

impl Slot {
    /// Whether belts and miners can deliver into it (belts are linked separately).
    pub(super) fn is_sink(self) -> bool {
        match self {
            Slot::Storage(_) | Slot::Process(_) | Slot::Router(_) | Slot::Generator(_) | Slot::Lab(_) => true,
            Slot::Belt(_)
            | Slot::Miner(_)
            | Slot::Pole(_)
            | Slot::Pipe(_)
            | Slot::Quarry(_)
            | Slot::Sensor(_)
            | Slot::Rail(_) => false,
        }
    }

    pub(super) fn kind(self) -> Kind {
        match self {
            Slot::Belt(_) => Kind::Belt,
            Slot::Miner(_) => Kind::Miner,
            Slot::Storage(_) => Kind::Storage,
            Slot::Process(_) => Kind::Process,
            Slot::Router(_) => Kind::Router,
            Slot::Generator(_) => Kind::Generator,
            Slot::Pole(_) => Kind::Pole,
            Slot::Lab(_) => Kind::Lab,
            Slot::Pipe(_) => Kind::Pipe,
            Slot::Quarry(_) => Kind::Quarry,
            Slot::Sensor(_) => Kind::Sensor,
            Slot::Rail(_) => Kind::Rail,
        }
    }
}

/// The machines items can be delivered into, borrowed apart from the belts and miners.
pub(crate) struct Sinks<'a> {
    pub(super) storages: &'a mut [Storage],
    pub(super) processors: &'a mut [Processor],
    pub(super) routers: &'a mut [Router],
    pub(super) generators: &'a mut [Generator],
    pub(super) labs: &'a mut [Lab],
    /// Which machine recipes research allows, by index (processors take only what these use).
    pub(super) unlocked: &'a [bool],
}

impl Sinks<'_> {
    /// Whether the sink at `slot` takes one `item` now.
    pub(super) fn can_accept(&self, slot: Slot, item: ItemId) -> bool {
        match slot {
            Slot::Storage(i) => self.storages[i as usize].buf.can_accept(item),
            Slot::Process(i) => self.processors[i as usize].room_for(item, self.unlocked) > 0,
            Slot::Router(i) => self.routers[i as usize].can_accept(),
            Slot::Generator(i) => self.generators[i as usize].room_for(item) > 0,
            Slot::Lab(i) => self.labs[i as usize].room_for(item) > 0,
            Slot::Belt(_)
            | Slot::Miner(_)
            | Slot::Pole(_)
            | Slot::Pipe(_)
            | Slot::Quarry(_)
            | Slot::Sensor(_)
            | Slot::Rail(_) => false,
        }
    }

    /// Hands one `item` to the sink at `slot`; false if it doesn't take it.
    pub(super) fn accept(&mut self, slot: Slot, item: ItemId) -> bool {
        match slot {
            Slot::Storage(i) => self.storages[i as usize].buf.add(item, 1) == 0,
            Slot::Process(i) => self.processors[i as usize].accept(item, self.unlocked),
            Slot::Router(i) => self.routers[i as usize].accept(item),
            Slot::Generator(i) => self.generators[i as usize].accept(item),
            Slot::Lab(i) => self.labs[i as usize].accept(item),
            Slot::Belt(_)
            | Slot::Miner(_)
            | Slot::Pole(_)
            | Slot::Pipe(_)
            | Slot::Quarry(_)
            | Slot::Sensor(_)
            | Slot::Rail(_) => false,
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
        derive_slopes(&mut self.belts, &self.at);
        let at = &self.at;
        let belts = &self.belts;
        let processors = &self.processors;
        // Whether the sink `s` takes items into its cell `cell` from `from` (multi-block machines: at a port).
        let takes = |s: Slot, cell: IVec3, from: IVec3| match s {
            Slot::Process(i) => processors[i as usize].takes_from(cell, from),
            _ => true,
        };
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
            Some(&s) if s.is_sink() && takes(s, t, t - DIRS[dir as usize]) => Link::Machine(s),
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
        // Miners and quarries: belts leading away, then machines on every face but `skip` (a miner's drill).
        let extractor_outs = |pos: IVec3, skip: usize| -> Vec<Link> {
            let mut v: Vec<Link> = feeds(pos).into_iter().map(|belt| Link::Belt { belt, mid: false }).collect();
            for (f, &n) in FACES.iter().enumerate() {
                if let Some(&s) = at.get(&(pos + n)).filter(|&&s| f != skip && s.is_sink() && takes(s, pos + n, pos)) {
                    v.push(Link::Machine(s));
                }
            }
            v
        };
        let miner_outs: Vec<Vec<Link>> = self.miners.iter().map(|m| extractor_outs(m.pos, m.drill as usize)).collect();
        // A quarry's box starts a cell away, so it delivers on every side.
        let quarry_outs: Vec<Vec<Link>> = self.quarries.iter().map(|q| extractor_outs(q.pos, FACES.len())).collect();
        let storage_outs: Vec<Vec<u32>> = self.storages.iter().map(|s| feeds(s.pos)).collect();
        let process_outs: Vec<Vec<u32>> = processors
            .iter()
            .map(|p| p.out_faces().into_iter().filter_map(|(c, s)| lead_away(c, s)).collect())
            .collect();
        let process_side_outs: Vec<Vec<u32>> = processors
            .iter()
            .map(|p| p.side_faces().into_iter().filter_map(|(c, s)| lead_away(c, s)).collect())
            .collect();
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
        for (q, o) in self.quarries.iter_mut().zip(quarry_outs) {
            q.outs = o;
            q.next_out %= q.outs.len().max(1);
        }
        for (s, o) in self.storages.iter_mut().zip(storage_outs) {
            s.outs = o;
            s.next_out %= s.outs.len().max(1);
        }
        for ((p, o), side) in self.processors.iter_mut().zip(process_outs).zip(process_side_outs) {
            p.outs = o;
            p.next_out %= p.outs.len().max(1);
            p.side_outs = side;
            p.next_side %= p.side_outs.len().max(1);
        }
        for (r, o) in self.routers.iter_mut().zip(router_outs) {
            r.outs = o;
        }

        #[cfg(test)]
        if !self.by_hand {
            self.hook_by_reach();
        }
        let hooked = self.resolve_hooks();
        let (poles, gens, miners, labs) = (&self.poles, &self.generators, &self.miners, &self.labs);
        let (processors, pipework) = (&self.processors, &self.pipework);
        self.power = Power::rebuild(poles, &hooked, gens, miners, processors, labs, pipework, &self.quarries);
        self.link_pipework();
        self.link_rails();
        process::link_steam(&mut self.processors, &self.at, &mut self.pipework);

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
