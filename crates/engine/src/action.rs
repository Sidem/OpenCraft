//! Actions: the only way anything changes the core (`Sim`). Players' input, the authority (pickups)
//! and debug helpers all queue actions with `Sim::queue`; `Sim::step` applies them at a tick boundary.
//!
//! Actions carry resolved data (positions, slots, facing), never "what the player is looking at", so
//! any peer can apply them without that player's camera. `apply` validates against the current state
//! and quietly does nothing when an action no longer fits (the block changed, the slot is empty).
//! Results leave as `SimEvent`s. To add an action: a variant here, its arm in `action/apply.rs`
//! (`apply_to_player`, or `apply` if it doesn't need the player to be here yet) and its bytes in
//! `action/codec.rs` (co-op sends actions to every peer).

use crate::block::BlockId;
use crate::factory::Job;
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    /// Breaks the block at `pos` by hand, with whatever the player holds: ore keeps `HAND_YIELD`
    /// (more with a good pickaxe: tools.rs) and costs its deposit a block; a tool for the block loses a
    /// use; machines drop their contents.
    BreakBlock {
        pos: IVec3,
    },
    /// Places one item from `slot` at `pos`. `facing` is the belt direction (`factory::dir_from_yaw`;
    /// a multi-block machine's turn); `against` is the clicked block, which gives a miner its drill face
    /// and deposit. A multi-block machine takes `pos` as its anchor and needs every cell free.
    PlaceBlock {
        pos: IVec3,
        slot: u8,
        facing: u8,
        against: IVec3,
    },
    /// Turns the belt or router at `pos` a quarter turn clockwise (the R key).
    Rotate {
        pos: IVec3,
    },
    /// Empties the box or miner at `pos` into the inventory, or takes a machine's output.
    TakeContents {
        pos: IVec3,
    },
    /// Chooses what the machine at `pos` makes (`MACHINE_RECIPES` index; `u16::MAX` for nothing).
    /// The inputs it held go back to the player.
    SetRecipe {
        pos: IVec3,
        recipe: u16,
    },
    /// Sets what the filter at `pos` sends straight on (`NONE` for nothing).
    SetFilter {
        pos: IVec3,
        item: ItemId,
    },
    /// Chooses the rule of the sensor at `pos` (`factory::sensor::RULES` index).
    SetSensor {
        pos: IVec3,
        rule: u8,
    },
    /// Plants a ghost (`ghosts.rs`) of what the item in `slot` places, at `pos` and `facing`; nothing is used.
    PlaceGhost {
        pos: IVec3,
        slot: u8,
        facing: u8,
    },
    /// Plants a ghost of `block` at `pos` outright (a stamped blueprint: `blueprint/`); nothing is used.
    PlantGhost {
        pos: IVec3,
        block: BlockId,
        facing: u8,
        tier: u8,
    },
    /// Marks the block (or machine) at `pos` for tear-down: a ghost of air that drones break (`drones/`).
    MarkRemoval {
        pos: IVec3,
    },
    /// Starts or stops the jetpack's thrust (`helpers/`): the player holds jump in the air with one in the pack.
    Jetpack {
        on: bool,
    },
    /// Sends the personal drone to fetch `item` from the nearest box to `at`, the player's cell (`helpers/`).
    Fetch {
        item: ItemId,
        at: IVec3,
    },
    /// Removes the ghost covering `pos`.
    RemoveGhost {
        pos: IVec3,
    },
    /// Sets the quarry at `pos`'s box (`factory::WIDTHS` and `DEPTHS` indices) and whether it is paused.
    SetQuarry {
        pos: IVec3,
        width: u8,
        depth: u8,
        paused: bool,
    },
    /// Marks a terraforming site between corners `a` and `b`: `job` to `level` (`Sites::mark` refuses misfits).
    MarkSite {
        a: (i32, i32),
        b: (i32, i32),
        level: i32,
        job: Job,
    },
    /// Marks a tunnel site from `from` towards `to` with section `size` (`Sites::mark_tunnel` refuses what doesn't fit).
    MarkTunnel {
        from: IVec3,
        to: IVec3,
        size: u8,
    },
    /// Removes the terraforming site with this id.
    RemoveSite {
        id: u32,
    },
    /// Wires the power pole at `pole` to the machine or pole at `to` (`Factory::connect`: in range, free slots).
    Connect {
        pole: IVec3,
        to: IVec3,
    },
    /// Cuts the wire between the pole at `pole` and what stands at `to`.
    Disconnect {
        pole: IVec3,
        to: IVec3,
    },
    /// Puts the locomotive in `slot` on the rail node at `pos` (`place_train`), or couples the wagon there (`couple`).
    PlaceTrain {
        pos: IVec3,
        slot: u8,
    },
    /// Picks up the train nearest the rail node at `pos`, wagons and cargo too, as items.
    TakeTrain {
        pos: IVec3,
    },
    /// Adds `dock` to the schedule of the train near `node`, or with `clear` empties it (`Factory::set_stop`).
    TrainStop {
        node: IVec3,
        dock: IVec3,
        clear: bool,
    },
    /// Takes the signal off the rail node at `pos` back into the inventory, or puts the one in `slot` on it.
    ToggleSignal {
        pos: IVec3,
        slot: u8,
    },
    /// Starts or stops the hover pack's hover (`helpers/`): the player holds jump in the air with one in the pack.
    Hover {
        on: bool,
    },
    /// Sets the cargo route of the drone port with a cell at `from` to the port with one at `to`, or with `clear`
    /// removes it (`drones/cargo.rs`).
    SetRoute {
        from: IVec3,
        to: IVec3,
        clear: bool,
    },
    /// The hover pack starts (`on`) or stops charging from the power pole at `pole` (`helpers/`): the hands send it
    /// as the player walks into or out of the pole's reach.
    Charge {
        pole: IVec3,
        on: bool,
    },
    /// Raises the tiered machine at `pos` one tier with kits from the inventory (`factory/upgrades.rs`).
    Upgrade {
        pos: IVec3,
    },
    /// Chooses what every lab in the world researches (`u8::MAX` to stop); only an available tech.
    SetResearch {
        tech: u8,
    },
    /// Puts as many of `item` from the inventory into the machine at `pos` as it takes.
    Insert {
        pos: IVec3,
        item: ItemId,
    },
    /// Queues `times` crafts of a hand recipe (`crafting.rs`), with the crafts of any missing parts; what
    /// the inventory can't pay for is left out.
    Craft {
        recipe: u16,
        times: u32,
    },
    /// Cancels the player's `order`th queued craft, giving back what it took.
    CancelCraft {
        order: u16,
    },
    /// Equipment-panel click on worn-gear `slot` (`equipment.rs`): the cursor stack swaps with the gear (only gear for
    /// that slot goes on); `shift` takes it off into the inventory.
    ClickGear {
        slot: u8,
        shift: bool,
    },
    /// Inventory-screen click; `shift` moves the stack between hotbar and backpack (gear goes on instead).
    ClickSlot {
        slot: u8,
        shift: bool,
    },
    /// Box-screen click on the box's `slot`: like `ClickSlot` with the cursor stack; `shift` moves the
    /// stack into the inventory (what doesn't fit stays).
    ClickBox {
        pos: IVec3,
        slot: u8,
        shift: bool,
    },
    /// Box-screen shift-click on inventory `slot`: moves the stack into the box (what doesn't fit stays).
    StoreSlot {
        pos: IVec3,
        slot: u8,
    },
    /// Shift-right-click on inventory `slot`: moves every stack of its item between hotbar and backpack
    /// (until the other side is full).
    QuickMoveAll {
        slot: u8,
    },
    /// Box-screen shift-right-click on inventory `slot`: moves every stack of its item into the box.
    StoreAll {
        pos: IVec3,
        slot: u8,
    },
    /// Box-screen shift-right-click on the box's `slot`: moves every stack of its item into the inventory.
    TakeAll {
        pos: IVec3,
        slot: u8,
    },
    /// Right-click on inventory `slot` with the cursor stack: takes half of the slot (rounded up) onto an empty cursor,
    /// or half of what is left onto a cursor holding the same item. With `shift` (the cursor holds a stack): puts one
    /// item of the cursor into the slot.
    RightClickSlot {
        slot: u8,
        shift: bool,
    },
    /// The same right-click on the box's `slot`.
    RightClickBox {
        pos: IVec3,
        slot: u8,
        shift: bool,
    },
    /// Shift-right-click outside the inventory screen: throws one item of the cursor stack (a tool goes whole).
    ThrowCursor,
    /// Sorts the backpack (the hotbar keeps its layout): stacks merged, then ordered by item.
    SortInventory,
    /// Sorts the slots of the box at `pos` the same way.
    SortBox {
        pos: IVec3,
    },
    /// The inventory screen closed: the cursor stack goes back, or is thrown if there is no room.
    CloseInventory,
    SelectSlot {
        slot: u8,
    },
    ScrollSlot {
        delta: i8,
    },
    DropSelected {
        count: u32,
    },
    /// Issued by the authority when a loose item reaches the player.
    PickUp {
        item: ItemId,
        count: u32,
    },
    /// Debug and creative only.
    Give {
        item: ItemId,
        count: u32,
    },
    /// The player joins (nothing happens if it is already here): with what it had when it left, if
    /// `key` is away, else with an empty inventory.
    Join {
        key: u64,
    },
    /// The player leaves and its id is free again. With a key, its inventory and `pos` (where its
    /// body stood) wait in `Sim::away`; without one they are gone.
    Leave {
        pos: Vec3,
    },
}

mod apply;
mod blocks;
mod clicks;
mod codec;
mod multiblock;
#[cfg(test)]
mod tests;
