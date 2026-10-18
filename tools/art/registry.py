"""The layer table: which engine layer each recipe paints. Add a recipe by appending a row (engine layer ids are in
`crates/engine/src/block/tex.rs`; alternates of ores and leaves are 73 + 3 * i + 0..2 in `WITH_ALTERNATES` order:
coal 73-75, iron 76-78, copper 79-81, limestone 82-84, quartz 85-87, leaves 88-90).

`tiles`: the tile repeats over greedy-merged faces, so `check.py` demands a seamless wrap. `cutout`: alpha is coverage
(leaves, glass); otherwise alpha carries `255 - glow`.
"""
from dataclasses import dataclass
from typing import Callable

import forest
import hints as hint_tiles
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

LAYERS += NATURAL
