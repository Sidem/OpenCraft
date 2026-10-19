//! Texture layers 186–332, from aluminium (Milestone 9) to the ruins; `tex.rs` re-exports them. The rules are
//! `tex.rs`'s: never renumber; new layers append at the end of `tex.rs`, not here.

/// Bauxite ore (`textures/ores.rs`): one look, the alternates are full.
pub const BAUXITE_ORE: u16 = 186;
/// Aluminium (`textures/aluminium.rs`): crushed bauxite, the ingot, plate and battery, the electrolytic cell.
pub const CRUSHED_BAUXITE: u16 = 187;
pub const ALUMINIUM_INGOT: u16 = 188;
pub const ALUMINIUM_PLATE: u16 = 189;
pub const BATTERY: u16 = 190;
pub const CELL_SIDE: u16 = 191;
pub const CELL_TOP: u16 = 192;
/// Trains (`textures/transport.rs`): the rail block's icon, the locomotive's and the wagon's, the docks' faces.
pub const RAIL: u16 = 193;
pub const LOCOMOTIVE: u16 = 194;
pub const WAGON: u16 = 195;
pub const DOCK_SIDE: u16 = 196;
pub const DOCK_LOAD_TOP: u16 = 197;
pub const DOCK_UNLOAD_TOP: u16 = 198;
pub const SIGNAL: u16 = 199;
/// The hover pack's icon (`textures/aluminium.rs`).
pub const HOVER_PACK: u16 = 200;
/// The cargo drone's icon (`textures/aluminium.rs`).
pub const CARGO_DRONE: u16 = 201;
/// Oil sand and uranium ore (`textures/ores.rs`): one look each, the alternates are full.
pub const OIL_SAND: u16 = 202;
pub const URANIUM_ORE: u16 = 203;
/// Canisters and the pumpjack (`textures/chemistry.rs`).
pub const EMPTY_CANISTER: u16 = 204;
pub const CRUDE_CANISTER: u16 = 205;
pub const PUMPJACK_SIDE: u16 = 206;
pub const PUMPJACK_TOP: u16 = 207;
/// The refinery's streams (naphtha, diesel, heavy oil canisters; sulfur) and the refinery's and cracker's faces.
pub const NAPHTHA_CANISTER: u16 = 208;
pub const DIESEL_CANISTER: u16 = 209;
pub const HEAVY_OIL_CANISTER: u16 = 210;
pub const SULFUR: u16 = 211;
pub const REFINERY_SIDE: u16 = 212;
pub const REFINERY_TOP: u16 = 213;
pub const CRACKER_SIDE: u16 = 214;
pub const CRACKER_TOP: u16 = 215;
/// The chemical plant's products (plastic, acid and lubricant canisters) and its faces.
pub const PLASTIC: u16 = 216;
pub const ACID_CANISTER: u16 = 217;
pub const LUBRICANT_CANISTER: u16 = 218;
pub const CHEM_SIDE: u16 = 219;
pub const CHEM_TOP: u16 = 220;
/// Hydrogen and oxygen canisters, the diesel generator's and the electrolyser's faces.
pub const HYDROGEN_CANISTER: u16 = 221;
pub const OXYGEN_CANISTER: u16 = 222;
pub const DIESEL_SIDE: u16 = 223;
pub const DIESEL_TOP: u16 = 224;
pub const ELECTROLYSER_SIDE: u16 = 225;
pub const ELECTROLYSER_TOP: u16 = 226;
/// Washed ores, tailings and the washer's faces.
pub const WASHED_IRON: u16 = 227;
pub const WASHED_COPPER: u16 = 228;
pub const WASHED_BAUXITE: u16 = 229;
pub const TAILINGS: u16 = 230;
pub const WASHER_SIDE: u16 = 231;
pub const WASHER_TOP: u16 = 232;
/// The hoist shaft's frame (cutout, like the ladder's), its ends, and the winch's side.
pub const HOIST: u16 = 233;
pub const HOIST_TOP: u16 = 234;
pub const WINCH_SIDE: u16 = 235;
/// The fuel cell icon, and the centrifuge's and the reactor's faces.
pub const FUEL_CELL: u16 = 236;
pub const CENTRIFUGE_SIDE: u16 = 237;
pub const CENTRIFUGE_TOP: u16 = 238;
pub const REACTOR_SIDE: u16 = 239;
pub const REACTOR_TOP: u16 = 240;
/// The gold science pack's flask fill and the Mk5 miner's housing.
pub const GOLD_PACK: u16 = 241;
pub const MINER_MK5_SIDE: u16 = 242;
/// The recycling coin and the recycler's faces.
pub const COIN: u16 = 243;
pub const RECYCLER_SIDE: u16 = 244;
pub const RECYCLER_TOP: u16 = 245;
/// The pure water canister, the wafer and the AI accelerator icons, and the chip fab's faces.
pub const PURE_WATER_CANISTER: u16 = 246;
pub const WAFER: u16 = 247;
pub const ACCELERATOR: u16 = 248;
pub const FAB_SIDE: u16 = 249;
pub const FAB_TOP: u16 = 250;
/// Solid accent tiles of the power grids' colours (`factory/grid_colour.rs`): `GRID_COLOURS` layers from `GRID_FIRST`.
pub const GRID_FIRST: u16 = 251;
pub const GRID_COLOURS: usize = 6;
pub const GRID_LAST: u16 = GRID_FIRST + GRID_COLOURS as u16 - 1;
/// The fibre node's post and cap, and the light of the fibre lines (textures/compute.rs).
pub const NODE_SIDE: u16 = 257;
pub const NODE_TOP: u16 = 258;
pub const FIBRE: u16 = 259;
/// The datacenter's server hall and roof (`textures/compute.rs`).
pub const DC_SIDE: u16 = 260;
pub const DC_TOP: u16 = 261;
/// The cooling tower's louvred body and fan deck (`textures/compute.rs`).
pub const TOWER_SIDE: u16 = 262;
pub const TOWER_TOP: u16 = 263;
/// The AI lab's neural-net panels and crown (`textures/compute.rs`).
pub const AI_SIDE: u16 = 264;
pub const AI_TOP: u16 = 265;
/// The optimizer's amber-banded panels and its lens crown (`textures/compute.rs`).
pub const OPT_SIDE: u16 = 266;
pub const OPT_TOP: u16 = 267;
/// The laser emitter's and receiver's panels and the beam's light (`textures/laser.rs`).
pub const EMIT_SIDE: u16 = 268;
pub const EMIT_TOP: u16 = 269;
pub const RECV_SIDE: u16 = 270;
pub const RECV_TOP: u16 = 271;
pub const BEAM: u16 = 272;
/// The hauler packs Mk3 to Mk5 (`textures/gear.rs`, which also draws Mk1 and Mk2 at 180 and 181).
pub const HAULER_PACK_MK3: u16 = 273;
pub const HAULER_PACK_MK4: u16 = 274;
pub const HAULER_PACK_MK5: u16 = 275;
/// The laser mirror's silvered glass and the data receiver's panels (`textures/laser.rs`).
pub const MIRROR: u16 = 276;
pub const DATA_SIDE: u16 = 277;
pub const DATA_TOP: u16 = 278;
/// The swarm hub's body and crown (`textures/hub.rs`).
pub const HUB_SIDE: u16 = 279;
pub const HUB_TOP: u16 = 280;
/// Smoke puffs (`factory/smoke.rs`): a dense puff and a thinning one, both cut out (`textures/smoke.rs`).
pub const SMOKE: u16 = 281;
pub const SMOKE_THIN: u16 = 282;
/// The locomotive's paint: dark green with a brass band (`textures/transport.rs`).
pub const LOCO_BODY: u16 = 283;
/// Soot-blackened iron for chimneys, smokeboxes, wheels and dark mouths (`textures/smoke.rs`).
pub const SOOT: u16 = 284;
/// Fabricator gloves (`textures/gear.rs`).
pub const CRAFT_GLOVES: u16 = 285;
/// The launch site's pad and gantry, the rocket parts' icons and the rocket's hull (`textures/orbit.rs`).
pub const LAUNCH_SIDE: u16 = 286;
pub const LAUNCH_TOP: u16 = 287;
pub const LIGHT_FRAME: u16 = 288;
pub const ROCKET_ENGINE: u16 = 289;
pub const FUEL_TANK: u16 = 290;
pub const AVIONICS: u16 = 291;
pub const ROCKET_FUEL_CANISTER: u16 = 292;
pub const ROCKET_HULL: u16 = 293;
pub const SURVEY_SATELLITE: u16 = 294;
pub const COMMS_SATELLITE: u16 = 295;
/// The power satellite's icon and the rectenna's field and side (`textures/orbit.rs`).
pub const POWER_SATELLITE: u16 = 296;
pub const RECTENNA_TOP: u16 = 297;
pub const RECTENNA_SIDE: u16 = 298;
/// The science satellite's icon and the ground station's dish and side (`textures/orbit.rs`).
pub const SCIENCE_SATELLITE: u16 = 299;
pub const GROUND_TOP: u16 = 300;
pub const GROUND_SIDE: u16 = 301;
/// The drop pod's icon and capsule, the pod launcher's and landing pad's tops and their shared side (`textures/orbit.rs`).
pub const DROP_POD: u16 = 302;
pub const POD_SIDE: u16 = 303;
pub const POD_LAUNCH_TOP: u16 = 304;
pub const POD_PAD_TOP: u16 = 305;
/// Ground cover on crossed quads, never blocks (`mesher/cover.rs`, `textures/hires.rs`).
pub const GRASS_TUFT: u16 = 306;
pub const DAISY: u16 = 307;
pub const POPPY: u16 = 308;
pub const BLUEBELL: u16 = 309;
pub const FERN: u16 = 310;
pub const REED: u16 = 311;
pub const DRY_BRUSH: u16 = 312;
pub const LICHEN: u16 = 313;
/// The forest's other trees (`textures/forest.rs`, blocks in `block/forest.rs`); logs share `LOG_TOP`.
pub const BIRCH_SIDE: u16 = 314;
pub const BIRCH_LEAVES: u16 = 315;
pub const PINE_LEAVES: u16 = 316;
pub const CACTUS_SIDE: u16 = 317;
pub const CACTUS_TOP: u16 = 318;
pub const DEAD_SIDE: u16 = 319;
pub const BIRCH_SAPLING: u16 = 320;
pub const PINE_SAPLING: u16 = 321;
/// Indicator plants over stained ground, one per stain: iron, coal, copper, limestone and quartz (`textures/hires.rs`).
pub const INDICATOR_IRON: u16 = 322;
pub const INDICATOR_COAL: u16 = 323;
pub const INDICATOR_COPPER: u16 = 324;
pub const INDICATOR_PALE: u16 = 325;
/// The ruins (`textures/ruins.rs`, blocks in `block/ruins.rs`): rusted plating, a machine husk's side, a girder and the scrap metal item.
pub const RUST_PLATE: u16 = 326;
pub const RUST_MACHINE: u16 = 327;
pub const RUST_GIRDER: u16 = 328;
pub const SCRAP: u16 = 329;
/// The logbook's cover and its page edge (`textures/ruins.rs`).
pub const LOGBOOK_SIDE: u16 = 330;
pub const LOGBOOK_TOP: u16 = 331;
pub const VAULT_DOOR: u16 = 332;
