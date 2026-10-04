//! Equipment: what a player wears. Four slots (`SLOT_NAMES`), kept in the [`Inventory`] (`worn`, saved since
//! version 29, so a player who leaves keeps them), each taking the gear made for it ([`GEAR`], data).
//!
//! - **Back** (hauler packs) opens more backpack rows: [`Inventory::capacity`] is the base 36 slots plus the
//!   pack's. A pack comes off only while the rows it opens are empty, so no stack is ever stranded.
//! - **Boots** and **Torso** scale the body's walking, sprinting and jumping ([`Inventory::boost`], read by
//!   `authority.rs` into the body each tick). Bodies are not core state, so this never reaches the hash.
//! - **Tool belt** (the mining rig) speeds hand-breaking ([`Inventory::mining_speed`], read by `interaction.rs`).
//!
//! Gear is one item per slot and never stacks. Actions: `ClickGear` (the equipment panel: swap with the cursor, or
//! with Shift take off) and shift-click on a gear item in the inventory (`Inventory::wear_from`).
//! To add gear: an item id and row (`item.rs`), a texture (`textures/gear.rs`), a row in [`GEAR`], a recipe
//! (`recipes/gear.rs`) and a tech (`research/personal.rs`).

#[cfg(test)]
mod tests;

use crate::inventory::{Inventory, Stack, INVENTORY_SLOTS};
use crate::item::{ItemId, EXO_FRAME, HAULER_PACK, HAULER_PACK_MK2, MINING_RIG, SERVO_BOOTS, SPRING_BOOTS};
use crate::player::Boost;

pub const SLOTS: usize = 4;
pub const SLOT_NAMES: [&str; SLOTS] = ["Back", "Boots", "Torso", "Tool belt"];
const BACK: usize = 0;
const BOOTS: usize = 1;
const TORSO: usize = 2;
const BELT: usize = 3;

/// What a piece of gear changes: extra backpack slots, and multipliers (1.0 is no change).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Effect {
    pub pack: usize,
    /// Walking and sprinting both; `sprint` multiplies sprinting on top.
    pub walk: f64,
    pub sprint: f64,
    pub jump: f64,
    pub mining: f64,
}

const NONE: Effect = Effect { pack: 0, walk: 1.0, sprint: 1.0, jump: 1.0, mining: 1.0 };

pub struct Gear {
    pub item: ItemId,
    /// Which equipment slot it goes in.
    pub slot: usize,
    pub effect: Effect,
    /// What it does, in the player's words (the inventory tooltip).
    pub blurb: &'static str,
}

/// Jumping 1.32 blocks becomes about 2: the speed scales by the square root of the heights.
const SPRING_JUMP: f64 = 1.23;

pub const GEAR: [Gear; 6] = [
    Gear { item: HAULER_PACK, slot: BACK, effect: Effect { pack: 9, ..NONE }, blurb: "+9 backpack slots" },
    Gear { item: HAULER_PACK_MK2, slot: BACK, effect: Effect { pack: 18, ..NONE }, blurb: "+18 backpack slots" },
    Gear { item: SPRING_BOOTS, slot: BOOTS, effect: Effect { jump: SPRING_JUMP, ..NONE }, blurb: "Jump 2 blocks high" },
    Gear { item: SERVO_BOOTS, slot: BOOTS, effect: Effect { walk: 1.15, ..NONE }, blurb: "Walk and sprint 15% faster" },
    Gear { item: EXO_FRAME, slot: TORSO, effect: Effect { sprint: 1.1, ..NONE }, blurb: "Sprint 10% faster" },
    Gear {
        item: MINING_RIG,
        slot: BELT,
        effect: Effect { mining: 1.5, ..NONE },
        blurb: "Break blocks by hand 50% faster",
    },
];

pub fn gear(item: ItemId) -> Option<&'static Gear> {
    GEAR.iter().find(|g| g.item == item)
}

/// What everything in `worn` does together: slots add up, multipliers multiply.
pub fn effect(worn: &[ItemId; SLOTS]) -> Effect {
    let mut e = NONE;
    for g in worn.iter().filter_map(|&i| gear(i)) {
        e.pack += g.effect.pack;
        e.walk *= g.effect.walk;
        e.sprint *= g.effect.sprint;
        e.jump *= g.effect.jump;
        e.mining *= g.effect.mining;
    }
    e
}

impl Inventory {
    /// Slots in use: the base ones and the worn pack's rows.
    pub fn capacity(&self) -> usize {
        INVENTORY_SLOTS + effect(&self.worn).pack
    }

    /// What the worn gear does to the body's movement.
    pub fn boost(&self) -> Boost {
        let e = effect(&self.worn);
        Boost { walk: e.walk, sprint: e.walk * e.sprint, jump: e.jump }
    }

    /// How much faster the worn gear breaks blocks by hand.
    pub fn mining_speed(&self) -> f32 {
        effect(&self.worn).mining as f32
    }

    /// Whether any slot past [`Inventory::capacity`] holds something (an invalid state: a pack came off full).
    pub fn stranded(&self) -> bool {
        self.slots[self.capacity()..].iter().any(|s| !s.is_empty())
    }

    /// `Action::ClickGear` on equipment slot `slot`. Without Shift: the cursor stack swaps with the worn item (only
    /// gear for that slot goes on; an empty cursor takes it off). With Shift: the worn item goes into the inventory.
    pub fn click_gear(&mut self, slot: usize, shift: bool) {
        let Some(&old) = self.worn.get(slot) else { return };
        let backup = self.clone();
        let done = if shift {
            old != ItemId::NONE && self.set_worn(slot, ItemId::NONE) && self.add(old, 1) == 0
        } else if self.cursor.is_empty() {
            let ok = old != ItemId::NONE && self.set_worn(slot, ItemId::NONE);
            self.cursor = if ok { Stack { item: old, count: 1 } } else { Stack::default() };
            ok
        } else {
            let c = self.cursor;
            let fits = c.count == 1 && gear(c.item).is_some_and(|g| g.slot == slot);
            let ok = fits && self.set_worn(slot, c.item);
            if ok {
                self.cursor = if old == ItemId::NONE { Stack::default() } else { Stack { item: old, count: 1 } };
            }
            ok
        };
        if done {
            self.version += 1;
        } else {
            *self = backup;
        }
    }

    /// Shift-click on a gear item in inventory `slot`: puts it on, and what it replaces goes into the inventory.
    /// Returns whether `slot` held gear that went on.
    pub fn wear_from(&mut self, slot: usize) -> bool {
        let Some(&stack) = self.slots.get(slot) else { return false };
        let Some(g) = gear(stack.item).filter(|_| stack.count == 1) else { return false };
        let backup = self.clone();
        let old = self.worn[g.slot];
        self.slots[slot] = Stack::default();
        let done = self.set_worn(g.slot, g.item) && (old == ItemId::NONE || self.add(old, 1) == 0);
        if done {
            self.version += 1;
        } else {
            *self = backup;
        }
        done
    }

    /// Puts `item` in equipment `slot` unless that would strand a stack beyond the new capacity.
    fn set_worn(&mut self, slot: usize, item: ItemId) -> bool {
        let old = std::mem::replace(&mut self.worn[slot], item);
        if self.stranded() {
            self.worn[slot] = old;
            return false;
        }
        true
    }
}
