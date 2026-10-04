//! Procedural 16×16 block and item textures, generated at startup so the game ships with zero art assets.
//! Every pattern tiles seamlessly (noise lattices wrap at the texture edge). `generate` writes one
//! RGBA layer per `block::tex` constant; to add a texture, add the constant there and its pattern
//! arm in `pixel`. An alternate layer draws its base's pattern with another `alt` (`tex::look`).
//! Terrain lives in `nature.rs` (with the surface hints) and `ores.rs`, province rocks in
//! `geology.rs`, shared painting helpers in `paint.rs`, machines in `machines.rs`, items in `items.rs`,
//! planks, ladders and sticks in `wood.rs`, tier stripes in `stripes.rs`, bricks and quicklime in
//! `masonry.rs`, the assembler, ports, concrete and the motor in `assembly.rs`, steelmaking in `steel.rs`, steam, crushing and silos in `heavy.rs`,
//! the arc furnace, silicon, circuits and the violet pack in `electronics.rs`, solar panels and accumulators in `solar.rs`, the drone chain's parts in `robotics.rs`, water and steam pipes in `piping.rs`, worn gear in `gear.rs`, aluminium in `aluminium.rs`.

use crate::block::tex;
use crate::math::{hash3, unit};

mod aluminium;
mod assembly;
mod avatar;
mod electronics;
mod gear;
mod geology;
mod heavy;
mod items;
mod machines;
mod masonry;
mod nature;
mod ores;
mod paint;
mod piping;
mod plants;
mod robotics;
mod solar;
mod steel;
mod stripes;
mod tools;
mod wood;

use machines::{
    belt_side, belt_top, constructor, crate_wood, drill, generator, ingot, lab, lamp, miner_side, miner_top, plate,
    pole, router_top, smelter, wire,
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
    let (layer, alt) = tex::look(layer);
    let alt = u32::from(alt);
    match layer {
        tex::STONE..=tex::LEAVES
        | tex::BEDROCK
        | tex::SPENT_ROCK
        | tex::RUSTY_SOIL..=tex::PALE_SOIL
        | tex::RUSTY_GRASS_TOP..=tex::PALE_SAND => nature::pixel(layer, alt, x, y),
        tex::COAL_ORE => ores::coal(x, y, alt),
        tex::IRON_ORE => ores::iron(x, y, alt),
        tex::COPPER_ORE => ores::copper(x, y, alt),
        tex::LIMESTONE => ores::limestone(x, y, alt),
        tex::QUARTZ_ORE => ores::quartz(x, y, alt),
        tex::BAUXITE_ORE => ores::bauxite(x, y, alt),
        tex::BELT_TOP => belt_top(x, y, [192.0, 119.0, 70.0]),
        tex::FAST_BELT_TOP => belt_top(x, y, [106.0, 164.0, 176.0]),
        tex::FRAME => frame(x, y),
        tex::MINER_SIDE => miner_side(x, y, [196.0, 117.0, 69.0]),
        tex::MINER_MK2_SIDE => miner_side(x, y, [106.0, 164.0, 176.0]),
        tex::BELT_MK3_TOP => belt_top(x, y, [86.0, 140.0, 222.0]),
        tex::MINER_MK3_SIDE => miner_side(x, y, [86.0, 140.0, 222.0]),
        tex::BELT_MK4_TOP => belt_top(x, y, [154.0, 91.0, 214.0]),
        tex::MINER_MK4_SIDE => miner_side(x, y, [154.0, 91.0, 214.0]),
        tex::MINER_TOP => miner_top(x, y),
        tex::DRILL => drill(x, y),
        tex::BOX_SIDE => crate_wood(x, y, false),
        tex::BOX_TOP => crate_wood(x, y, true),
        tex::LAMP_GREEN => lamp(x, y, [96.0, 214.0, 112.0]),
        tex::LAMP_YELLOW => lamp(x, y, [236.0, 190.0, 64.0]),
        tex::LAMP_RED => lamp(x, y, [226.0, 72.0, 60.0]),
        tex::LAMP_BLUE => lamp(x, y, [74.0, 150.0, 236.0]),
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
        tex::RED_PACK | tex::GREEN_PACK | tex::IRON_ROD..=tex::FLASK_GLASS | tex::HANDLE | tex::STEEL | tex::GEAR => {
            items::pixel(layer, x, y)
        }
        tex::AVATAR_SUIT..=tex::AVATAR_VISOR | tex::AVATAR_JOINT..=tex::AVATAR_PACK => avatar::pixel(layer, x, y),
        tex::STRATEGY_FOG => [56, 68, 79, 255],
        tex::STONE_PICKAXE..=tex::IRON_SHOVEL => {
            let i = layer - tex::STONE_PICKAXE;
            tools::tool(x, y, i % 3, if i >= 3 { tools::IRON_HEAD } else { tools::STONE_HEAD })
        }
        tex::SAPLING => plants::sapling(x, y),
        tex::TORCH => plants::torch(x, y),
        tex::GRANITE => geology::granite(x, y),
        tex::SANDSTONE => geology::sandstone(x, y),
        tex::BASALT => geology::basalt(x, y),
        tex::GLASS => geology::glass(x, y),
        tex::LAMP => geology::lamp(x, y),
        tex::WATER => geology::water(x, y),
        tex::SCANNER => tools::scanner(x, y),
        tex::SCANNER_MK2 => tools::scanner_mk2(x, y),
        tex::CORE_DRILL => tools::core_drill(x, y),
        tex::PLANKS => wood::planks(x, y),
        tex::LADDER => wood::ladder(x, y),
        tex::LADDER_TOP => wood::ladder_top(x, y),
        tex::STICK => wood::stick(x, y),
        tex::STRIPE_1..=tex::STRIPE_5 => stripes::stripe(x, y, (layer - tex::STRIPE_1) as u8),
        tex::STONE_BRICKS => masonry::stone_bricks(x, y),
        tex::QUICKLIME => masonry::quicklime(x, y),
        tex::ASSEMBLER_SIDE..=tex::MOTOR => assembly::pixel(layer, x, y),
        tex::BLAST_SIDE..=tex::SLAG | tex::STEEL_INGOT..=tex::STEEL_BEAM | tex::STEEL_HEAD | tex::BLUE_PACK => {
            steel::pixel(layer, x, y)
        }
        tex::BOILER_SIDE..=tex::CRUSHED_COPPER => heavy::pixel(layer, x, y),
        tex::ARC_SIDE..=tex::VIOLET_PACK => electronics::pixel(layer, x, y),
        tex::SOLAR_TOP..=tex::ACCUMULATOR_SIDE => solar::pixel(layer, x, y),
        tex::PROCESSOR..=tex::PLANNER => robotics::pixel(layer, x, y),
        tex::PIPE_WATER..=tex::PIPE_STEAM => piping::pixel(layer, x, y),
        tex::HAULER_PACK..=tex::MINING_RIG => gear::pixel(layer, x, y),
        tex::CRUSHED_BAUXITE..=tex::CELL_TOP => aluminium::pixel(layer, x, y),
        tex::STEEL_PICKAXE..=tex::STEEL_SHOVEL => tools::tool(x, y, layer - tex::STEEL_PICKAXE, tools::STEEL_HEAD),
        _ => [255, 0, 255, 255],
    }
}

#[cfg(test)]
mod tests;
