//! One flight of the personal drone: planning it from the front of the job queue, and flying it tick by tick.
//! `plan` takes jobs going the same way (fetches, or stores and put-aways) until the trip carries `CARGO_STACKS`
//! stacks, chooses where each item comes from or goes to (`factory/stock.rs`, nearest first), and orders the stops
//! as a tour from home, always on to the nearest stop left. Each stop's `tick` is when the drone hovers over it
//! (half of `DWELL` after it arrives); `total` brings it home. Stores leave the pack when the trip is planned.

use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::Factory;
use crate::inventory::{Inventory, Stack, HOTBAR_SLOTS};
use crate::item::{self, ItemId, PERSONAL_DRONE};
use crate::math::IVec3;
use crate::sim::{PlayerId, SimEvent};
use crate::TICK_RATE;

use super::{Drone, Job, Note, ANY_SLOT, CARGO_STACKS, DWELL, RANGE, SPEED};

/// Most stops one trip may hold in a save.
const MAX_STOPS: usize = 64;

/// A box or machine the trip visits: `count` of `item` to take (a fetch) or leave (a store) at tick `tick`, for pack
/// slot `slot` (the one it lands in or came from; `ANY_SLOT` for none).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stop {
    pub at: IVec3,
    pub item: ItemId,
    pub count: u32,
    pub tick: u32,
    pub slot: u8,
}

/// A flight in progress.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Trip {
    /// Leaving items in boxes (else bringing them back).
    pub store: bool,
    /// Where it took off: home when the trip was planned.
    pub from: IVec3,
    pub stops: Vec<Stop>,
    /// What it carries now.
    pub hold: Vec<Stack>,
    /// Ticks flown, and the whole trip's.
    pub ticks: u32,
    pub total: u32,
}

impl Trip {
    /// One tick: the stop due now takes or leaves its items. True when the trip is home.
    pub(super) fn advance(&mut self, factory: &mut Factory) -> bool {
        self.ticks += 1;
        for s in self.stops.iter().filter(|s| s.tick == self.ticks) {
            if self.store {
                let carried = held(&self.hold, s.item).min(s.count);
                let left = factory.store_in_boxes(&[s.at], Stack { item: s.item, count: carried }).count;
                unload(&mut self.hold, s.item, carried - left);
            } else {
                let took = factory.take_stock(s.at, s.item, s.count);
                load(&mut self.hold, s.item, took);
            }
        }
        self.ticks >= self.total
    }

    /// Landing from a fetch: what a stop brought for a pack slot goes into that slot first (while it is empty or holds
    /// the same item), with a pickup toast; the rest of the hold lands as usual.
    pub(super) fn fill_slots(&mut self, inv: &mut Inventory, player: PlayerId, events: &mut Vec<SimEvent>) {
        if self.store {
            return;
        }
        for s in self.stops.iter().filter(|s| s.slot != ANY_SLOT) {
            let n = held(&self.hold, s.item).min(s.count);
            let Some(at) = inv.open_mut().get_mut(usize::from(s.slot)) else { continue };
            let k = n.min(item::stack_size(s.item).saturating_sub(at.count));
            if k == 0 || !(at.is_empty() || at.item == s.item) {
                continue;
            }
            *at = Stack { item: s.item, count: at.count + k };
            inv.version += 1;
            unload(&mut self.hold, s.item, k);
            events.push(SimEvent::Gained { player, item: s.item, count: k });
        }
    }

    pub(super) fn write(&self, w: &mut ByteWriter) {
        w.bool(self.store);
        w.ivec3(self.from);
        w.u8(self.stops.len() as u8);
        for s in &self.stops {
            w.ivec3(s.at);
            w.item(s.item);
            w.u32(s.count);
            w.u32(s.tick);
            w.u8(s.slot);
        }
        w.u8(self.hold.len() as u8);
        self.hold.iter().for_each(|s| s.write_state(w));
        w.u32(self.ticks);
        w.u32(self.total);
    }

    pub(super) fn read(r: &mut ByteReader) -> Option<Trip> {
        let (store, from, n) = (r.bool()?, r.ivec3()?, r.u8()? as usize);
        if n > MAX_STOPS {
            return None;
        }
        let stop = |r: &mut ByteReader| {
            Some(Stop { at: r.ivec3()?, item: r.item()?, count: r.u32()?, tick: r.u32()?, slot: r.u8()? })
        };
        let stops = (0..n).map(|_| stop(r)).collect::<Option<Vec<_>>>()?;
        let n = r.u8()? as usize;
        if n > MAX_STOPS {
            return None;
        }
        let hold = (0..n).map(|_| Stack::read_state(r)).collect::<Option<Vec<_>>>()?;
        Some(Trip { store, from, stops, hold, ticks: r.u32()?, total: r.u32()? })
    }
}

/// Plans the next trip from the front of `drone.jobs`, dropping jobs that cannot be done (with a note) or are done.
pub(super) fn plan(
    drone: &mut Drone,
    factory: &Factory,
    inv: &mut Inventory,
    player: PlayerId,
    events: &mut Vec<SimEvent>,
) -> Option<Trip> {
    let home = drone.home;
    let store = !matches!(drone.jobs.first()?, Job::Fetch { .. });
    let mut p = Planner { factory, home, cargo: CARGO_STACKS, stops: Vec::new(), hold: Vec::new() };
    let mut note = |note, item| events.push(SimEvent::DroneNote { player, note, item });
    while p.cargo > 0 {
        let Some(&job) = drone.jobs.first() else { break };
        let done = match job {
            Job::Fetch { item, count, slot } if !store => p.fetch(inv, item, count, slot, &mut note),
            Job::Store { item, count, slot } if store => p.store(inv, item, count, slot, false, &mut note),
            Job::PutAway if store => p.put_away(inv),
            _ => break,
        };
        match done {
            Left::Done => {
                drone.jobs.remove(0);
            }
            Left::Some(n) => {
                if let Job::Fetch { count, .. } | Job::Store { count, .. } = &mut drone.jobs[0] {
                    *count = n;
                }
            }
            // The put-away goes on next trip.
            Left::Stay => break,
        }
    }
    if p.stops.is_empty() {
        return None;
    }
    let (stops, total) = tour(home, p.stops);
    Some(Trip { store, from: home, stops, hold: p.hold, ticks: 0, total })
}

/// What is left of a job after this trip took its share.
enum Left {
    Done,
    Some(u32),
    /// A put-away: the queue keeps it until nothing more goes.
    Stay,
}

struct Planner<'a> {
    factory: &'a Factory,
    home: IVec3,
    /// Stacks the trip can still carry.
    cargo: u32,
    stops: Vec<Stop>,
    hold: Vec<Stack>,
}

impl Planner<'_> {
    /// Plans taking up to `count` of `item` for pack slot `slot` (no more than the pack takes, nor the cargo), in
    /// the order `stock_near` gives.
    fn fetch(
        &mut self,
        inv: &Inventory,
        item: ItemId,
        count: u32,
        slot: u8,
        note: &mut impl FnMut(Note, ItemId),
    ) -> Left {
        let stack = item::stack_size(item).max(1);
        let planned: u32 = self.stops.iter().filter(|s| s.item == item).map(|s| s.count).sum();
        let room = inv.space_for(item).saturating_sub(planned);
        let want = count.min(self.cargo * stack).min(room);
        if want == 0 {
            note(Note::PackFull, item);
            return Left::Done;
        }
        let mut got = 0;
        for s in self.factory.stock_near(self.home, item, RANGE) {
            let take = s.count.min(want - got);
            self.stops.push(Stop { at: s.pos, item, count: take, tick: 0, slot });
            got += take;
            if got == want {
                break;
            }
        }
        self.cargo = self.cargo.saturating_sub(got.div_ceil(stack));
        if got == count {
            return Left::Done;
        }
        if got < want {
            // Everything in reach is coming.
            note(if got == 0 { Note::NoneInReach } else { Note::OnlySome }, item);
            return Left::Done;
        }
        if want == room {
            note(Note::PackFull, item);
            return Left::Done;
        }
        Left::Some(count - got)
    }

    /// Plans leaving up to `count` of `item` from pack slot `slot` (else from anywhere; outside the hotbar only, for a
    /// put-away) in its home box, else (not for a put-away) in an empty box, else a catch-all (`homes_near`).
    fn store(
        &mut self,
        inv: &mut Inventory,
        item: ItemId,
        count: u32,
        slot: u8,
        put_away: bool,
        note: &mut impl FnMut(Note, ItemId),
    ) -> Left {
        let stack = item::stack_size(item).max(1);
        let (from, to) = match slot {
            ANY_SLOT if put_away => (HOTBAR_SLOTS, inv.open().len()),
            ANY_SLOT => (0, inv.open().len()),
            s => (usize::from(s), usize::from(s) + 1),
        };
        let range = from.min(inv.open().len())..to.min(inv.open().len());
        let have: u32 = inv.open()[range].iter().filter(|s| s.item == item).map(|s| s.count).sum();
        let want = count.min(have).min(self.cargo * stack);
        if want == 0 {
            return Left::Done;
        }
        let mut put = 0;
        for h in self.factory.homes_near(self.home, item, RANGE) {
            // Room once this trip's earlier stops there are in; an empty box takes one new kind a trip, a catch-all any.
            let at = h.at;
            let planned = self.stops.iter().filter(|s| s.at == at).map(|s| Stack { item: s.item, count: s.count });
            let room = self.factory.room_after(at, planned, item);
            let mixed = self.stops.iter().any(|s| s.at == at && s.item != item);
            let empty = self.factory.box_count(at, item) == 0;
            if (empty && (put_away || (mixed && !h.misc))) || room == 0 {
                continue;
            }
            let n = room.min(want - put);
            self.stops.push(Stop { at, item, count: n, tick: 0, slot });
            put += n;
            if put == want {
                break;
            }
        }
        if put == 0 {
            note(Note::NoRoom, item);
            return Left::Done;
        }
        take_from_pack(inv, item, put, from, to);
        load(&mut self.hold, item, put);
        self.cargo = self.cargo.saturating_sub(put.div_ceil(stack));
        if put < want || put == count {
            return Left::Done;
        }
        Left::Some(count - put)
    }

    /// Plans storing pack stacks outside the hotbar whose item has a home box in reach, until the cargo is full.
    fn put_away(&mut self, inv: &mut Inventory) -> Left {
        let first = self.stops.len();
        let mut tried: Vec<ItemId> = Vec::new();
        while self.cargo > 0 {
            let Some(item) = next_to_put_away(self.factory, self.home, inv, &tried) else { break };
            tried.push(item);
            self.store(inv, item, u32::MAX, ANY_SLOT, true, &mut |_, _| {});
        }
        if self.stops.len() > first {
            Left::Stay
        } else {
            Left::Done
        }
    }
}

/// The first item outside the hotbar (not in `skip`, not the drone) that already has a box with room in reach of
/// `home`: what a put-away takes next.
pub fn next_to_put_away(factory: &Factory, home: IVec3, inv: &Inventory, skip: &[ItemId]) -> Option<ItemId> {
    inv.open().iter().skip(HOTBAR_SLOTS).map(|s| s.item).find(|&i| {
        i != ItemId::NONE && i != PERSONAL_DRONE && !skip.contains(&i) && factory.has_home_near(home, i, RANGE)
    })
}

/// Orders `stops` as a tour from `home` (always on to the nearest stop left, the first listed on a tie) and times it:
/// each stop's tick, and the trip's total back home.
fn tour(home: IVec3, mut left: Vec<Stop>) -> (Vec<Stop>, u32) {
    let mut out = Vec::with_capacity(left.len());
    let (mut at, mut t) = (home, 0);
    while !left.is_empty() {
        let next = (0..left.len()).min_by_key(|&i| d2(at, left[i].at)).unwrap_or(0);
        let mut s = left.remove(next);
        t += leg(at, s.at) + DWELL / 2;
        s.tick = t.max(1);
        t += DWELL - DWELL / 2;
        at = s.at;
        out.push(s);
    }
    (out, (t + leg(at, home)).max(TICK_RATE))
}

/// Ticks to fly from `a` to `b`.
pub fn leg(a: IVec3, b: IVec3) -> u32 {
    ((d2(a, b) as f64).sqrt() / SPEED * f64::from(TICK_RATE)).ceil() as u32
}

fn d2(a: IVec3, b: IVec3) -> i64 {
    let d = a - b;
    i64::from(d.x).pow(2) + i64::from(d.y).pow(2) + i64::from(d.z).pow(2)
}

/// Takes `n` of `item` out of the pack's open slots `from..to`, last slots first.
fn take_from_pack(inv: &mut Inventory, item: ItemId, mut n: u32, from: usize, to: usize) {
    inv.version += 1;
    for s in inv.open_mut().iter_mut().take(to).skip(from).rev().filter(|s| s.item == item) {
        let k = s.count.min(n);
        s.count -= k;
        n -= k;
        if s.count == 0 {
            *s = Stack::default();
        }
        if n == 0 {
            break;
        }
    }
}

fn held(hold: &[Stack], item: ItemId) -> u32 {
    hold.iter().filter(|s| s.item == item).map(|s| s.count).sum()
}

fn load(hold: &mut Vec<Stack>, item: ItemId, n: u32) {
    if n == 0 {
        return;
    }
    match hold.iter_mut().find(|s| s.item == item) {
        Some(s) => s.count += n,
        None => hold.push(Stack { item, count: n }),
    }
}

fn unload(hold: &mut Vec<Stack>, item: ItemId, n: u32) {
    if let Some(s) = hold.iter_mut().find(|s| s.item == item) {
        s.count -= n.min(s.count);
    }
    hold.retain(|s| s.count > 0);
}
