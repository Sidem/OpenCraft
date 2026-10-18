"""Surface-hint variants of the plain ground tiles (rusty, dark, green, pale = iron, coal, copper, limestone/quartz): the
plain recipe in `natural` with a stain from `stains` laid over it, so a hint is the same ground, just marked."""
import natural
import stains


def _post(kind):
    return lambda img, base: stains.apply(img, base, kind)


def soil(kind):
    return natural.dirt(0, _post(kind))


def grass_top(kind):
    return natural.grass_top(0, _post(kind))


def grass_side(kind):
    return natural.grass_side(0, _post(kind))


def sand(kind):
    return natural.sand(0, _post(kind))
