//! Derived factory data, rebuilt by `relink` after any machine is added or removed: belt outputs
//! and corner curves, miner and box outputs, and the downstream-first belt update order.
//! Nothing here is saved; it's a pure function of the machines and their positions.

use crate::math::IVec3;

use super::{opposite, Factory, Link, Slot, DIRS, FACES};

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
                    Some(Slot::Storage(k)) => Link::Storage(*k),
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
                    if let Some(Slot::Storage(k)) = at.get(&(m.pos + n)) {
                        v.push(Link::Storage(*k));
                    }
                }
                v
            })
            .collect();
        let storage_outs: Vec<Vec<u32>> = self.storages.iter().map(|s| feeds(s.pos)).collect();

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
