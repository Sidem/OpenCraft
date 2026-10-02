//! What breaking and placing a block do to the core (`Action::BreakBlock`, `Action::PlaceBlock`): the
//! world edit, tool wear, drops, deposits, factory machines and the events that follow. Split out of
//! `action.rs` to keep it small; the arms there just call these.

use super::Sim;
use crate::block::{self, BlockId, AIR, LEAVES, SAPLING};
use crate::factory;
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::sim::{PlayerId, SimEvent};
use crate::tools;

impl Sim {
    pub(super) fn break_block(&mut self, player: PlayerId, pos: IVec3) {
        // A multi-block machine breaks whole, from any of its cells, as its anchor (`multiblock.rs`).
        let pos = self.factory.footprint_at(pos).map_or(pos, |f| f.0);
        let id = self.world.block_anywhere_or_generate(pos);
        let def = block::def(id);
        if id == AIR || def.break_time < 0.0 {
            return;
        }
        let ore = block::is_ore(id);
        if ore {
            self.factory.deposits.hand_mined(&mut self.world, pos);
        }
        if !self.world.set_block_anywhere(pos, AIR) {
            return;
        }
        self.events.push(SimEvent::BlockBroken { player, pos, block: id });
        self.block_changed(pos, id);
        self.clear_parts(pos);
        let held = self.wear_tool(player, id);
        // A tiered machine drops its tier's item (`factory/tiers.rs`).
        let tiered = self.factory.tiered_at(pos).and_then(|t| factory::tiers::item_of(def.drop, t.1));
        let mut drops = self.factory.remove(pos);
        if def.drop != AIR {
            let item = tiered.unwrap_or(ItemId::block(def.drop));
            drops.insert(0, Stack { item, count: if ore { tools::ore_yield(held) } else { 1 } });
        }
        if id == LEAVES && self.leaf_drops_sapling() {
            drops.push(Stack { item: SAPLING.into(), count: 1 });
        }
        let center = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        for s in drops {
            let vel = Vec3::new(self.rng.range(-1.5, 1.5), 4.0, self.rng.range(-1.5, 1.5));
            self.events.push(SimEvent::Dropped { pos: center, vel, item: s.item, count: s.count });
        }
    }

    /// The item the player breaks `block` with (their selected slot). A tool for it loses a use (its
    /// stack count); one that wears out reports it.
    fn wear_tool(&mut self, player: PlayerId, block: BlockId) -> ItemId {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return ItemId::NONE };
        let inv = &mut core.inventory;
        let held = inv.selected_stack().item;
        if tools::tool_for(held, block).is_some() {
            inv.take_slot(inv.selected, 1);
            if inv.selected_stack().is_empty() {
                self.events.push(SimEvent::ToolWornOut { player, item: held });
            }
        }
        held
    }

    pub(super) fn place_block(&mut self, player: PlayerId, pos: IVec3, slot: u8, facing: u8, against: IVec3) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let inv = &mut core.inventory;
        let Some(stack) = inv.slots.get(slot as usize).copied() else { return };
        let Some(placed) = stack.item.places().filter(|_| !stack.is_empty()) else { return };
        if let Some(fp) = factory::footprint::of(placed) {
            return self.place_footprint(player, pos, slot, (placed, fp), facing);
        }
        if placed == SAPLING && !self.can_plant(pos) || placed == block::TORCH && !self.torch_fits(pos) {
            return;
        }
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let inv = &mut core.inventory;
        let old = self.world.block_anywhere_or_generate(pos);
        if !crate::block::replaceable(old) || !self.world.set_block_anywhere(pos, placed) {
            return;
        }
        inv.take_slot(slot as usize, 1);
        let tier = factory::tiers::placed_by(stack.item).map_or(0, |(_, t)| t);
        self.factory.place(&mut self.world, placed, pos, facing, against, tier);
        self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
        self.block_changed(pos, old);
    }
}
