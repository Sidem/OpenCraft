"""Surface-hint variants of the realistic ground tiles (rusty, dark, green, pale = iron, coal, copper, limestone/quartz):
the plain recipe in `real` with a stain from `stains` laid over it, so a hint is the same ground, just marked."""
import real
import stains
from lib import rgba


def _stained(tile, kind, grass=False):
    return rgba(stains.apply(tile[..., :3], kind, grass))


def soil(kind):
    return _stained(real.dirt(), kind)


def grass_top(kind):
    return _stained(real.grass_top(), kind, grass=True)


def grass_side(kind):
    return _stained(real.grass_side(), kind, grass=True)


def sand(kind):
    return _stained(real.sand(), kind)
