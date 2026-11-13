//! The personal drone (`item::PERSONAL_DRONE`, in the pack): the player's errand runner between them and the boxes
//! and machines of their base, per player in `Helpers::drone` (core state, saved since version 71).
//!
//! Orders arrive as `Action::Drone { order, at }` (`at`: the player's cell, the drone's `home`) and queue as [`Job`]s:
//! - `Fetch { item, count, slot }`: brings that many from the boxes and machines within [`RANGE`] of home (storage
//!   first, then line boxes, then what machines made: `factory/stock.rs`), landing in pack slot `slot` first.
//! - `Store { item, count, slot }`: takes that many out of the pack (out of `slot` only, when given) to the box
//!   already holding the item, else an empty one, else a catch-all labelled Misc; never a box on a production line
//!   (`Factory::homes_near`).
//! - `PutAway`: every pack stack outside the hotbar whose item already has a box in reach goes to it, trip after trip.
//! - `Craft { recipe, times }`: queues the hand craft when it comes up, after the fetches queued with it have landed
//!   (`DroneOrder::Craft` queues the fetches the craft lacks first: `crafting::fetches_for`).
//!
//! The drone flies one [`Trip`] at a time (`trip.rs`): jobs from the front going the same way, up to [`CARGO_STACKS`]
//! stacks, a tour of stops nearest first; at each stop's tick it takes or leaves its items; on landing the hold goes
//! into the pack (`SimEvent::Gained`, what does not fit is thrown at the player's feet). What cannot be done says why
//! (`SimEvent::DroneNote`). Invariants: the hold is empty when no trip flies, and nothing is ever lost (a box gone or
//! full keeps its items in the hold, which lands in the pack); jobs start only with the drone in the pack; integer
//! ticks (each leg's length from one `sqrt`). To add an order: a `Job` arm here, in `trip.rs` and in its bytes.

pub(crate) mod trip;

#[cfg(test)]
mod tests;

use crate::bytes::{ByteReader, ByteWriter};
use crate::crafting::fetches_for;
use crate::inventory::Stack;
use crate::item::{self, ItemId, PERSONAL_DRONE};
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim, SimEvent};
use crate::TICK_RATE;

pub use trip::{next_to_put_away, Trip};

/// How far from home, in blocks, the drone reaches for a box or a machine.
pub const RANGE: i32 = 64;
/// Stacks one trip carries.
pub const CARGO_STACKS: u32 = 6;
/// Most jobs waiting at once.
pub const MAX_JOBS: usize = 24;
/// Blocks a second it flies.
pub const SPEED: f64 = 16.0;
/// Ticks it hovers over each box while it loads or unloads.
pub const DWELL: u32 = TICK_RATE / 2;
/// A fetch or store tied to no pack slot.
pub const ANY_SLOT: u8 = u8::MAX;

/// One errand waiting in the queue.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Job {
    /// `slot`: the pack slot it lands in first ([`ANY_SLOT`]: wherever it fits).
    Fetch {
        item: ItemId,
        count: u32,
        slot: u8,
    },
    /// `slot`: the pack slot it takes them from ([`ANY_SLOT`]: anywhere, last slots first).
    Store {
        item: ItemId,
        count: u32,
        slot: u8,
    },
    PutAway,
    Craft {
        recipe: u16,
        times: u16,
    },
}

/// What a player tells the drone (`Action::Drone`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DroneOrder {
    /// Queue a job (a craft first queues the fetches it lacks).
    Job(Job),
    /// Drop the waiting job at this index (the trip in flight finishes).
    Cancel(u8),
    /// Only move home: the hands send it as the player walks while jobs wait.
    Follow,
}

/// Why an order could not be done, for the player's HUD.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Note {
    /// No box or machine in reach holds the item.
    NoneInReach,
    /// Fewer were in reach than asked for: all of them came.
    OnlySome,
    /// No box in reach can take the item.
    NoRoom,
    /// The pack has no room for the item.
    PackFull,
    /// Neither the pack nor anything in reach has what the craft needs.
    CantCraft,
    /// Nothing outside the hotbar has a box of its own in reach.
    NothingToPutAway,
    /// The queue is full.
    Busy,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Drone {
    /// The player's cell at the last order: where trips start and end.
    pub home: IVec3,
    pub jobs: Vec<Job>,
    pub trip: Option<Trip>,
}

impl Drone {
    /// Whether it has nothing to do.
    pub fn idle(&self) -> bool {
        self.jobs.is_empty() && self.trip.is_none()
    }

    /// How many of `item` waiting jobs and the trip in flight will still bring.
    pub fn incoming(&self, item: ItemId) -> u32 {
        let queued = self.jobs.iter().map(|j| match *j {
            Job::Fetch { item: i, count, .. } if i == item => count,
            _ => 0,
        });
        let flying = self.trip.iter().filter(|t| !t.store).flat_map(|t| &t.stops).filter(|s| s.item == item);
        queued.sum::<u32>() + flying.map(|s| s.count).sum::<u32>()
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.home);
        w.u8(self.jobs.len() as u8);
        self.jobs.iter().for_each(|j| j.write(w));
        w.bool(self.trip.is_some());
        if let Some(t) = &self.trip {
            t.write(w);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Drone> {
        let home = r.ivec3()?;
        let n = r.u8()? as usize;
        if n > MAX_JOBS {
            return None;
        }
        let jobs = (0..n).map(|_| Job::read(r)).collect::<Option<Vec<_>>>()?;
        let trip = if r.bool()? { Some(Trip::read(r)?) } else { None };
        Some(Drone { home, jobs, trip })
    }

    /// A save from before version 71 kept one errand: a stack of `item` from the box at `from`, then under way. It
    /// loads as that fetch, waiting to fly again.
    pub fn from_errand(item: ItemId, from: IVec3) -> Drone {
        Drone { home: from, jobs: vec![Job::Fetch { item, count: item::stack_size(item), slot: ANY_SLOT }], trip: None }
    }

    /// Adds `job`, merging it into a waiting fetch or store of the same item and slot. False when the queue is full.
    fn push(&mut self, job: Job) -> bool {
        for j in &mut self.jobs {
            match (j, job) {
                (Job::Fetch { item, count, slot }, Job::Fetch { item: i, count: n, slot: s })
                | (Job::Store { item, count, slot }, Job::Store { item: i, count: n, slot: s })
                    if *item == i && *slot == s =>
                {
                    *count = count.saturating_add(n);
                    return true;
                }
                (Job::PutAway, Job::PutAway) => return true,
                _ => {}
            }
        }
        self.jobs.len() < MAX_JOBS && {
            self.jobs.push(job);
            true
        }
    }
}

impl Job {
    fn write(&self, w: &mut ByteWriter) {
        match *self {
            Job::Fetch { item, count, slot } | Job::Store { item, count, slot } => {
                w.u8(u8::from(matches!(self, Job::Store { .. })));
                w.item(item);
                w.u32(count);
                w.u8(slot);
            }
            Job::PutAway => w.u8(2),
            Job::Craft { recipe, times } => {
                w.u8(3);
                w.u16(recipe);
                w.u16(times);
            }
        }
    }

    fn read(r: &mut ByteReader) -> Option<Job> {
        Some(match r.u8()? {
            0 => Job::Fetch { item: r.item()?, count: r.u32()?, slot: r.u8()? },
            1 => Job::Store { item: r.item()?, count: r.u32()?, slot: r.u8()? },
            2 => Job::PutAway,
            3 => Job::Craft { recipe: r.u16()?, times: r.u16()? },
            _ => return None,
        })
    }
}

impl DroneOrder {
    /// Bytes inside `Action::Drone` (`action/codec.rs`).
    pub fn write(&self, w: &mut ByteWriter) {
        match *self {
            DroneOrder::Follow => w.u8(0),
            DroneOrder::Cancel(i) => {
                w.u8(1);
                w.u8(i);
            }
            DroneOrder::Job(j) => {
                w.u8(2);
                j.write(w);
            }
        }
    }

    pub fn read(r: &mut ByteReader) -> Option<DroneOrder> {
        Some(match r.u8()? {
            0 => DroneOrder::Follow,
            1 => DroneOrder::Cancel(r.u8()?),
            2 => DroneOrder::Job(Job::read(r)?),
            _ => return None,
        })
    }
}

impl Sim {
    /// `Action::Drone`: moves the player's drone home to `at` and takes the order (the drone must be in the pack).
    pub(crate) fn drone_order(&mut self, player: PlayerId, order: DroneOrder, at: IVec3) {
        let half = self.settlements.gear_works();
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let drone = &mut core.helpers.drone;
        drone.home = at;
        let job = match order {
            DroneOrder::Follow => return,
            DroneOrder::Cancel(i) => {
                if (i as usize) < drone.jobs.len() {
                    drone.jobs.remove(i as usize);
                }
                return;
            }
            DroneOrder::Job(job) => job,
        };
        if core.inventory.count(PERSONAL_DRONE) == 0 || !job.valid() {
            return;
        }
        let note = |note, item| SimEvent::DroneNote { player, note, item };
        if job == Job::PutAway && trip::next_to_put_away(&self.factory, at, &core.inventory, &[]).is_none() {
            self.events.push(note(Note::NothingToPutAway, ItemId::NONE));
            return;
        }
        let ok = match job {
            Job::Craft { recipe, times } => {
                let research = &self.factory.research;
                let stock = |item| self.factory.stock_count(at, item, RANGE);
                match fetches_for(&core.inventory, research, half, recipe, u32::from(times), &stock) {
                    Some(list) if drone.jobs.len() + list.len() < MAX_JOBS => {
                        list.into_iter().all(|(item, count)| drone.push(Job::Fetch { item, count, slot: ANY_SLOT }))
                            && drone.push(job)
                    }
                    Some(_) => false,
                    None => {
                        self.events.push(note(Note::CantCraft, crate::recipes::RECIPES[recipe as usize].output));
                        return;
                    }
                }
            }
            _ => drone.push(job),
        };
        if !ok {
            self.events.push(note(Note::Busy, ItemId::NONE));
        }
    }

    /// One tick of every player's drone: the trip in flight moves on; else the next job starts.
    pub(crate) fn run_drones(&mut self) {
        let half = self.settlements.gear_works();
        for (slot, core) in self.players.iter_mut().enumerate() {
            let Some(core) = core else { continue };
            let player = PlayerId(slot as u8);
            let (drone, inv) = (&mut core.helpers.drone, &mut core.inventory);
            if let Some(t) = drone.trip.as_mut() {
                if t.advance(&mut self.factory) {
                    t.fill_slots(inv, player, &mut self.events);
                    let (hold, toast) = (std::mem::take(&mut t.hold), !t.store);
                    drone.trip = None;
                    for s in hold.into_iter().filter(|s| s.count > 0) {
                        land(inv, player, s, toast, &mut self.events);
                    }
                }
                continue;
            }
            if drone.jobs.is_empty() || inv.count(PERSONAL_DRONE) == 0 {
                continue;
            }
            if let Job::Craft { recipe, times } = drone.jobs[0] {
                drone.jobs.remove(0);
                if !core.crafts.enqueue(inv, &self.factory.research, half, recipe, u32::from(times)) {
                    let item = crate::recipes::RECIPES[recipe as usize].output;
                    self.events.push(SimEvent::DroneNote { player, note: Note::CantCraft, item });
                }
                continue;
            }
            drone.trip = trip::plan(drone, &self.factory, inv, player, &mut self.events);
        }
    }

    /// The player leaves: what the drone carries goes back into the pack (so `Away` keeps it).
    pub(crate) fn recall_drone(&mut self, player: PlayerId) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let hold = core.helpers.drone.trip.take().map(|t| t.hold).unwrap_or_default();
        for s in hold.into_iter().filter(|s| s.count > 0) {
            land(&mut core.inventory, player, s, false, &mut self.events);
        }
    }
}

impl Job {
    /// Whether the job names a real item or recipe and a count.
    fn valid(&self) -> bool {
        match *self {
            Job::Fetch { item, count, .. } | Job::Store { item, count, .. } => {
                count > 0 && item != PERSONAL_DRONE && item::def(item).is_some()
            }
            Job::PutAway => true,
            Job::Craft { recipe, times } => times > 0 && (recipe as usize) < crate::recipes::RECIPES.len(),
        }
    }
}

/// Puts a landed stack into the pack (a pickup toast for what fetching brought), the rest thrown at the player's feet.
fn land(inv: &mut crate::inventory::Inventory, player: PlayerId, s: Stack, toast: bool, events: &mut Vec<SimEvent>) {
    let left = inv.add(s.item, s.count);
    if toast && left < s.count {
        events.push(SimEvent::Gained { player, item: s.item, count: s.count - left });
    }
    if left > 0 {
        events.push(SimEvent::Thrown { player, item: s.item, count: left });
    }
}
