//! Pointing at one item riding a belt: which one the crosshair meets, what its tag (`provenance.rs`) says, and taking it
//! (`Action::TakeBeltItem`, the one place here that changes the core). The rest are read-only queries for the target
//! readout; nothing runs unless the crosshair is on a belt.
//! `aim_boxes` is what the crosshair may hit in a cell: a belt's frame plus its items, because a tall item pokes above
//! the rails and aiming at its top should still select the belt. `hit_boxes` (the frame alone) stays the outline.
//! To say more about an item: add a line to `belt_item_report`.

use crate::block;
use crate::hitbox::Boxes;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};
use crate::recipes::{MachineRecipe, MACHINE_RECIPES};

use super::belt::Belt;
use super::describe::fmt_duration;
use super::links::Slot;
use super::process::ProcessSpec;
use super::provenance::Tag;
use super::{Factory, INSTANCE_FLOATS};

/// An item the crosshair is on: its place in the belt's list (`IN_ROUTER` for one held by a splitter or filter), what it
/// is and how far along the belt (255ths: what `Action::TakeBeltItem` names it by, since the list shifts while an action
/// travels), its tag, and its box (lowest corner, then size, in cell units).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemAim {
    pub cell: IVec3,
    pub index: usize,
    pub item: ItemId,
    pub at: u8,
    pub tag: Tag,
    pub bounds: [f32; 6],
}

/// `ItemAim::index` of an item a router holds: it has no belt, so it cannot be taken.
pub const IN_ROUTER: usize = usize::MAX;
/// A router's cell, as the box of the item it holds.
const ROUTER_BOX: [f32; 6] = [0.1, 0.0, 0.1, 0.8, 0.5, 0.8];
/// The box of an item its belt does not draw (a lift's half that the next belt shows).
const WHOLE_CELL: [f32; 6] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
/// Ticks a key's birth may be off by after an item changed hands between belts within one frame.
const BIRTH_SLACK: i64 = 2;

/// What names one item from frame to frame, though it moves and its list index shifts: what it is, the machine it came
/// from and the tick it was put on a belt (now minus the ticks it has ridden). Items of an old save share a tag, so
/// `find_item` also goes by the last place seen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemKey {
    pub item: ItemId,
    pub from: IVec3,
    pub birth: i64,
}

impl ItemAim {
    pub fn key(&self, tick: u64) -> ItemKey {
        ItemKey { item: self.item, from: self.tag.from, birth: tick as i64 - i64::from(self.tag.ticks) }
    }

    pub fn in_router(&self) -> bool {
        self.index == IN_ROUTER
    }
}

impl Factory {
    /// Where the item `key` names is now, at `tick`; of several that fit (an old save's), the one nearest `near`.
    /// Looks at every belt item, so call it only while something is selected.
    pub fn find_item(&self, key: ItemKey, tick: u64, near: IVec3) -> Option<ItemAim> {
        let fits = |item: ItemId, tag: &Tag| {
            item == key.item
                && tag.from == key.from
                && (tick as i64 - i64::from(tag.ticks) - key.birth).abs() <= BIRTH_SLACK
        };
        let far = |c: IVec3| {
            let d = [c.x - near.x, c.y - near.y, c.z - near.z].map(i64::from);
            d.iter().map(|d| d * d).sum::<i64>()
        };
        let mut best: Option<(i64, ItemAim)> = None;
        let mut consider = |cell: IVec3, aim: &dyn Fn() -> ItemAim| {
            let d = far(cell);
            if best.is_none_or(|(near, _)| d < near) {
                best = Some((d, aim()));
            }
        };
        for belt in &self.belts {
            for (index, it) in belt.items.iter().enumerate() {
                if fits(it.item, &it.tag) {
                    consider(belt.pos, &|| belt_aim(belt, index));
                }
            }
        }
        for r in self.routers.iter().filter(|r| r.held != ItemId::NONE && fits(r.held, &r.held_tag)) {
            consider(r.pos, &|| ItemAim {
                cell: r.pos,
                index: IN_ROUTER,
                item: r.held,
                at: 0,
                tag: r.held_tag,
                bounds: ROUTER_BOX,
            });
        }
        best.map(|(_, aim)| aim)
    }

    /// What the crosshair can hit in `cell`: `hit_boxes`, with a belt's items added.
    pub fn aim_boxes(&self, cell: IVec3) -> Option<Boxes> {
        let Some(&Slot::Belt(i)) = self.at.get(&cell) else { return self.hit_boxes(cell) };
        let belt = &self.belts[i as usize];
        let mut out = Vec::new();
        belt.frame(&mut out, Vec3::ZERO, 0.0);
        for it in &belt.items {
            belt.item_box(it, &mut out, Vec3::ZERO);
        }
        Boxes::from_instances(&out)
    }

    /// The item of the belt in `cell` that the ray from `origin` along the unit vector `dir` meets first.
    pub fn belt_item_under(&self, cell: IVec3, origin: Vec3, dir: Vec3) -> Option<ItemAim> {
        let Some(&Slot::Belt(i)) = self.at.get(&cell) else { return None };
        let belt = &self.belts[i as usize];
        let (mut best, mut out): (Option<(f64, usize)>, Vec<f32>) = (None, Vec::with_capacity(INSTANCE_FLOATS));
        for (index, it) in belt.items.iter().enumerate() {
            out.clear();
            if !belt.item_box(it, &mut out, Vec3::ZERO) {
                continue;
            }
            let Some(boxes) = Boxes::from_instances(&out) else { continue };
            let Some(t) = boxes.enter(origin - cell.as_vec3(), dir) else { continue };
            if best.is_none_or(|(near, _)| t < near) {
                best = Some((t, index));
            }
        }
        best.map(|(_, index)| belt_aim(belt, index))
    }

    /// Core: takes one `item` off the belt at `pos`, the one nearest progress `at` (255ths), by offering it to `take`
    /// (which returns how many of its `n` it kept: 1, or a tool's uses). A tool the taker keeps only part of stays, with
    /// the rest of its uses. Returns whether anything was taken.
    pub fn take_belt_item(
        &mut self,
        pos: IVec3,
        item: ItemId,
        at: u8,
        mut take: impl FnMut(ItemId, u32) -> u32,
    ) -> bool {
        let Some(&Slot::Belt(i)) = self.at.get(&pos) else { return false };
        let belt = &mut self.belts[i as usize];
        let want = f32::from(at) / 255.0;
        let nearest = (0..belt.items.len())
            .filter(|&k| belt.items[k].item == item && belt.shows(belt.items[k].p))
            .min_by(|&a, &b| (belt.items[a].p - want).abs().total_cmp(&(belt.items[b].p - want).abs()));
        let Some(k) = nearest else { return false };
        let taken = take(item, belt.items[k].n);
        if taken >= belt.items[k].n {
            belt.items.remove(k);
        } else {
            belt.items[k].n -= taken;
        }
        taken > 0
    }

    /// Tests: puts one `item` at progress `over` on the belt in `pos`, as if the machine at `from` had made it.
    #[cfg(test)]
    pub(crate) fn put_on_belt(&mut self, pos: IVec3, item: ItemId, from: IVec3, over: f32) {
        let Some(&Slot::Belt(i)) = self.at.get(&pos) else { panic!("no belt at {pos:?}") };
        assert!(self.belts[i as usize].accept(item, 1, Tag::made_at(from), false, over));
    }

    /// The item an aim names and its readout: how long it has ridden belts, who put it there and where that is.
    /// None when the belt has changed since (the item moved on).
    pub fn belt_item_report(&self, aim: &ItemAim) -> (ItemId, String) {
        let time = match aim.tag.seconds() {
            0 => "Just left its maker".to_string(),
            s => format!("{} on belts", fmt_duration(f64::from(s))),
        };
        let origin = match aim.tag.origin() {
            Some(from) => self.origin_words(from, aim.item, aim.cell),
            None => {
                "Origin unknown: it was already on the belt when this world was saved by an older version".to_string()
            }
        };
        (aim.item, format!("{time}\n{origin}"))
    }

    /// "Made in a Smelter from Iron Ore ×1 · 12 east, 3 up from here".
    fn origin_words(&self, from: IVec3, item: ItemId, here: IVec3) -> String {
        let Some(&slot) = self.at.get(&from) else {
            return format!("Put on the belt by a machine since removed · {}", offset_words(from - here));
        };
        let name = block::def(self.block_name_of(slot, from)).name;
        let (verb, recipe) = match slot {
            Slot::Miner(_) => ("Mined by", String::new()),
            Slot::Quarry(_) => ("Quarried by", String::new()),
            Slot::Storage(_) => ("Taken out of", String::new()),
            Slot::Process(i) => ("Made in", self.recipe_words(self.processors[i as usize].spec, item)),
            _ => ("Put on the belt by", String::new()),
        };
        format!("{verb} a {name}{recipe} · {}", offset_words(from - here))
    }

    /// The machine's block: its tier's if it has tiers, else the plain one of its kind.
    fn block_name_of(&self, slot: Slot, pos: IVec3) -> block::BlockId {
        match (self.tiered_at(pos), slot) {
            (Some((id, _)), _) => id,
            (None, Slot::Process(i)) => self.processors[i as usize].spec.block,
            _ => slot.kind().def().block,
        }
    }

    /// " from Iron Ore ×1": the recipe of this machine's kind that makes `item` (" or " the next, if two do).
    /// The machine's own recipe may have changed since, so the kind's table is the answer, not its state.
    fn recipe_words(&self, spec: &ProcessSpec, item: ItemId) -> String {
        let makes = |r: &&MachineRecipe| spec.categories.contains(&r.category) && r.outputs.iter().any(|o| o.0 == item);
        let words = |r: &MachineRecipe| {
            let inputs: Vec<String> = r.inputs.iter().map(|(i, n)| format!("{} ×{n}", item::name(*i))).collect();
            inputs.join(" + ")
        };
        let found: Vec<String> = MACHINE_RECIPES.iter().filter(makes).take(2).map(words).collect();
        if found.is_empty() {
            String::new()
        } else {
            format!(" from {}", found.join(" or "))
        }
    }
}

/// The aim for the item at `index` on `belt`, with its box as the belt draws it.
fn belt_aim(belt: &Belt, index: usize) -> ItemAim {
    let it = &belt.items[index];
    let mut out = Vec::with_capacity(INSTANCE_FLOATS);
    let drawn = belt.item_box(it, &mut out, Vec3::ZERO).then(|| Boxes::from_instances(&out)).flatten();
    let at = (it.p.clamp(0.0, 1.0) * 255.0).round() as u8;
    ItemAim { cell: belt.pos, index, item: it.item, at, tag: it.tag, bounds: drawn.map_or(WHOLE_CELL, |b| b.bounds()) }
}

/// "12 east, 3 up from here" (north is -z, as everywhere in the readouts); "right here" for a zero offset.
fn offset_words(d: IVec3) -> String {
    let part = |n: i32, plus: &str, minus: &str| match n {
        0 => None,
        n if n > 0 => Some(format!("{n} {plus}")),
        n => Some(format!("{} {minus}", -n)),
    };
    let parts: Vec<String> = [part(d.x, "east", "west"), part(d.z, "south", "north"), part(d.y, "up", "down")]
        .into_iter()
        .flatten()
        .collect();
    if parts.is_empty() {
        "right here".to_string()
    } else {
        format!("{} from here", parts.join(", "))
    }
}

#[cfg(test)]
mod tests;
