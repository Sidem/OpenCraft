//! What breaking and placing a block do to the core (`Action::BreakBlock`, `Action::PlaceBlock`): the
//! world edit, drops, deposits, factory machines and the events that follow. Split out of
//! `action.rs` to keep it small; the arms there just call these. Drones (`drones/`) build and break
//! through `put_block` and `dismantle`, which take nothing from any inventory.

use crate::block::{self, AIR};
use crate::factory;
use crate::inventory::Stack;
use crate::item::{ItemId, LOCOMOTIVE, RAIL_SIGNAL, WAGON};
use crate::math::{IVec3, Vec3};
use crate::sim::{leaf_held_by_tree, PlayerId, Sim, SimEvent};
use crate::tool_belt;

impl Sim {
    /// Puts the locomotive in `slot` on the rail node at `pos`, or couples the wagon there to the train beside it
    /// (`Action::PlaceTrain`); nothing if it does not fit.
    pub(super) fn place_train(&mut self, player: PlayerId, pos: IVec3, slot: u8) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        let held = core.inventory.slots.get(slot as usize).filter(|s| s.count > 0).map(|s| s.item);
        let done = match held {
            Some(LOCOMOTIVE) => self.factory.place_train(pos),
            Some(WAGON) => self.factory.couple(pos),
            _ => false,
        };
        if done {
            core.inventory.consume_slot(slot as usize, 1);
        }
    }

    /// Takes the signal off the rail node at `pos` into the inventory, or puts the one in `slot` on it
    /// (`Action::ToggleSignal`).
    pub(super) fn toggle_signal(&mut self, player: PlayerId, pos: IVec3, slot: u8) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        if self.factory.signal_at(pos) {
            self.factory.set_signal(pos, false);
            let left = core.inventory.add(RAIL_SIGNAL, 1);
            if left > 0 {
                self.events.push(SimEvent::Thrown { player, item: RAIL_SIGNAL, count: left });
            }
        } else if core.inventory.slots.get(slot as usize).is_some_and(|s| s.count > 0 && s.item == RAIL_SIGNAL)
            && self.factory.set_signal(pos, true)
        {
            core.inventory.consume_slot(slot as usize, 1);
        }
    }

    /// Picks up the train nearest the node at `pos`, with its wagons and cargo, as items (`Action::TakeTrain`).
    pub(super) fn take_train(&mut self, player: PlayerId, pos: IVec3) {
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        for stack in self.factory.take_train(pos).unwrap_or_default() {
            let left = core.inventory.add(stack.item, stack.count);
            if left > 0 {
                self.events.push(SimEvent::Thrown { player, item: stack.item, count: left });
            }
        }
    }

    pub(crate) fn break_block(&mut self, player: PlayerId, pos: IVec3) {
        if let Some((pos, drops)) = self.dismantle(player, pos, true) {
            self.drop_stacks(pos, drops);
        }
    }

    /// Breaks the block at `pos` (a multi-block machine whole, from any of its cells) and returns its anchor
    /// and what it leaves: the block's drop and a machine's contents. `with_tool` mines with the player's worn Tool
    /// Belt (`tool_belt/`: the ore it keeps); without it the bare hands (a drone) do. `None` when nothing breakable is there.
    pub(crate) fn dismantle(&mut self, player: PlayerId, pos: IVec3, with_tool: bool) -> Option<(IVec3, Vec<Stack>)> {
        // A multi-block machine breaks whole, from any of its cells, as its anchor (`multiblock.rs`).
        let pos = self.factory.footprint_at(pos).map_or(pos, |f| f.0);
        let id = self.world.block_anywhere_or_generate(pos);
        let def = block::def(id);
        // A grown tree is felled whole, never a block at a time (`sim/felling.rs`).
        if id == AIR || def.break_time < 0.0 || block::is_tree_wood(id) {
            return None;
        }
        // A rope comes up as a whole strand (`action/rope.rs`).
        if id == block::ROPE {
            return self.take_rope(player, pos);
        }
        // Its leaves too: they come down with it (`sim/timers.rs`, `leaf_held_by_tree`).
        if block::is_leaves(id) && leaf_held_by_tree(|p| self.world.block_anywhere_or_generate(p), pos) {
            return None;
        }
        let ore = block::is_ore(id);
        let share = if ore { self.factory.deposits.hand_mined(&mut self.world, pos) } else { None };
        let old_wet = if block::waterloggable(id) { self.world.wet_anywhere(pos) } else { 0 };
        if !self.world.set_block_anywhere(pos, block::block_of_wet(old_wet)) {
            return None;
        }
        self.events.push(SimEvent::BlockBroken { player, pos, block: id });
        self.block_changed(pos, id);
        self.clear_parts(pos);
        if id == block::TOWN_DEPOT {
            self.settlements.sleep(pos);
        }
        if id == block::ELEVATOR_ANCHOR {
            self.elevator_broken(pos);
        }
        // A town's own block drops nothing: its drones put it back for free.
        let town_block = self.settlements.broken(pos, id);
        let belt =
            if with_tool { self.player(player).map_or(ItemId::NONE, |c| c.inventory.belt()) } else { ItemId::NONE };
        if let Some(share) = share.filter(|_| with_tool) {
            let kept = tool_belt::ore_yield(belt);
            self.events.push(SimEvent::HandMined { player, ore: id, kept, lost: share.saturating_sub(kept) });
        }
        // A tiered machine drops its tier's item (`factory/tiers.rs`).
        let tiered = self.factory.tiered_at(pos).and_then(|t| factory::tiers::item_of(def.drop, t.1));
        let mut drops = if factory::is_dock(id) { self.take_dock(pos) } else { self.factory.remove(pos) };
        // Breaking a port calls its drones home as items.
        drops.extend(self.recall_drones(pos));
        drops.extend(self.recall_cargo(pos));
        if def.drop != AIR && !town_block {
            let item = tiered.unwrap_or(ItemId::block(def.drop));
            drops.insert(0, Stack { item, count: if ore { tool_belt::ore_yield(belt) } else { 1 } });
        }
        if block::is_leaves(id) && self.leaf_drops_sapling() {
            drops.push(Stack { item: block::sapling_of(id).into(), count: 1 });
        }
        drops.extend(self.salvage(id));
        Some((pos, drops))
    }

    /// Lets `stacks` fall out of the block at `pos` as loose items.
    pub(crate) fn drop_stacks(&mut self, pos: IVec3, stacks: Vec<Stack>) {
        let center = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        for s in stacks {
            let vel = Vec3::new(self.rng.range(-1.5, 1.5), 4.0, self.rng.range(-1.5, 1.5));
            self.events.push(SimEvent::Dropped { pos: center, vel, item: s.item, count: s.count });
        }
    }

    /// Builds the block in `slot` at `pos`. On a ghost of the same block it takes the ghost's facing and
    /// clears the ghost once built.
    pub(super) fn place_block(&mut self, player: PlayerId, pos: IVec3, slot: u8, facing: u8, against: IVec3) {
        let held = self.player(player).and_then(|c| c.inventory.slots.get(slot as usize)).and_then(|s| s.item.places());
        let Some(placed) = held else { return };
        let facing = self.ghosts.facing_for(pos, placed).unwrap_or(facing);
        self.build_block(player, pos, slot, facing, against);
        if self.world.block_anywhere_or_generate(pos) == placed {
            self.ghosts.built(pos, placed);
        }
    }

    fn build_block(&mut self, player: PlayerId, pos: IVec3, slot: u8, facing: u8, against: IVec3) {
        let Some(stack) = self.player(player).and_then(|c| c.inventory.slots.get(slot as usize)).copied() else {
            return;
        };
        if stack.is_empty() || !self.put_block(player, pos, stack.item, facing, against) {
            return;
        }
        if let Some(Some(core)) = self.players.get_mut(player.0 as usize) {
            core.inventory.consume_slot(slot as usize, 1);
        }
    }

    /// Puts the block `item` places at `pos` (a multi-block machine whole), if its cells are free, with the
    /// effects of a player building it (`player` is credited in the event). Takes nothing from anyone: the
    /// caller pays. False when nothing was placed.
    pub(crate) fn put_block(&mut self, player: PlayerId, pos: IVec3, item: ItemId, facing: u8, against: IVec3) -> bool {
        let Some(placed) = item.places() else { return false };
        if factory::is_dock(placed) {
            return self.put_dock(player, pos, placed, facing, against);
        }
        if let Some(fp) = factory::footprint::of(placed) {
            return self.put_footprint(player, pos, item, (placed, fp), facing);
        }
        if placed == block::SIGN {
            return self.put_sign(player, pos, facing);
        }
        if placed == block::STREET {
            return self.put_street_lamp(player, pos, facing);
        }
        if block::is_sapling(placed) && !self.can_plant(pos) || placed == block::TORCH && !self.torch_fits(pos) {
            return false;
        }
        // A pole needs ground under it or a wall beside it, never open air (`factory/pole_mount.rs`).
        let solid = |p| block::SOLID[self.world.block_anywhere_or_generate(p) as usize];
        if placed == block::POLE && !factory::pole_mount::is_held_up(solid, pos) {
            return false;
        }
        // A forester needs flat grass or dirt in front of it (`factory/process/forester/patch.rs`).
        if placed == block::FORESTER {
            let world = &mut self.world;
            let ground = factory::forester_survey(pos, facing, 0, |p| world.block_anywhere_or_generate(p));
            if ground.size.is_none() {
                return false;
            }
        }
        // A rail node stands on the ground (`factory/rail/ground.rs`).
        if placed == block::RAIL && !factory::rail_node_stands(|p| self.world.block_anywhere_or_generate(p), pos) {
            return false;
        }
        let legacy_col = crate::colour::legacy_concrete_colour(placed);
        let placed = if legacy_col.is_some() { block::CONCRETE } else { placed };
        let old = self.world.block_anywhere_or_generate(pos);
        if !crate::block::replaceable(old) || !self.world.set_block_anywhere(pos, placed) {
            return false;
        }
        if block::waterloggable(placed) {
            let wet = block::wet_of(old);
            if wet != 0 {
                self.world.set_wet_anywhere(pos, wet);
            }
        }
        if let Some(col) = legacy_col {
            self.world.set_paint_anywhere(pos, crate::colour::all_faces(col));
        }
        let tier = factory::tiers::placed_by(item).map_or(0, |(_, t)| t);
        self.factory.place(&mut self.world, placed, pos, facing, against, tier);
        self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
        self.block_changed(pos, old);
        true
    }

    /// Places a shapeable building block from `slot` at `pos` with `shape` (`shape/mod.rs`), or grows an
    /// existing same-material horizontal slab at `pos` in place (zero extra item: 1 block per cell).
    pub(super) fn place_shaped(&mut self, player: PlayerId, pos: IVec3, slot: u8, s: u8) {
        if !crate::shape::is_valid(s) {
            return;
        }
        let Some(stack) = self.player(player).and_then(|c| c.inventory.slots.get(slot as usize)).copied() else {
            return;
        };
        let Some(raw_placed) = stack.item.places().filter(|&b| !stack.is_empty() && crate::shape::is_shapeable(b))
        else {
            return;
        };
        let legacy_col = crate::colour::legacy_concrete_colour(raw_placed);
        let placed = if legacy_col.is_some() { block::CONCRETE } else { raw_placed };
        let old = self.world.block_anywhere_or_generate(pos);
        let old_shape = self.world.shape_anywhere(pos);
        if block::replaceable(old) {
            if !self.world.set_shaped_anywhere(pos, placed, s) {
                return;
            }
            if let Some(col) = legacy_col {
                self.world.set_paint_anywhere(pos, crate::colour::all_faces(col));
            }
            if let Some(Some(core)) = self.players.get_mut(player.0 as usize) {
                core.inventory.consume_slot(slot as usize, 1);
            }
            self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
            self.block_changed(pos, old);
            self.ghosts.built(pos, placed);
            return;
        }
        // In-place slab growth: same material, strictly thicker slab on the same face (or Full cube).
        if old == placed && old_shape != s {
            let grows = match (crate::shape::kind(old_shape), crate::shape::kind(s)) {
                (
                    crate::shape::Kind::Slab { quarters: a, top: ta },
                    crate::shape::Kind::Slab { quarters: b, top: tb },
                ) => ta == tb && b > a,
                (crate::shape::Kind::Slab { .. }, crate::shape::Kind::Full) => true,
                _ => false,
            };
            let prev_paint = self.world.paint_anywhere(pos);
            if grows && self.world.set_shaped_anywhere(pos, placed, s) {
                if prev_paint != 0 {
                    self.world.set_paint_anywhere(pos, prev_paint);
                }
                self.events.push(SimEvent::BlockPlaced { player, pos, block: placed });
            }
        }
    }

    /// Sprays `colour` (`0` = strip, `1..=16` = pigment hue) onto a single `face` (`0..6`) of `pos`.
    /// Consumes 1 surface charge (24 surfaces per pigment) in normal mode; free in creative and when stripping.
    pub(super) fn paint_face(&mut self, player: PlayerId, pos: IVec3, face: u8, colour: u8) {
        let face = usize::from(face);
        if face >= 6 || colour as usize > crate::colour::COUNT {
            return;
        }
        let block = self.world.block_anywhere_or_generate(pos);
        if !crate::colour::is_paintable(block) {
            return;
        }
        let current = crate::colour::get_face(self.world.paint_anywhere(pos), face);
        if current == colour {
            return;
        }
        let creative = self.mode == crate::mode::Mode::Creative;
        let Some(Some(core)) = self.players.get_mut(player.0 as usize) else { return };
        if !self.paint.spend(colour, &mut core.inventory, creative) {
            return;
        }
        self.world.set_face_paint_anywhere(pos, face, colour);
    }

    /// `Action::Connect`: a wire, or a track between two rail nodes when every point of it lies on the ground
    /// (`factory/rail/ground.rs`).
    pub(super) fn connect(&mut self, pole: IVec3, to: IVec3) {
        if let Some(curve) = self.factory.track_fit(pole, to).and(self.factory.track_curve(pole, to)) {
            if !factory::rail_faults(&curve, |p| self.world.block_anywhere_or_generate(p)).is_empty() {
                return;
            }
        }
        self.factory.connect(pole, to);
    }
}
