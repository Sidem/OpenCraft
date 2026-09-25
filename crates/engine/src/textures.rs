//! Procedural 16×16 block and item textures, generated at startup so the MVP ships with zero art assets.
//! Every pattern tiles seamlessly (noise lattices wrap at the texture edge). `generate` writes one
//! RGBA layer per `block::tex` constant; to add a texture, add the constant there and its pattern
//! arm in `pixel`. Machine and item patterns live in `textures/machines.rs`.

use crate::block::tex;
use crate::math::{hash3, unit};

mod machines;

use machines::{
    belt_side, belt_top, constructor, crate_wood, drill, flask, generator, ingot, lab, lamp, miner_side, miner_top,
    plate, pole, router_top, smelter, wire,
};

pub const TEX_SIZE: usize = 16;

pub fn generate() -> Vec<u8> {
    let mut out = vec![0u8; tex::COUNT * TEX_SIZE * TEX_SIZE * 4];
    for layer in 0..tex::COUNT {
        for y in 0..TEX_SIZE {
            for x in 0..TEX_SIZE {
                let i = ((layer * TEX_SIZE + y) * TEX_SIZE + x) * 4;
                out[i..i + 4].copy_from_slice(&pixel(layer as u16, x as i32, y as i32));
            }
        }
    }
    out
}

/// Per-pixel white noise in [0, 1).
fn n(seed: u32, x: i32, y: i32) -> f64 {
    unit(hash3(seed, x.rem_euclid(16), y.rem_euclid(16), 0))
}

/// Tileable value noise on a `cells`×`cells` lattice.
fn smooth(seed: u32, x: i32, y: i32, cells: i32) -> f64 {
    let s = 16.0 / cells as f64;
    let (fx, fy) = (x as f64 / s, y as f64 / s);
    let (x0, y0) = (fx.floor() as i32, fy.floor() as i32);
    let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
    let (tx, ty) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let v = |ix: i32, iy: i32| unit(hash3(seed, ix.rem_euclid(cells), iy.rem_euclid(cells), 1));
    let a = v(x0, y0) + (v(x0 + 1, y0) - v(x0, y0)) * tx;
    let b = v(x0, y0 + 1) + (v(x0 + 1, y0 + 1) - v(x0, y0 + 1)) * tx;
    a + (b - a) * ty
}

fn rgb(c: [f64; 3], k: f64) -> [u8; 4] {
    let ch = |v: f64| (v * k).round().clamp(0.0, 255.0) as u8;
    [ch(c[0]), ch(c[1]), ch(c[2]), 255]
}

fn stone(x: i32, y: i32) -> [u8; 4] {
    let k = 0.78 + 0.16 * n(1, x, y) + 0.14 * smooth(2, x, y, 4);
    rgb([122.0, 122.0, 126.0], k)
}

fn dirt(x: i32, y: i32) -> [u8; 4] {
    let speck = if n(4, x, y) < 0.12 { 0.72 } else { 1.0 };
    let k = (0.8 + 0.2 * n(3, x, y) + 0.1 * smooth(5, x, y, 4)) * speck;
    rgb([124.0, 88.0, 60.0], k)
}

fn grass(x: i32, y: i32) -> [u8; 4] {
    let k = 0.76 + 0.2 * n(6, x, y) + 0.14 * smooth(7, x, y, 4);
    rgb([96.0, 156.0, 56.0], k)
}

/// Stone with 5 tileable mineral clusters.
fn ore(x: i32, y: i32, seed: u32, main: [f64; 3], accent: [f64; 3]) -> [u8; 4] {
    for i in 0..5 {
        let h = hash3(seed, i, 0, 0);
        let (cx, cy) = ((h & 15) as i32, ((h >> 4) & 15) as i32);
        let wrap = |d: i32| d.abs().min(16 - d.abs());
        let (dx, dy) = (wrap(x - cx), wrap(y - cy));
        let r2 = 1 + ((h >> 8) % 3) as i32;
        if dx * dx + dy * dy <= r2 {
            let k = 0.8 + 0.3 * n(seed + 1, x, y);
            return if n(seed + 2, x, y) < 0.3 { rgb(accent, k) } else { rgb(main, k) };
        }
    }
    stone(x, y)
}

/// Worked-out rock: paler than stone, pitted, with rust stains left by the ore.
fn spent_rock(x: i32, y: i32) -> [u8; 4] {
    let k = 0.8 + 0.14 * n(40, x, y) + 0.12 * smooth(41, x, y, 4);
    if n(42, x, y) < 0.1 {
        rgb([96.0, 88.0, 82.0], k)
    } else if smooth(43, x, y, 8) > 0.72 && n(44, x, y) < 0.5 {
        rgb([158.0, 108.0, 76.0], k)
    } else {
        rgb([146.0, 138.0, 128.0], k)
    }
}

/// Brushed steel with a dark rim and corner rivets.
fn frame(x: i32, y: i32) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let rivet = (x == 2 || x == 13) && (y == 2 || y == 13);
    let k = 0.9 + 0.1 * n(45, x, y / 4);
    if rivet {
        rgb([182.0, 188.0, 196.0], 1.0)
    } else if edge {
        rgb([78.0, 82.0, 90.0], k)
    } else {
        rgb([120.0, 126.0, 134.0], k)
    }
}

fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::STONE => stone(x, y),
        tex::DIRT => dirt(x, y),
        tex::GRASS_TOP => grass(x, y),
        tex::GRASS_SIDE => {
            let depth = 3 + (hash3(8, x, 0, 0) % 3) as i32;
            if y < depth || (y == depth && n(9, x, y) < 0.4) {
                let [r, g, b, a] = grass(x, y);
                let k = if y + 1 >= depth { 0.85 } else { 1.0 };
                [(r as f64 * k) as u8, (g as f64 * k) as u8, (b as f64 * k) as u8, a]
            } else {
                dirt(x, y)
            }
        }
        tex::SAND => rgb([222.0, 206.0, 152.0], 0.9 + 0.08 * n(10, x, y) + 0.06 * smooth(11, x, y, 4)),
        tex::LOG_SIDE => {
            let groove = hash3(12, x, 0, 0).is_multiple_of(4);
            let k = 0.74 + 0.18 * n(13, x, 0) + 0.1 * n(14, x, y / 3) - if groove { 0.18 } else { 0.0 };
            rgb([112.0, 84.0, 52.0], k)
        }
        tex::LOG_TOP => {
            let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 7.0 {
                rgb([112.0, 84.0, 52.0], 0.8 + 0.15 * n(15, x, y))
            } else {
                let ring = ((d * 1.15 + 0.35 * n(16, x, y)) as i32) % 2 == 0;
                rgb(if ring { [186.0, 150.0, 96.0] } else { [160.0, 124.0, 76.0] }, 0.94 + 0.08 * n(17, x, y))
            }
        }
        tex::LEAVES => {
            let base = [64.0, 124.0, 44.0];
            let mut c = rgb(base, 0.62 + 0.42 * n(18, x, y) + 0.12 * smooth(19, x, y, 4));
            if n(20, x, y) < 0.2 {
                // Transparent, but keep a leaf colour so mipmaps don't bleed dark fringes.
                c[3] = 0;
            }
            c
        }
        tex::COAL_ORE => ore(x, y, 21, [40.0, 40.0, 44.0], [74.0, 74.0, 82.0]),
        tex::IRON_ORE => ore(x, y, 24, [218.0, 180.0, 152.0], [168.0, 112.0, 86.0]),
        tex::COPPER_ORE => ore(x, y, 27, [226.0, 134.0, 72.0], [88.0, 172.0, 140.0]),
        tex::BEDROCK => {
            let k = 0.3 + 0.6 * n(30, x, y) * (0.6 + 0.4 * smooth(31, x, y, 4));
            rgb([130.0, 130.0, 136.0], k)
        }
        tex::SPENT_ROCK => spent_rock(x, y),
        tex::BELT_TOP => belt_top(x, y, [176.0, 150.0, 92.0]),
        tex::FAST_BELT_TOP => belt_top(x, y, [96.0, 170.0, 236.0]),
        tex::FRAME => frame(x, y),
        tex::MINER_SIDE => miner_side(x, y, [232.0, 146.0, 60.0]),
        tex::MINER_MK2_SIDE => miner_side(x, y, [96.0, 170.0, 236.0]),
        tex::MINER_TOP => miner_top(x, y),
        tex::DRILL => drill(x, y),
        tex::BOX_SIDE => crate_wood(x, y, false),
        tex::BOX_TOP => crate_wood(x, y, true),
        tex::LAMP_GREEN => lamp(x, y, [96.0, 214.0, 112.0]),
        tex::LAMP_YELLOW => lamp(x, y, [236.0, 190.0, 64.0]),
        tex::LAMP_RED => lamp(x, y, [226.0, 72.0, 60.0]),
        tex::IRON_INGOT => ingot(x, y, [176.0, 180.0, 188.0]),
        tex::COPPER_INGOT => ingot(x, y, [206.0, 118.0, 70.0]),
        tex::SMELTER_SIDE => smelter(x, y, true),
        tex::SMELTER_TOP => smelter(x, y, false),
        tex::IRON_PLATE => plate(x, y),
        tex::COPPER_WIRE => wire(x, y),
        tex::CONSTRUCTOR_SIDE => constructor(x, y, true),
        tex::CONSTRUCTOR_TOP => constructor(x, y, false),
        tex::SPLITTER_TOP => router_top(x, y, false),
        tex::FILTER_TOP => router_top(x, y, true),
        tex::RAMP_UP_SIDE..=tex::UNDERPASS_OUT_SIDE => belt_side(x, y, (layer - tex::RAMP_UP_SIDE) as u8),
        tex::GENERATOR_SIDE => generator(x, y, false),
        tex::GENERATOR_TOP => generator(x, y, true),
        tex::POLE_SIDE => pole(x, y),
        tex::LAB_SIDE => lab(x, y, false),
        tex::LAB_TOP => lab(x, y, true),
        tex::RED_PACK => flask(x, y, [214.0, 60.0, 56.0]),
        tex::GREEN_PACK => flask(x, y, [72.0, 190.0, 88.0]),
        _ => [255, 0, 255, 255],
    }
}

#[cfg(test)]
mod tests;
