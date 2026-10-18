"""The layer table: which engine layer each recipe paints. Add a recipe by appending a row (engine layer ids are in
`crates/engine/src/block/tex.rs`; alternates of ores and leaves are 73 + 3 * i + 0..2 in `WITH_ALTERNATES` order:
coal 73-75, iron 76-78, copper 79-81, limestone 82-84, quartz 85-87, leaves 88-90).

`tiles`: the tile repeats over greedy-merged faces, so `check.py` demands a seamless wrap. `cutout`: alpha is coverage
(leaves, glass); otherwise alpha carries `255 - glow`.
"""
from dataclasses import dataclass
from typing import Callable

import decor
import forest
import later
import heavy
import process
import hints as hint_tiles
import industry
import machines
import masonry
import natural
import ores
import plants
import rocks
import stains


@dataclass
class Entry:
    layer: int
    name: str
    make: Callable
    tiles: bool = False
    cutout: bool = False
    cover: tuple = None  # cutouts: the allowed share of covered texels (default: leaves 0.55-0.85)


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
    *alts(62, 85, "quartz_ore", rocks.quartz_ore),
    *hints(64, hint_tiles.soil, tiles=True),
    *hints(91, hint_tiles.grass_top, tiles=True),
    *hints(95, hint_tiles.grass_side),
    *hints(99, hint_tiles.sand, tiles=True),
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
    Entry(0, "stone", natural.stone, tiles=True),
    Entry(1, "dirt", natural.dirt, tiles=True),
    Entry(2, "grass_top", natural.grass_top, tiles=True),
    Entry(3, "grass_side", natural.grass_side),
    Entry(4, "sand", natural.sand, tiles=True),
    Entry(5, "log_side", natural.bark, tiles=True),
    Entry(6, "log_top", natural.log_end),
    Entry(109, "planks", natural.planks, tiles=True),
    Entry(119, "stone_bricks", masonry.bricks, tiles=True),
    *alts(7, 88, "leaves", lambda a: natural.leaves(a), cutout=True),
    *alts(8, 73, "coal_ore", ores.coal),
    *alts(9, 76, "iron_ore", ores.iron),
    *alts(10, 79, "copper_ore", ores.copper),
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

LAYERS += NATURAL + INDUSTRY + HEAVY + LATER + DECOR
