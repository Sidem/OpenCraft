//! Procedural 16×16 block and item textures, generated at startup so the MVP ships with zero art assets.
//! Every pattern tiles seamlessly (noise lattices wrap at the texture edge). `generate` writes one
//! RGBA layer per `block::tex` constant; to add a texture, add the constant there and its pattern
//! arm in `pixel`. Natural patterns live in `textures/nature.rs`; machines/items in `textures/machines.rs`.

use crate::block::tex;
use crate::math::{hash3, unit};

mod geology;
mod machines;
mod plants;
mod nature;
mod tools;

use nature::stone;

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

/// Other players (avatars.rs): 0 a blue work suit with a belt, 1 skin, 2 a yellow hard hat, 3 a dark
/// visor with a glint.
fn avatar(x: i32, y: i32, part: u16) -> [u8; 4] {
    let k = 0.86 + 0.1 * n(60 + part as u32, x, y);
    match part {
        0 if y == 9 || y == 10 => rgb([62.0, 52.0, 44.0], k),
        0 => rgb([58.0, 96.0, 168.0], k),
        1 => rgb([214.0, 170.0, 132.0], k + 0.06),
        2 => rgb([236.0, 190.0, 52.0], k + if x + y < 8 { 0.1 } else { 0.0 }),
        _ if (3..6).contains(&x) && y < 6 => rgb([168.0, 196.0, 220.0], 1.0),
        _ => rgb([34.0, 40.0, 52.0], k),
    }
}

fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::STONE..=tex::LEAVES | tex::BEDROCK | tex::SPENT_ROCK => nature::pixel(layer, x, y),
        tex::COAL_ORE => ore(x, y, 21, [40.0, 40.0, 44.0], [74.0, 74.0, 82.0]),
        tex::IRON_ORE => ore(x, y, 24, [218.0, 180.0, 152.0], [168.0, 112.0, 86.0]),
        tex::COPPER_ORE => ore(x, y, 27, [226.0, 134.0, 72.0], [88.0, 172.0, 140.0]),
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
        tex::AVATAR_SUIT..=tex::AVATAR_VISOR => avatar(x, y, layer - tex::AVATAR_SUIT),
        tex::STONE_PICKAXE..=tex::IRON_SHOVEL => {
            let i = layer - tex::STONE_PICKAXE;
            tools::tool(x, y, i % 3, i >= 3)
        }
        tex::SAPLING => plants::sapling(x, y),
        tex::GRANITE => geology::granite(x, y),
        tex::SANDSTONE => geology::sandstone(x, y),
        tex::BASALT => geology::basalt(x, y),
        tex::LIMESTONE => geology::limestone(x, y),
        tex::QUARTZ_ORE => ore(x, y, 80, [236.0, 232.0, 240.0], [196.0, 190.0, 214.0]),
        tex::GLASS => geology::glass(x, y),
        tex::RUSTY_SOIL => geology::soil(x, y, [176.0, 84.0, 40.0]),
        tex::DARK_SOIL => geology::soil(x, y, [40.0, 34.0, 30.0]),
        tex::GREEN_SOIL => geology::soil(x, y, [80.0, 150.0, 120.0]),
        tex::PALE_SOIL => geology::soil(x, y, [226.0, 218.0, 196.0]),
        tex::SCANNER => tools::scanner(x, y),
        tex::CORE_DRILL => tools::core_drill(x, y),
        _ => [255, 0, 255, 255],
    }
}

#[cfg(test)]
mod tests;
