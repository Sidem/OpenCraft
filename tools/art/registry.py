"""The layer table: which engine layer each recipe paints. Add a recipe by appending a row (engine layer ids are in
`crates/engine/src/block/tex.rs`; alternates of ores and leaves are 73 + 3 * i + 0..2 in `WITH_ALTERNATES` order:
coal 73-75, iron 76-78, copper 79-81, limestone 82-84, quartz 85-87, leaves 88-90).

`tiles`: the tile repeats over greedy-merged faces, so `check.py` demands a seamless wrap. `cutout`: alpha is coverage
(leaves, glass); otherwise alpha carries `255 - glow`.
"""
from dataclasses import dataclass
from typing import Callable

import machines
import masonry
import natural
import ores


@dataclass
class Entry:
    layer: int
    name: str
    make: Callable
    tiles: bool = False
    cutout: bool = False


def alts(base, first, name, make, **kw):
    """The base layer and its three alternates (`make(alt)` with alt 0..3)."""
    layers = [(base, name, 0)] + [(first + i, f"{name}_alt{i + 1}", i + 1) for i in range(3)]
    return [Entry(layer, n, lambda a=a: make(a), **kw) for layer, n, a in layers]


PILOT = [
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
