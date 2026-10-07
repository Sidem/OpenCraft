//! Ids of the items of Milestone 10 and after (chemistry, washing, research centers, nuclear power, gold science), appended
//! in id order; their rows are the end of `EXTRA` in `item.rs`. To add one: a constant here and its row there.

use super::ItemId;

/// An empty canister (the Oil Processing tech): pressed from steel, it comes back from every fluid it carried.
pub const EMPTY_CANISTER: ItemId = ItemId(348);
/// A canister of crude oil, filled by a pumpjack (`factory/process/pump.rs`).
pub const CRUDE_CANISTER: ItemId = ItemId(349);
/// The refinery's streams (the Refining tech; `factory/process/refinery.rs`): canisters of naphtha (light), diesel
/// (middle) and heavy oil, and the sulfur it leaves.
pub const NAPHTHA_CANISTER: ItemId = ItemId(350);
pub const DIESEL_CANISTER: ItemId = ItemId(351);
pub const HEAVY_OIL_CANISTER: ItemId = ItemId(352);
pub const SULFUR: ItemId = ItemId(353);
/// The chemical plant's products (the Plastics tech, `factory/process/refinery.rs`): plastic, and canisters of acid
/// and lubricant.
pub const PLASTIC: ItemId = ItemId(354);
pub const ACID_CANISTER: ItemId = ItemId(355);
pub const LUBRICANT_CANISTER: ItemId = ItemId(356);
/// The electrolyser's gases (the Electrolysis tech, `factory/process/refinery.rs`), canisters of hydrogen and oxygen.
pub const HYDROGEN_CANISTER: ItemId = ItemId(357);
pub const OXYGEN_CANISTER: ItemId = ItemId(358);
/// Washed ore (the Ore Washing tech): crushed ore cleaned in the washer, one ingot for one.
pub const WASHED_IRON: ItemId = ItemId(359);
pub const WASHED_COPPER: ItemId = ItemId(360);
pub const WASHED_BAUXITE: ItemId = ItemId(361);
/// Research center tiers Mk2–Mk4 (Mk1 is the block): a tier item places the center at that tier.
pub const RESEARCH_CENTER_MK2: ItemId = ItemId(362);
pub const RESEARCH_CENTER_MK3: ItemId = ItemId(363);
pub const RESEARCH_CENTER_MK4: ItemId = ItemId(364);
/// A fuel cell (the Nuclear Power tech): enriched in the centrifuge, burns in the reactor.
pub const FUEL_CELL: ItemId = ItemId(365);
/// The gold science pack and the gold kit (the Gold Science and Mk5 Machines techs), and the Mk5 machines (the kit
/// raises machines to Mk5, `factory/upgrades.rs`).
pub const GOLD_PACK: ItemId = ItemId(366);
pub const GOLD_KIT: ItemId = ItemId(367);
pub const MINER_MK5: ItemId = ItemId(368);
pub const SMELTER_MK5: ItemId = ItemId(369);
pub const CONSTRUCTOR_MK5: ItemId = ItemId(370);
pub const ASSEMBLER_MK5: ItemId = ItemId(371);
pub const BLAST_FURNACE_MK5: ItemId = ItemId(372);
pub const DRONE_PORT_MK5: ItemId = ItemId(373);
pub const RESEARCH_CENTER_MK5: ItemId = ItemId(374);
/// The underpass tiers above Mk1 (Mk1 is the block's own item): longer reach, the belt speed of the same Mk.
pub const UNDERPASS_MK2: ItemId = ItemId(375);
pub const UNDERPASS_MK3: ItemId = ItemId(376);
pub const UNDERPASS_MK4: ItemId = ItemId(377);
/// A recycling coin (the Recycling tech): what a recycler pays for the items it destroys (`recipes/recycling.rs`). It stacks to
/// [`COIN_STACK`], is worth nothing itself and is not offered in creative worlds.
pub const COIN: ItemId = ItemId(378);
pub const COIN_STACK: u32 = 1024;
