//! Block ids of Milestone 10's machines: the pumpjack, refinery, cracker, chemical plant, diesel generator, electrolyser
//! and washer, the tailings block it leaves, the research center, the water wheel, the hoist shaft and winch, the centrifuge and the reactor. Definitions: the `DEFS` rows in `mod.rs`; their processors: `factory/process/`.

use super::BlockId;

/// The pumpjack (the Oil Processing tech): drills down to an oil reservoir and fills canisters.
pub const PUMPJACK: BlockId = 83;
/// The refinery and the cracker (the Refining tech): fractional distillation of crude oil canisters.
pub const REFINERY: BlockId = 84;
pub const CRACKER: BlockId = 85;
/// The chemical plant (the Plastics tech): plastic, acid and lubricant.
pub const CHEMICAL_PLANT: BlockId = 86;
/// The diesel generator (Diesel Power) and the electrolyser (Electrolysis): canisters in, power or gas canisters out.
pub const DIESEL_GENERATOR: BlockId = 87;
pub const ELECTROLYSER: BlockId = 88;
/// The ore washer (the Ore Washing tech) and its tailings: a fill block that the crusher grinds to sand.
pub const WASHER: BlockId = 89;
pub const TAILINGS: BlockId = 90;
/// The research center (the Research Center tech): a 2×2×2 lab with eight pack slots.
pub const RESEARCH_CENTER: BlockId = 91;
/// The water wheel (the Hydropower tech): a 3×3×1 power source that reads the water touching it.
pub const WATER_WHEEL: BlockId = 92;
/// The hoist shaft (the Hoists tech): a climbable steel frame, fast while a winch beside its top cell has power.
pub const HOIST: BlockId = 93;
/// The hoist winch: a 1×1×1 power sink at the top of a shaft.
pub const WINCH: BlockId = 94;
/// The centrifuge (the Nuclear Power tech): 2×2×3, turns uranium ore into fuel cells.
pub const CENTRIFUGE: BlockId = 95;
/// The nuclear reactor (the Nuclear Power tech): 3×3×3, up to 2 MW from fuel cells, water-cooled.
pub const REACTOR: BlockId = 96;
