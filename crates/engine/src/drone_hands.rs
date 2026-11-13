//! The local player's hands for the personal drone (`helpers/drone/`; presentation only, acting by `Action::Drone`).
//! Once a tick: while jobs wait and the player has walked [`FOLLOW_STEP`] blocks from the drone's home, a `Follow`
//! moves home along; with restock on, a hand stack that is being used up (placed, thrown, burnt) and falls to a
//! quarter stack sends the drone for a stack more, if any is in reach and none is already coming. The drone's notes
//! (`SimEvent::DroneNote`) become a line of words on the HUD for [`NOTE_TICKS`]; `drone_label` is its status line.
//! The panel's queries (what is in reach, what a craft would fetch, what each pack slot is waiting for) live here
//! too, read-only. A restock lands in the hand's own slot.
//! To add an order button: an `api/drone.rs` method that calls `order`.

use crate::action::Action;
use crate::crafting::fetches_for;
use crate::factory::stock::Stocked;
use crate::helpers::drone::{next_to_put_away, DroneOrder, Job, Note, ANY_SLOT, RANGE};
use crate::inventory::Stack;
use crate::item::{self, ItemId, PERSONAL_DRONE};
use crate::math::IVec3;
use crate::{Game, TICK_RATE};

#[cfg(test)]
mod tests;

/// Blocks the player may walk from the drone's home before the hands move it along.
pub const FOLLOW_STEP: i32 = 8;
/// How long a note stays on the HUD, in ticks.
const NOTE_TICKS: u64 = 4 * TICK_RATE as u64;

/// What the hands remember between ticks (never saved).
pub struct DroneHands {
    /// Restock the hand stack as it runs low (the panel's switch, kept by the host).
    pub restock: bool,
    /// The hand stack last tick.
    held: Stack,
    /// The latest note and the tick it ends.
    note: Option<(String, u64)>,
}

impl Default for DroneHands {
    fn default() -> DroneHands {
        DroneHands { restock: true, held: Stack::default(), note: None }
    }
}

impl Game {
    /// Once a tick: keep the drone's home near the player and restock the hand stack.
    pub(crate) fn update_drone_hands(&mut self) {
        let Some(p) = self.sim.player(self.local) else { return };
        if p.inventory.count(PERSONAL_DRONE) == 0 {
            return;
        }
        let at = self.drone_home();
        let d = at - p.helpers.drone.home;
        let far = d.x.abs().max(d.y.abs()).max(d.z.abs()) > FOLLOW_STEP;
        let follow = far && !p.helpers.drone.idle();
        let (held, hand) = (p.inventory.selected_stack(), p.inventory.selected as u8);
        let last = std::mem::replace(&mut self.drone_hands.held, held);
        let stack = item::stack_size(last.item);
        // Used up a little at a time; a whole stack moved away at once (into a box, say) is not restocked.
        let used = !last.is_empty()
            && (held.item == last.item || held.is_empty())
            && held.count < last.count
            && (last.count - held.count < stack / 2 || stack <= 2);
        let low = used && held.count <= stack / 4 && p.helpers.drone.incoming(last.item) == 0;
        let restock = self.drone_hands.restock && low && self.sim.factory.stock_count(at, last.item, RANGE) > 0;
        if restock {
            self.order(Job::Fetch { item: last.item, count: stack - held.count, slot: hand });
        } else if follow {
            self.act(Action::Drone { order: DroneOrder::Follow, at });
        }
    }

    /// Queues `job` for the local player's drone, from where they stand.
    pub(crate) fn order(&mut self, job: Job) {
        let at = self.drone_home();
        self.act(Action::Drone { order: DroneOrder::Job(job), at });
    }

    /// Drops the waiting job at `index`.
    pub(crate) fn cancel_drone_job(&mut self, index: u8) {
        let at = self.drone_home();
        self.act(Action::Drone { order: DroneOrder::Cancel(index), at });
    }

    /// The player's cell: the drone's home.
    pub(crate) fn drone_home(&self) -> IVec3 {
        self.body().pos.floor()
    }

    /// A note from the core for the local player (`events.rs`), shown for a few seconds.
    pub(crate) fn drone_noted(&mut self, note: Note, item: ItemId) {
        let name = if item == ItemId::NONE { "" } else { item::name(item) };
        let text = match note {
            Note::NoneInReach => format!("Drone: no {name} in any box or machine within {RANGE} blocks"),
            Note::OnlySome => format!("Drone: bringing all the {name} in reach"),
            Note::NoRoom => {
                format!("Drone: no box for {name} (it fills only boxes off belt lines): place an empty box, or label one Misc")
            }
            Note::PackFull => format!("Drone: no room in your pack for {name}"),
            Note::CantCraft => format!("Drone: can't gather the parts for {name}"),
            Note::NothingToPutAway => "Drone: nothing outside the hotbar has a box of its own in reach".to_string(),
            Note::Busy => "Drone: its queue is full".to_string(),
        };
        self.drone_hands.note = Some((text, self.sim.tick + NOTE_TICKS));
    }

    /// The drone's HUD line: its latest note, else what it is doing ("" with no drone or nothing to say).
    pub(crate) fn drone_label(&self) -> String {
        if let Some((text, until)) = &self.drone_hands.note {
            if self.sim.tick < *until {
                return text.clone();
            }
        }
        let Some(p) = self.sim.player(self.local).filter(|p| p.inventory.count(PERSONAL_DRONE) > 0) else {
            return String::new();
        };
        let drone = &p.helpers.drone;
        let waiting = match drone.jobs.len() {
            0 => String::new(),
            n => format!(" · {n} more waiting"),
        };
        let Some(t) = &drone.trip else {
            return if drone.jobs.is_empty() { String::new() } else { format!("Drone{waiting}") };
        };
        let secs = t.total.saturating_sub(t.ticks).div_ceil(TICK_RATE);
        let mut kinds: Vec<ItemId> = Vec::new();
        for s in &t.stops {
            if !kinds.contains(&s.item) {
                kinds.push(s.item);
            }
        }
        let count: u32 = t.stops.iter().map(|s| s.count).sum();
        let what = match kinds.as_slice() {
            [one] => format!("{} ×{count}", item::name(*one)),
            many => format!("{} kinds", many.len()),
        };
        let verb = if t.store { "storing" } else { "fetching" };
        format!("Drone {verb} {what} · back in {secs} s{waiting}")
    }

    /// Everything within the drone's reach of the player: (item, in boxes, in machines).
    pub(crate) fn stock_in_reach(&self) -> Vec<Stocked> {
        self.sim.factory.stock_totals(self.drone_home(), RANGE)
    }

    /// How many kinds a put-away would take from the pack now.
    pub(crate) fn put_away_kinds(&self) -> u32 {
        let Some(p) = self.sim.player(self.local) else { return 0 };
        let mut kinds = Vec::new();
        while let Some(item) = next_to_put_away(&self.sim.factory, self.drone_home(), &p.inventory, &kinds) {
            kinds.push(item);
        }
        kinds.len() as u32
    }

    /// The trip in flight, then each waiting job, as four numbers: kind (10 fetching, 11 storing; 0 fetch, 1 store,
    /// 2 put away, 3 craft), item (none for a trip of several kinds; a craft's product), count (a craft's times), and
    /// the trip's seconds left or the craft's recipe.
    pub(crate) fn job_rows(&self) -> Vec<u32> {
        let Some(p) = self.sim.player(self.local) else { return Vec::new() };
        let drone = &p.helpers.drone;
        let mut rows = Vec::new();
        if let Some(t) = &drone.trip {
            let first = t.stops.first().map_or(ItemId::NONE, |s| s.item);
            let item = if t.stops.iter().all(|s| s.item == first) { first } else { ItemId::NONE };
            let count = t.stops.iter().map(|s| s.count).sum();
            rows.extend([
                10 + u32::from(t.store),
                u32::from(item.0),
                count,
                t.total.saturating_sub(t.ticks).div_ceil(TICK_RATE),
            ]);
        }
        for j in &drone.jobs {
            rows.extend(match *j {
                Job::Fetch { item, count, .. } => [0, u32::from(item.0), count, 0],
                Job::Store { item, count, .. } => [1, u32::from(item.0), count, 0],
                Job::PutAway => [2, 0, 0, 0],
                Job::Craft { recipe, times } => {
                    let out = crate::recipes::RECIPES.get(recipe as usize).map_or(ItemId::NONE, |r| r.output);
                    [3, u32::from(out.0), u32::from(times), u32::from(recipe)]
                }
            });
        }
        rows
    }

    /// Every pack slot an order is tied to, six numbers each: slot, 1 if it stores from it (else it fills it), item,
    /// 1 once the drone is on its way (else waiting), the trip's progress in thousandths, its seconds left. The trip's
    /// slots come first; a slot appears once.
    pub(crate) fn slot_rows(&self) -> Vec<u32> {
        let Some(p) = self.sim.player(self.local) else { return Vec::new() };
        let drone = &p.helpers.drone;
        let mut rows: Vec<u32> = Vec::new();
        let mut add = |slot: u8, store: bool, item: ItemId, flying: [u32; 3]| {
            if slot != ANY_SLOT && !rows.chunks(6).any(|r| r[0] == u32::from(slot)) {
                rows.extend([u32::from(slot), u32::from(store), u32::from(item.0)]);
                rows.extend(flying);
            }
        };
        if let Some(t) = &drone.trip {
            let left = t.total.saturating_sub(t.ticks);
            let flying = [1, t.ticks * 1000 / t.total.max(1), left.div_ceil(TICK_RATE)];
            t.stops.iter().for_each(|s| add(s.slot, t.store, s.item, flying));
        }
        for j in &drone.jobs {
            if let Job::Fetch { item, slot, .. } | Job::Store { item, slot, .. } = *j {
                add(slot, matches!(j, Job::Store { .. }), item, [0; 3]);
            }
        }
        rows
    }

    /// What the drone would fetch so that `times` of `recipe` can be crafted, or None when it can't be gathered.
    pub(crate) fn drone_craft_fetches_for(&self, recipe: u16, times: u32) -> Option<Vec<(ItemId, u32)>> {
        let p = self.sim.player(self.local)?;
        let at = self.drone_home();
        let stock = |item| self.sim.factory.stock_count(at, item, RANGE);
        let half = self.sim.settlements.gear_works();
        fetches_for(&p.inventory, &self.sim.factory.research, half, recipe, times, &stock)
    }
}
