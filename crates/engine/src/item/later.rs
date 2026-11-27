//! Ids of the items of Milestone 10 and after (chemistry, washing, research centers, nuclear power, gold science), appended
//! in id order; their rows are the end of `EXTRA` in `item.rs`. To add one: a constant here and its row there.

use super::ItemId;

/// An empty barrel (the Oil Processing tech): pressed from steel, it comes back from every fluid it carried.
pub const EMPTY_BARREL: ItemId = ItemId(348);
/// A barrel of crude oil, filled by a pumpjack (`factory/process/pump.rs`).
pub const CRUDE_BARREL: ItemId = ItemId(349);
/// The refinery's streams (the Refining tech; `factory/process/refinery.rs`): barrels of naphtha (light), diesel
/// (middle) and heavy oil, and the sulfur it leaves.
pub const NAPHTHA_BARREL: ItemId = ItemId(350);
pub const DIESEL_BARREL: ItemId = ItemId(351);
pub const HEAVY_OIL_BARREL: ItemId = ItemId(352);
pub const SULFUR: ItemId = ItemId(353);
/// The chemical plant's products (the Plastics tech, `factory/process/refinery.rs`): plastic, and barrels of acid
/// and lubricant.
pub const PLASTIC: ItemId = ItemId(354);
pub const ACID_BARREL: ItemId = ItemId(355);
pub const LUBRICANT_BARREL: ItemId = ItemId(356);
/// The electrolyser's gases (the Electrolysis tech, `factory/process/refinery.rs`), barrels of hydrogen and oxygen.
pub const HYDROGEN_BARREL: ItemId = ItemId(357);
pub const OXYGEN_BARREL: ItemId = ItemId(358);
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
/// Compute's parts (the Wafers and Accelerators techs, `factory/process/fab.rs`): a barrel of ultra-pure water (a
/// chemical plant recipe), the wafer and the AI accelerator, both made in the chip fab.
pub const PURE_WATER_BARREL: ItemId = ItemId(379);
pub const WAFER: ItemId = ItemId(380);
pub const AI_ACCELERATOR: ItemId = ItemId(381);
/// The hauler packs Mk3 to Mk5 (Mk1 and Mk2 are 333 and 334): 18 more backpack slots each (`equipment.rs`).
pub const HAULER_PACK_MK3: ItemId = ItemId(382);
pub const HAULER_PACK_MK4: ItemId = ItemId(383);
pub const HAULER_PACK_MK5: ItemId = ItemId(384);
/// Work Gloves Mk5, once the Fabricator Gloves (worn in the Gloves slot, `equipment.rs`): hand crafting 125% faster.
pub const CRAFT_GLOVES: ItemId = ItemId(385);
/// Rocket parts (the Rocketry tech; `recipes/orbit.rs`): the assembler builds the light frame, engine, fuel tank and
/// avionics, the chemical plant fills rocket fuel barrels, and the launch site consumes them all.
pub const LIGHT_FRAME: ItemId = ItemId(386);
pub const ROCKET_ENGINE: ItemId = ItemId(387);
pub const FUEL_TANK: ItemId = ItemId(388);
pub const AVIONICS: ItemId = ItemId(389);
pub const ROCKET_FUEL_BARREL: ItemId = ItemId(390);
/// A survey satellite (`satellites.rs`): the assembler builds it, a launch site's rocket carries it into orbit.
pub const SURVEY_SATELLITE: ItemId = ItemId(391);
/// A comms satellite: data grids and drone ports reach across its footprint (`factory/fibre.rs`, `factory/ports.rs`).
pub const COMMS_SATELLITE: ItemId = ItemId(392);
/// A power satellite: it beams power down to rectennas under its footprint (`factory/process/rectenna.rs`).
pub const POWER_SATELLITE: ItemId = ItemId(393);
/// A science satellite: a ground station under its footprint downlinks its data as compute (`factory/process/ground.rs`).
pub const SCIENCE_SATELLITE: ItemId = ItemId(394);
/// A drop pod (the Drop Pods tech): a pod launcher spends one to send a load of cargo to a landing pad (`pods.rs`).
pub const DROP_POD: ItemId = ItemId(395);
/// Salvage from the ruins' rusted blocks (`sim/scrap.rs`); a smelter melts it into iron (`recipes/scrap.rs`).
pub const SCRAP_METAL: ItemId = ItemId(396);
/// The telescope (`telescope.rs`): look through it to pin faraway ground and see every pin as a column of light.
pub const TELESCOPE: ItemId = ItemId(397);
/// The Bladerang, once the sickle (`bladerang.rs`): thrown, it cuts the ground plants along its loop
/// (`world/cover_clear.rs`) and brings back loose items.
pub const BLADERANG: ItemId = ItemId(398);
/// The forester Mk2 and Mk3 (`factory/process/forester.rs`): faster, and a bigger patch where the ground allows.
pub const FORESTER_MK2: ItemId = ItemId(399);
pub const FORESTER_MK3: ItemId = ItemId(400);
/// Ids 401 to 409 are the woods' logs and the bamboo cane (`FIRST_WOOD_LOG`, `item.rs`): new items go after them, their
/// rows after the bamboo's. The storage box's Mk4 and Mk5 and the pumpjack's Mk2 (`factory/tiers.rs`).
pub const BOX_MK4: ItemId = ItemId(410);
pub const BOX_MK5: ItemId = ItemId(411);
pub const PUMPJACK_MK2: ItemId = ItemId(412);
/// The X-ray goggles (worn in the Head slot, `equipment.rs`): while on, what is built shows through the ground.
pub const XRAY_GOGGLES: ItemId = ItemId(413);
/// The five ground pigments (`recipes/pigment.rs`): a mineral ground fine, by hand. Concrete tints and roof tiles take them
/// in place of the raw mineral; the Pigments tech mixes more hues from them (`docs/ROADMAP_CHEMISTRY.md` DYE).
pub const PIGMENT_RED: ItemId = ItemId(414);
pub const PIGMENT_GREEN: ItemId = ItemId(415);
pub const PIGMENT_BLACK: ItemId = ItemId(416);
pub const PIGMENT_WHITE: ItemId = ItemId(417);
pub const PIGMENT_YELLOW: ItemId = ItemId(418);
/// The eleven pigments of the Pigments tech (`recipes/dye.rs`), in `HUES` order: blue from copper and acid in a chemical plant,
/// then ten mixes of two others. Ids 419 to 429.
pub const PIGMENT_BLUE: ItemId = ItemId(419);
pub const PIGMENT_ORANGE: ItemId = ItemId(420);
pub const PIGMENT_PURPLE: ItemId = ItemId(421);
pub const PIGMENT_CYAN: ItemId = ItemId(422);
pub const PIGMENT_LIME: ItemId = ItemId(423);
pub const PIGMENT_PINK: ItemId = ItemId(424);
pub const PIGMENT_SKY: ItemId = ItemId(425);
pub const PIGMENT_GREY: ItemId = ItemId(426);
pub const PIGMENT_BROWN: ItemId = ItemId(427);
pub const PIGMENT_MAROON: ItemId = ItemId(428);
pub const PIGMENT_NAVY: ItemId = ItemId(429);
/// Natural gas barrels (CHEM-GAS): raw gas drawn from gas shale or associated oil wells, sweetened and fractionated
/// into methane, ethane and LPG, and cracked into ethylene for polyethylene. Ids 430 to 434.
pub const RAW_GAS_BARREL: ItemId = ItemId(430);
pub const METHANE_BARREL: ItemId = ItemId(431);
pub const ETHANE_BARREL: ItemId = ItemId(432);
pub const LPG_BARREL: ItemId = ItemId(433);
pub const ETHYLENE_BARREL: ItemId = ItemId(434);
/// The five natural gas machines' items (wide blocks 256–260, paired in `item::WIDE_BLOCK_ITEMS`). Ids 435 to 439.
pub const GAS_WELL_ITEM: ItemId = ItemId(435);
pub const WELLHEAD_SEPARATOR_ITEM: ItemId = ItemId(436);
pub const GAS_PLANT_ITEM: ItemId = ItemId(437);
pub const GAS_TURBINE_ITEM: ItemId = ItemId(438);
pub const STEAM_REFORMER_ITEM: ItemId = ItemId(439);
/// Charcoal (FIRE1): left behind when a grown tree burns down; serves as coal everywhere (`wood.rs`).
pub const CHARCOAL: ItemId = ItemId(440);
/// The flamethrower (FIRE2, `flame_hands.rs`): weed-torches ground plants and sets grown trees ablaze; runs on naphtha or diesel barrels.
pub const FLAMETHROWER: ItemId = ItemId(441);
/// The Seismic Thumper (F-SEIS, `factory/process/thumper.rs`): Mk1 (wide block 261) plus Mk2 and Mk3 tiers.
pub const THUMPER_ITEM: ItemId = ItemId(442);
pub const THUMPER_MK2: ItemId = ItemId(443);
pub const THUMPER_MK3: ItemId = ItemId(444);
/// Glass Kiln (L-COLOUR, wide block 262, `factory/process/kiln.rs`): melts silica + pigment into stained glass.
pub const GLASS_KILN_ITEM: ItemId = ItemId(445);
/// Handheld Paint Sprayer (L-COLOUR, `colour_hands.rs`): sprays paintable building surfaces in place using pigments.
pub const PAINT_SPRAYER: ItemId = ItemId(446);
/// Work Gloves Mk1–Mk4 (`equipment.rs`, `GLOVES_TIERS`; Mk5 is `CRAFT_GLOVES`).
pub const WORK_GLOVES: ItemId = ItemId(447);
pub const WORK_GLOVES_MK2: ItemId = ItemId(448);
pub const WORK_GLOVES_MK3: ItemId = ItemId(449);
pub const WORK_GLOVES_MK4: ItemId = ItemId(450);
/// Clay and kilns (MAT-CLAY, wide blocks 263–269, `block/clay.rs`): the two soft ores, then the five ceramics.
pub const CLAY_ITEM: ItemId = ItemId(451);
pub const KAOLIN_ITEM: ItemId = ItemId(452);
pub const CLAY_BRICKS_ITEM: ItemId = ItemId(453);
pub const TERRACOTTA_ITEM: ItemId = ItemId(454);
pub const PORCELAIN_ITEM: ItemId = ItemId(455);
pub const TURQUOISE_TILES_ITEM: ItemId = ItemId(456);
pub const AMBER_TILES_ITEM: ItemId = ItemId(457);
/// The Tool Belt Mk1–Mk5 (GEAR-BELT, `tool_belt/`): worn in the Tool belt slot, it replaced the nine hand tools.
pub const TOOL_BELT: ItemId = ItemId(458);
pub const TOOL_BELT_MK2: ItemId = ItemId(459);
pub const TOOL_BELT_MK3: ItemId = ItemId(460);
pub const TOOL_BELT_MK4: ItemId = ItemId(461);
pub const TOOL_BELT_MK5: ItemId = ItemId(462);
/// The Elevator Anchor as an item (M14A; it places `block::ELEVATOR_ANCHOR`).
pub const ELEVATOR_ANCHOR_ITEM: ItemId = ItemId(463);
/// The Weather Station as an item (ATMOS; it places `block::WEATHER_STATION`).
pub const WEATHER_STATION_ITEM: ItemId = ItemId(464);
/// The Site Grader as an item (it places `block::SITE_GRADER`).
pub const SITE_GRADER_ITEM: ItemId = ItemId(465);
/// The Headlamp (GEAR-SLOTS): worn in the Head slot, it lights the ground like a lamp in hand (`held_light.rs`).
pub const HEADLAMP: ItemId = ItemId(466);
/// A coil of rope (GEAR-ROPE): the item of the wide block `block::ROPE`; one coil hangs `rope::PER_COIL` cells.
pub const ROPE_ITEM: ItemId = ItemId(467);
/// The Line Hook (GEAR-HOOK): a left-hand device, `G` pulls the body to a block or rides power wires (`line_hook.rs`).
pub const LINE_HOOK: ItemId = ItemId(468);
