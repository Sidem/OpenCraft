"""The layer table: which engine layer each recipe paints. Add a recipe by appending a row (engine layer ids are in
`crates/engine/src/block/tex.rs`; alternates of ores and leaves are 73 + 3 * i + 0..2 in `WITH_ALTERNATES` order:
coal 73-75, iron 76-78, copper 79-81, limestone 82-84, quartz 85-87, leaves 88-90).

`tiles`: the tile repeats over greedy-merged faces, so `check.py` demands a seamless wrap. `cutout`: alpha is coverage
(leaves, glass); otherwise alpha carries `255 - glow`. `real`: the realistic look (ART_STYLE.md section 12).
"""
from dataclasses import dataclass
from typing import Callable

import decor
import forest
import later
import heavy
import process
import hints as hint_tiles
import icons
import icons2
import industry
import items
import kit
import machines
import masonry
import natural
import ores
import plants
import real
import rocks
import stains
from lib import ramp


@dataclass
class Entry:
    layer: int
    name: str
    make: Callable
    tiles: bool = False
    cutout: bool = False
    cover: tuple = None  # cutouts: the allowed share of covered texels (default: leaves 0.55-0.85)
    real: bool = False  # painted in the realistic look (continuous tone): judged by `realism.py`, not the colour cap


def alts(base, first, name, make, **kw):
    """The base layer and its three alternates (`make(alt)` with alt 0..3)."""
    layers = [(base, name, 0)] + [(first + i, f"{name}_alt{i + 1}", i + 1) for i in range(3)]
    return [Entry(layer, n, lambda a=a: make(a), **kw) for layer, n, a in layers]


def hints(first, make, **kw):
    """Four hint layers from `first`: rusty, dark, green, pale (the order of `block::tex`)."""
    return [Entry(first + i, f"{make.__name__}_{k}", lambda k=k: make(k), **kw) for i, k in enumerate(stains.KINDS)]


PLANTS = [(306, "tuft"), (307, "daisy"), (308, "poppy"), (309, "bluebell"), (310, "fern"), (311, "reed"), (312, "dry_brush"),
          (313, "lichen"), (57, "sapling"), (320, "birch_sapling"), (321, "pine_sapling"), (322, "rust_bloom"),
          (323, "coal_shrub"), (324, "copper_bell"), (325, "pale_star"), (108, "torch")]

NATURAL = [
    Entry(11, "bedrock", rocks.bedrock, tiles=True),
    Entry(12, "spent_rock", rocks.spent_rock, tiles=True),
    Entry(58, "granite", rocks.granite, tiles=True),
    Entry(59, "sandstone", rocks.sandstone, tiles=True),
    Entry(60, "basalt", rocks.basalt, tiles=True),
    *alts(61, 82, "limestone", lambda a: rocks.limestone(a * 5), tiles=True),
    *alts(62, 85, "quartz_ore", rocks.quartz_ore, real=True),
    *hints(64, hint_tiles.soil, tiles=True, real=True),
    *hints(91, hint_tiles.grass_top, tiles=True, real=True),
    *hints(95, hint_tiles.grass_side, real=True),
    *hints(99, hint_tiles.sand, tiles=True, real=True),
    Entry(106, "water", forest.water, tiles=True),
    Entry(314, "birch_bark", forest.birch_bark, tiles=True),
    Entry(315, "birch_leaves", forest.birch_leaves, cutout=True),
    Entry(316, "pine_leaves", forest.pine_leaves, cutout=True, cover=(0.45, 0.85)),
    Entry(317, "cactus_side", forest.cactus_side, tiles=True),
    Entry(318, "cactus_top", forest.cactus_top),
    Entry(319, "dead_wood", forest.dead_wood, tiles=True),
    *[Entry(layer, name, getattr(plants, name), cutout=True, cover=(0.02, 0.5)) for layer, name in PLANTS],
]

LAYERS = [
    Entry(0, "stone", real.stone, tiles=True, real=True),
    Entry(1, "dirt", real.dirt, tiles=True, real=True),
    Entry(2, "grass_top", real.grass_top, tiles=True, real=True),
    Entry(3, "grass_side", real.grass_side, real=True),
    Entry(4, "sand", real.sand, tiles=True, real=True),
    Entry(5, "log_side", natural.bark, tiles=True),
    Entry(6, "log_top", natural.log_end),
    Entry(109, "planks", natural.planks, tiles=True),
    Entry(119, "stone_bricks", masonry.bricks, tiles=True),
    *alts(7, 88, "leaves", lambda a: natural.leaves(a), cutout=True),
    *alts(8, 73, "coal_ore", ores.coal, real=True),  # realistic stone body; the deposits are redone in batch 2
    *alts(9, 76, "iron_ore", ores.iron, real=True),
    *alts(10, 79, "copper_ore", ores.copper, real=True),
    Entry(63, "glass", machines.glass, cutout=True),
    Entry(105, "lamp", machines.lamp),
    Entry(14, "frame", machines.frame),
    Entry(104, "steel", machines.steel),
    Entry(25, "smelter_side", machines.smelter_side),
    Entry(26, "smelter_top", machines.smelter_top),
    Entry(38, "generator_side", machines.generator_side),
    Entry(39, "generator_top", machines.generator_top),
]

INDUSTRY = [
    *[Entry(layer, f"belt_mk{i + 1}", lambda i=i: industry.belt(i), tiles=True) for i, layer in enumerate((13, 46, 139, 156))],
    *[Entry(layer, f"miner_mk{i + 1}", lambda i=i: industry.miner_side(i)) for i, layer in enumerate((15, 45, 140, 157, 242))],
    Entry(16, "miner_top", industry.miner_top),
    Entry(17, "drill", industry.drill, tiles=True),
    Entry(18, "box_side", industry.box_side),
    Entry(19, "box_top", industry.box_top),
    Entry(29, "constructor_side", industry.constructor_side),
    Entry(30, "constructor_top", industry.constructor_top),
    Entry(31, "splitter_top", industry.splitter_top),
    Entry(32, "filter_top", industry.filter_top),
    Entry(40, "pole_side", industry.pole_side, cutout=True, cover=(0.02, 0.4)),
    Entry(41, "lab_side", industry.lab_side),
    Entry(42, "lab_top", industry.lab_top),
    Entry(121, "assembler_side", industry.assembler_side),
    Entry(122, "assembler_top", industry.assembler_top),
    Entry(123, "port_in", industry.port_in),
    Entry(124, "port_out", industry.port_out),
    Entry(125, "concrete", industry.concrete, tiles=True),
    Entry(126, "motor", industry.motor_side),
    Entry(127, "blast_side", industry.blast_side),
    Entry(128, "blast_top", industry.blast_top),
    Entry(129, "port_side", industry.port_side),
]

HEAVY = [Entry(layer, name, make) for layer, name, make in (
    (33, "ramp_up_side", lambda: heavy.ramp_side(True)), (34, "ramp_down_side", lambda: heavy.ramp_side(False)),
    (35, "lift_side", heavy.lift_side), (36, "underpass_in", lambda: heavy.underpass(True)),
    (37, "underpass_out", lambda: heavy.underpass(False)), (178, "pipe_water", heavy.pipe_water),
    (179, "pipe_steam", heavy.pipe_steam), (196, "dock_side", heavy.dock_side),
    (197, "dock_load_top", lambda: heavy.dock_top(True)), (198, "dock_unload_top", lambda: heavy.dock_top(False)),
    (199, "signal", heavy.signal), (141, "boiler_side", heavy.boiler_side), (142, "boiler_top", heavy.boiler_top),
    (143, "turbine_side", heavy.turbine_side), (144, "turbine_top", heavy.turbine_top),
    (145, "crusher_side", heavy.crusher_side), (146, "crusher_top", heavy.crusher_top),
    (147, "silo_side", heavy.silo_side), (148, "silo_top", heavy.silo_top), (151, "arc_side", heavy.arc_side),
    (152, "arc_top", heavy.arc_top), (158, "solar_top", heavy.solar_top), (159, "solar_side", heavy.solar_side),
    (160, "accumulator_top", heavy.accumulator_top), (161, "accumulator_side", heavy.accumulator_side),
    (191, "cell_side", process.cell_side), (192, "cell_top", process.cell_top), (206, "pumpjack_side", process.pumpjack_side),
    (207, "pumpjack_top", process.pumpjack_top), (212, "refinery_side", process.refinery_side),
    (213, "refinery_top", process.refinery_top), (214, "cracker_side", process.cracker_side),
    (215, "cracker_top", process.cracker_top), (219, "chem_side", process.chem_side), (220, "chem_top", process.chem_top),
    (223, "diesel_side", process.diesel_side), (224, "diesel_top", process.diesel_top),
    (225, "electrolyser_side", process.electrolyser_side), (226, "electrolyser_top", process.electrolyser_top),
    (231, "washer_side", process.washer_side), (232, "washer_top", process.washer_top), (235, "winch_side", process.winch_side),
    (237, "centrifuge_side", process.centrifuge_side), (238, "centrifuge_top", process.centrifuge_top),
    (239, "reactor_side", process.reactor_side), (240, "reactor_top", process.reactor_top),
    (244, "recycler_side", process.recycler_side), (245, "recycler_top", process.recycler_top),
    (249, "fab_side", process.fab_side), (250, "fab_top", process.fab_top))] + [
    Entry(233, "hoist_frame", process.hoist_frame, cutout=True, cover=(0.02, 0.5)),
    Entry(234, "hoist_top", process.hoist_top, cutout=True, cover=(0.02, 0.7)),
]

LATER = [Entry(251 + i, f"grid_{i}", lambda i=i: later.grid_colour(i)) for i in range(6)] + [Entry(layer, name, make) for layer, name, make in (
    (257, "node_side", later.node_side), (258, "node_top", later.node_top), (259, "fibre", later.fibre),
    (260, "dc_side", later.dc_side), (261, "dc_top", later.dc_top), (262, "tower_side", later.tower_side),
    (263, "tower_top", later.tower_top), (264, "ai_side", later.ai_side), (265, "ai_top", later.ai_top),
    (266, "opt_side", later.opt_side), (267, "opt_top", later.opt_top),
    (268, "emit_side", lambda: later.laser_side(False)), (269, "emit_top", lambda: later.laser_top(False)),
    (270, "recv_side", lambda: later.laser_side(True)), (271, "recv_top", lambda: later.laser_top(True)),
    (272, "beam", later.beam), (276, "mirror", later.mirror), (277, "data_side", later.data_side),
    (278, "data_top", later.data_top), (279, "hub_side", later.hub_side), (280, "hub_top", later.hub_top),
    (286, "launch_side", later.launch_side), (287, "launch_top", later.launch_top), (288, "light_frame", later.light_frame),
    (293, "rocket_hull", later.rocket_hull), (297, "rectenna_top", later.rectenna_top),
    (298, "rectenna_side", later.rectenna_side), (300, "ground_top", later.ground_top),
    (301, "ground_side", later.ground_side), (303, "pod_side", later.pod_side),
    (304, "pod_launch_top", later.pod_launch_top), (305, "pod_pad_top", later.pod_pad_top),
    (283, "loco_body", later.loco_body), (284, "soot", later.soot),
    (326, "rust_plate", decor.rust_plate), (327, "rust_machine", decor.rust_machine), (328, "rust_girder", decor.rust_girder),
    (330, "logbook_side", decor.logbook_side), (331, "logbook_top", decor.logbook_top), (332, "vault_door", decor.vault_door),
    (346, "panel_light", decor.panel_light), (343, "metal_panel", decor.metal_panel))]

DECOR = [
    *[Entry(layer, name, lambda s=s, t=t: decor.coloured_concrete(t, s), tiles=True) for s, (layer, name, t) in enumerate((
        (333, "red_concrete", decor.RED_C), (334, "green_concrete", decor.GREEN_C), (335, "black_concrete", decor.BLACK),
        (336, "white_concrete", decor.WHITE), (337, "yellow_concrete", decor.YELLOW_C)))],
    Entry(338, "basalt_tiles", decor.basalt_tiles, tiles=True),
    Entry(339, "granite_bricks", lambda: decor.brick_face(rocks.GRANITE, 1), tiles=True),
    Entry(340, "sandstone_bricks", lambda: decor.brick_face(rocks.SANDSTONE, 2), tiles=True),
    Entry(341, "slag_bricks", lambda: decor.brick_face(decor.SLAG, 3), tiles=True),
    Entry(342, "framed_glass", decor.framed_glass, cutout=True, cover=(0.03, 0.5)),
    Entry(344, "catwalk", decor.catwalk, cutout=True, cover=(0.05, 0.5)),
    Entry(345, "railing", decor.railing, cutout=True, cover=(0.05, 0.5)),
]

_CRUSHED = ramp(["#10141c", "#2a303c", "#4a5262", "#707a8c", "#a0a8b8"])
_LIME = ramp(["#2a2638", "#6a6684", "#a8a4c0", "#dcdaec", "#ffffff"])
_SULFUR = ramp(["#3a2c06", "#8a6a0c", "#d0a818", "#f4de44", "#fffbb0"])
_PLASTIC = ramp(["#2a3248", "#6a7aa0", "#a8bcdc", "#dcecfa", "#ffffff"])
_WASHED = lambda *s: ramp(list(s))
_LIQUIDS = {
    "naphtha": ramp(["#4a4010", "#a89830", "#e8dc60", "#fff6a0"]), "heavy": ramp(["#1a0806", "#4a1a10", "#7a2c1a", "#a04a2c"]),
    "acid": ramp(["#102a06", "#3a8a10", "#8ae020", "#d0ff70"]), "hydrogen": ramp(["#0a2a4a", "#2a70c0", "#70b8f0", "#c8ecff"]),
    "oxygen": ramp(["#3a4a60", "#8aa0c0", "#d8e8f8", "#ffffff"]), "crude": ramp(["#050505", "#14100c", "#241c14", "#3a2c1c"]),
}

ITEMS = [Entry(layer, name, make, **kw) for layer, name, make, kw in (
    # status lights, tier stripes, lights and bands
    (20, "lamp_green", lambda: items.lamp_light(kit.GREEN, 20), {}), (21, "lamp_yellow", lambda: items.lamp_light(kit.GOLD, 21), {}),
    (22, "lamp_red", lambda: items.lamp_light(kit.RED, 22), {}), (107, "lamp_blue", lambda: items.lamp_light(kit.BLUE, 107), {}),
    *[(114 + i, f"stripe_{i + 1}", lambda i=i: items.stripe(i, 114 + i), {}) for i in range(5)],
    (68, "scanner", lambda: items.scanner(kit.GREEN, 68), {}), (166, "scanner_mk2", lambda: items.scanner(kit.VIOLET, 166), {}),
    (165, "strategy_fog", items.fog, dict(tiles=True)),
    # metals
    (23, "iron_ingot", lambda: items.ingot(icons.IRON_HEAD, 23), {}), (24, "copper_ingot", lambda: items.ingot(machines.COPPER, 24), {}),
    (27, "iron_plate", lambda: items.plate_item(icons.IRON_HEAD, 27), {}), (28, "copper_wire", items.wire, {}),
    (70, "iron_rod", items.rod, {}), (71, "screw", items.screw, {}), (113, "gear", icons.gear_icon, {}),
    (131, "steel_ingot", lambda: items.ingot(icons.STEEL_HEAD, 131), {}), (132, "steel_plate", lambda: items.plate_item(icons.STEEL_HEAD, 132), {}),
    (133, "steel_beam", items.beam, {}), (137, "steel_head", items.forged, {}), (130, "slag", items.slag, {}),
    (188, "aluminium_ingot", lambda: items.ingot(items.ALUMINIUM, 188), {}), (189, "aluminium_plate", lambda: items.plate_item(items.ALUMINIUM, 189), {}),
    (153, "silicon", items.silicon, {}), (69, "core_drill", items.core_drill, {}),
    # ores and what is made of them
    (186, "bauxite_ore", ores.bauxite, dict(tiles=True, real=True)), (202, "oil_sand", ores.oil_sand, dict(tiles=True, real=True)),
    (203, "uranium_ore", ores.uranium, dict(tiles=True, real=True)),
    (149, "crushed_iron", lambda: items.rubble(_CRUSHED, stains.RUST, 149), {}), (150, "crushed_copper", lambda: items.rubble(_CRUSHED, ores.MALACHITE, 150, fleck_frac=0.35), {}),
    (187, "crushed_bauxite", lambda: items.rubble(_CRUSHED, ores.BAUXITE, 187, fleck_frac=0.6), {}),
    (227, "washed_iron", lambda: items.rubble(_WASHED("#2a303c", "#6a7488", "#a8b2c6", "#e0e6f2"), stains.RUST, 227, 10, 0.12, 0.2), {}),
    (228, "washed_copper", lambda: items.rubble(_WASHED("#14241c", "#3a5a48", "#729a84", "#b8d6c4"), ores.MALACHITE, 228, 10, 0.4, 0.2), {}),
    (229, "washed_bauxite", lambda: items.rubble(_WASHED("#3a1c18", "#8a4a40", "#d08a78", "#ffcab8"), ores.BAUXITE, 229, 10, 0.3, 0.2), {}),
    (230, "tailings", lambda: items.rubble(_WASHED("#14100e", "#3a322a", "#6a5e4e", "#9a8c76"), _CRUSHED, 230, 14, 0.15), {}),
    (120, "quicklime", lambda: items.rubble(_LIME, _LIME, 120, 9, 0.0, 0.1, 0.05), {}),
    (211, "sulfur", lambda: items.rubble(_SULFUR, _SULFUR, 211, 10, 0.0, 0.25), {}), (216, "plastic", lambda: items.rubble(_PLASTIC, _PLASTIC, 216, 6, 0.0, 0.2, 0.06), {}),
    # science and glass
    (43, "red_pack", lambda: items.pack(kit.RED, 43), {}), (44, "green_pack", lambda: items.pack(kit.GREEN, 44), {}),
    (138, "blue_pack", lambda: items.pack(kit.BLUE, 138), {}), (155, "violet_pack", lambda: items.pack(kit.VIOLET, 155), {}),
    (241, "gold_pack", lambda: items.pack(kit.GOLD, 241), {}), (72, "flask_glass", items.flask_glass, {}),
    # wood, smoke, the avatar
    (103, "handle", items.handle, {}), (112, "stick", items.stick, {}),
    (110, "ladder", items.ladder_side, dict(cutout=True, cover=(0.1, 0.6))), (111, "ladder_top", items.ladder_top, dict(cutout=True, cover=(0.01, 0.2))),
    (281, "smoke", lambda: items.smoke(False), dict(cutout=True, cover=(0.2, 0.9))), (282, "smoke_thin", lambda: items.smoke(True), dict(cutout=True, cover=(0.05, 0.8))),
    (47, "avatar_suit", items.suit, {}), (48, "avatar_skin", items.skin, {}), (49, "avatar_helmet", items.helmet, {}),
    (50, "avatar_visor", items.visor, {}), (162, "avatar_joint", items.joint, {}), (163, "avatar_trim", items.trim, {}),
    (164, "avatar_pack", items.avatar_pack, {}),
    # tools
    *[(layer, f"{mat}_{kind}", lambda kind=kind, head=head, layer=layer: icons.tool(kind, head, layer), {})
      for mat, head, layers in (("stone", icons.STONE_HEAD, (51, 52, 53)), ("iron", icons.IRON_HEAD, (54, 55, 56)),
                                ("steel", icons.STEEL_HEAD, (134, 135, 136)))
      for layer, kind in zip(layers, ("pick", "axe", "shovel"))],
    # robot parts, gear, vehicles
    (154, "circuit", icons.circuit, {}), (167, "processor", lambda: icons.chip(167), {}), (168, "servo", icons.servo, {}),
    (169, "actuator", icons.actuator, {}), (170, "drone_cell", icons.drone_cell, {}), (171, "guidance", icons.guidance, {}),
    (172, "drone", lambda: icons.quad(172, machines.EMBER), {}), (173, "drone_port_top", icons2.port_top, {}),
    (174, "drone_port_side", icons2.port_side, {}), (175, "jetpack", icons.jetpack, {}), (176, "personal_drone", icons.personal_drone, {}),
    (177, "planner", icons.planner, {}),
    *[(layer, f"hauler_pack_{tier + 1}", lambda tier=tier, layer=layer: icons.hauler(tier, layer), {})
      for tier, layer in enumerate((180, 181, 273, 274, 275))],
    (182, "spring_boots", lambda: icons.boots(182, machines.EMBER, machines.AMBER), {}),
    (183, "servo_boots", lambda: icons.boots(183, machines.STEEL, machines.AETHER), {}),
    (184, "exo_frame", icons.exo_frame, {}), (185, "mining_rig", icons.mining_rig, {}), (285, "craft_gloves", icons.gloves, {}),
    (193, "rail", icons2.rail, {}), (194, "locomotive", icons2.loco, {}), (195, "wagon", icons2.wagon, {}),
    (200, "hover_pack", icons2.hover_pack, {}), (201, "cargo_drone", icons2.cargo_drone, {}),
    # canisters
    (204, "empty_canister", lambda: icons.canister(None, 204), {}),
    (205, "crude_canister", lambda: icons.canister(_LIQUIDS["crude"], 205, dark_fill=True), {}),
    (208, "naphtha_canister", lambda: icons.canister(_LIQUIDS["naphtha"], 208), {}),
    (209, "diesel_canister", lambda: icons.canister(machines.AMBER, 209), {}),
    (210, "heavy_oil_canister", lambda: icons.canister(_LIQUIDS["heavy"], 210), {}),
    (217, "acid_canister", lambda: icons.canister(_LIQUIDS["acid"], 217), {}),
    (218, "lubricant_canister", lambda: icons.canister(kit.GOLD, 218), {}),
    (221, "hydrogen_canister", lambda: icons.canister(_LIQUIDS["hydrogen"], 221), {}),
    (222, "oxygen_canister", lambda: icons.canister(_LIQUIDS["oxygen"], 222), {}),
    (246, "pure_water_canister", lambda: icons.canister(machines.AETHER, 246), {}),
    (292, "rocket_fuel_canister", lambda: icons.canister(machines.EMBER, 292, 0.95), {}),
    # small things, the rocket and the satellites
    (190, "battery", icons2.battery, {}), (236, "fuel_cell", icons2.fuel_cell, {}), (243, "coin", icons2.coin, {}),
    (247, "wafer", icons2.wafer, {}), (248, "accelerator", icons2.accelerator, {}), (329, "scrap", icons2.scrap, {}),
    (289, "rocket_engine", icons2.rocket_engine, {}), (290, "fuel_tank", icons2.fuel_tank, {}), (291, "avionics", icons2.avionics, {}),
    (294, "survey_satellite", lambda: icons2.satellite(294, kit.GOLD, machines.AMBER), {}),
    (295, "comms_satellite", icons2.comms_satellite, {}),
    (296, "power_satellite", lambda: icons2.satellite(296, kit.RED, machines.EMBER), {}),
    (299, "science_satellite", lambda: icons2.satellite(299, icons.WHITE, kit.GREEN), {}),
    (302, "drop_pod", icons2.drop_pod, {}),
)]

LAYERS += NATURAL + INDUSTRY + HEAVY + LATER + DECOR + ITEMS
