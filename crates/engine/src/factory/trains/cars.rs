//! Rolling stock: coupling a wagon behind a train, picking a whole train up, and what a train gives back when its
//! track goes. A wagon adds `CAR_MM` to the train's length and `CAR_SLOTS` slots to its cargo, and the track under
//! the train must still cover it all (`fit`): if it does not reach far enough back, edges behind the train are
//! put on the front of `path` (the way a train running the other direction would go on past its tail); with none
//! there, the train is pulled forward along the track ahead instead (so a locomotive just put on the first node
//! can still take wagons).

use crate::inventory::Stack;
use crate::item::{LOCOMOTIVE, WAGON};
use crate::math::{IVec3, Vec3};

use super::{Factory, Spot, Train, CAR_MM, CAR_SLOTS, MAX_CARS, PICKUP_BLOCKS};

/// Edges added behind or ahead of a train when coupling at most.
const MAX_EDGES: usize = 8;

type Edge = (IVec3, IVec3);

/// How a train's path changes to take one more wagon: edges before it, edges after it, and the new head.
struct Fit {
    before: Vec<Edge>,
    after: Vec<Edge>,
    head: i64,
}

impl Factory {
    /// What coupling a wagon to the train near the node at `pos` would do.
    pub fn couple_spot(&self, pos: IVec3) -> Spot {
        match self.train_near(pos).map(|i| &self.trains[i]) {
            None => Spot::NoTrain,
            Some(t) if t.cars >= MAX_CARS => Spot::Full,
            Some(t) if self.fit(t).is_none() => Spot::NoRoom,
            Some(_) => Spot::Ok,
        }
    }

    /// Couples a wagon behind the train near the node at `pos`. False (nothing done) unless `couple_spot` says `Ok`.
    pub fn couple(&mut self, pos: IVec3) -> bool {
        if self.couple_spot(pos) != Spot::Ok {
            return false;
        }
        let Some(i) = self.train_near(pos) else { return false };
        let Some(fit) = self.fit(&self.trains[i]) else { return false };
        let t = &mut self.trains[i];
        t.path.splice(0..0, fit.before);
        t.path.extend(fit.after);
        if t.head != fit.head {
            // Pulled forward off its stop (or a track end): it is moving again.
            t.idle = None;
        }
        t.head = fit.head;
        t.cars += 1;
        t.cargo.slots.resize(usize::from(t.cars) * CAR_SLOTS, Stack::default());
        true
    }

    /// Picks up the train nearest the node at `pos` (within `PICKUP_BLOCKS`) with its wagons and cargo.
    pub fn take_train(&mut self, pos: IVec3) -> Option<Vec<Stack>> {
        let i = self.train_near(pos)?;
        Some(stock(self.trains.remove(i)))
    }

    /// Takes every train off the track touching the node at `pos`, as the stock it gives back.
    pub(in crate::factory) fn derail_at(&mut self, pos: IVec3) -> Vec<Stack> {
        let (gone, kept): (Vec<Train>, Vec<Train>) =
            std::mem::take(&mut self.trains).into_iter().partition(|t| t.touches(pos));
        self.trains = kept;
        gone.into_iter().flat_map(stock).collect()
    }

    /// The first train with its front or its rear within `PICKUP_BLOCKS` of the node at `pos`.
    pub(super) fn train_near(&self, pos: IVec3) -> Option<usize> {
        let centre = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        let near = |p: Option<Vec3>| p.is_some_and(|p| (p - centre).length() <= PICKUP_BLOCKS + 0.5);
        self.trains.iter().position(|t| near(self.head_point(t)) || near(self.point_back(t, t.len_mm())))
    }

    /// Millimetres of track behind the tail of `t` back to the start of its path (negative: the tail is off it).
    pub(super) fn tail_room(&self, t: &Train) -> Option<i64> {
        let last = t.path.len().checked_sub(1)?;
        let mut before = 0;
        for &(a, b) in &t.path[..last] {
            before += self.edge_mm(a, b)?;
        }
        Some(before + t.head - t.len_mm())
    }

    /// How `t`'s path must change to cover one more wagon; `None` when the track runs out both ways.
    fn fit(&self, t: &Train) -> Option<Fit> {
        let room = self.tail_room(t)?;
        if let Some(before) = self.behind(t, room) {
            return Some(Fit { before, after: Vec::new(), head: t.head });
        }
        let (after, head) = self.ahead(t, CAR_MM - room.max(0))?;
        Some(Fit { before: Vec::new(), after, head })
    }

    /// The edges to put before `t.path` (tail first) so that `room` mm behind its tail grows to a wagon's length.
    fn behind(&self, t: &Train, mut room: i64) -> Option<Vec<Edge>> {
        let mut first = *t.path.first()?;
        let mut added = Vec::new();
        while room < CAR_MM {
            let c = self.next_from(first.1, first.0).filter(|_| added.len() < MAX_EDGES)?;
            room += self.edge_mm(c, first.0)?;
            first = (c, first.0);
            added.push(first);
        }
        added.reverse();
        Some(added)
    }

    /// The edges added after `t.path` and the head when the train is pulled `mm` forward along the track ahead.
    fn ahead(&self, t: &Train, mm: i64) -> Option<(Vec<Edge>, i64)> {
        let (mut a, mut b) = *t.path.last()?;
        let (mut head, mut added) = (t.head + mm, Vec::new());
        loop {
            let len = self.edge_mm(a, b)?;
            if head <= len {
                return Some((added, head));
            }
            let c = self.next_from(a, b).filter(|_| added.len() < MAX_EDGES)?;
            head -= len;
            added.push((b, c));
            (a, b) = (b, c);
        }
    }
}

/// What a train is made of: the locomotive, its wagons and what they carry.
fn stock(t: Train) -> Vec<Stack> {
    let mut all = vec![Stack { item: LOCOMOTIVE, count: 1 }];
    if t.cars > 0 {
        all.push(Stack { item: WAGON, count: u32::from(t.cars) });
    }
    all.extend(t.cargo.contents());
    all
}
