//! What the core reports outward (`SimEvent`): pushed into `Sim::events` during a tick, drained by `Game::handle_sim_events`
//! (`events.rs`) every tick. To tell the game about something new: a variant here and its arm there.

use crate::block::BlockId;
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};

use super::PlayerId;

/// Something that happened in the core that the authority or the view reacts to.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SimEvent {
    /// A miner is drawing ore; `pos` is the block it drills. Sent every `MINER_PULSE_TICKS`.
    MinerWorking {
        pos: IVec3,
    },
    BlockBroken {
        player: PlayerId,
        pos: IVec3,
        block: BlockId,
    },
    /// A block was placed, or the machine there upgraded in place (lock is its family's).
    BlockPlaced {
        player: PlayerId,
        pos: IVec3,
        block: BlockId,
    },
    /// A door at `pos` swung to `block` (its new state: `doors.rs`).
    DoorMoved {
        pos: IVec3,
        block: BlockId,
    },
    /// Items entered a player's inventory from the world (a pickup or a machine's contents).
    Gained {
        player: PlayerId,
        item: ItemId,
        count: u32,
    },
    /// A player crafted `count` of `item` (some may have been thrown for lack of room).
    Crafted {
        player: PlayerId,
        item: ItemId,
        count: u32,
    },
    /// Loose items to spawn at `pos` with velocity `vel` (a broken block's drops).
    Dropped {
        pos: Vec3,
        vel: Vec3,
        item: ItemId,
        count: u32,
    },
    /// A player broke a deposit's ore block by hand: they keep `kept` ore and `lost` units of the deposit are gone.
    HandMined {
        player: PlayerId,
        ore: BlockId,
        kept: u32,
        lost: u32,
    },
    /// A launch site's rocket went up with a satellite (`item`); `pos` is the site's anchor.
    Launched {
        pos: IVec3,
        item: ItemId,
    },
    /// The pod launcher at `pos` sent a drop pod up.
    PodSent {
        pos: IVec3,
    },
    /// A drop pod touched down on the landing pad at `pos` (its cargo lands as the pad has room).
    PodLanded {
        pos: IVec3,
    },
    /// A player's tool broke a block with its last use and is gone.
    ToolWornOut {
        player: PlayerId,
        item: ItemId,
    },
    /// A leaf decayed (block timers): it is gone (a sapling it left comes as a `Dropped`).
    LeafDecayed {
        pos: IVec3,
    },
    /// The sapling at `pos` grew into a tree.
    TreeGrew {
        pos: IVec3,
    },
    /// A quarry dug the `block` at `pos` (its drop went into the quarry).
    QuarryDug {
        pos: IVec3,
        block: BlockId,
    },
    /// A machine made `count` of `item` (a miner's ore, a quarry's block, a finished batch, byproducts included),
    /// for the production graphs (`analytics/`).
    Produced {
        item: ItemId,
        count: u32,
    },
    /// A machine used up `count` of `item` (a batch's inputs, burned fuel, research packs), for the graphs.
    Consumed {
        item: ItemId,
        count: u32,
    },
    /// Loose items to throw out in front of a player (dropping, or no room in the inventory).
    Thrown {
        player: PlayerId,
        item: ItemId,
        count: u32,
    },
}
