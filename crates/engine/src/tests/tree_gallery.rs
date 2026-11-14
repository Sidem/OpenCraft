//! Writes a save with every tree species side by side on a flat platform near spawn (ignored: a tool, not a check):
//!   cargo test -q -p opencraft-engine tree_gallery -- --ignored
//! Output: `artifacts/tree-gallery.raw.ocworld` (raw save bytes, git-ignored); gzip it for the Worlds panel's import
//! (the panel also accepts raw bytes). Row one (z = spawn z): the 11 mature kinds 20 blocks apart along +x. Row two
//! (12 blocks further along +z): the young form of each. The player stands a few blocks before the first tree, facing +x.
//! A row 8 blocks before the trees (-z): each wood's planks (two high), floor and door, 4 blocks apart along +x.
//! The save is loaded back and the first log of every mature tree is asserted.

use super::run_until_ready;
use crate::block::{AIR, DIRT, GRASS, SAND, STONE};
use crate::math::IVec3;
use crate::worldgen::{grow, grow_young, height_of, KINDS};
use crate::Game;

const SPACING: i32 = 20;
const SEED: u32 = 1337;
const PALM: usize = 5;
const CLEAR_HEIGHT: i32 = 44;

#[test]
#[ignore]
fn tree_gallery() {
    let mut g = Game::new_creative(SEED, 6);
    run_until_ready(&mut g);
    let spawn = g.body().pos.floor();
    let top = spawn.y + 2; // y of the grass layer
    let (x0, z0) = (spawn.x + 8, spawn.z);
    let n = KINDS.len() as i32;
    let (xa, xb) = (x0 - 14, x0 + (n - 1) * SPACING + 14);
    let (za, zb) = (z0 - 14, z0 + 12 + 14);
    let world = &mut g.sim.world;
    for z in za..=zb {
        for x in xa..=xb {
            for y in top - 4..=top + CLEAR_HEIGHT {
                let b = match top - y {
                    d if d < 0 => AIR,
                    0 => GRASS,
                    1 | 2 => DIRT,
                    _ => STONE,
                };
                world.set_block_anywhere_later(IVec3::new(x, y, z), b);
            }
        }
    }
    let mut spots = Vec::new();
    for (k, row) in (0..KINDS.len()).flat_map(|k| [(k, 0), (k, 1)]) {
        let (x, z) = (x0 + k as i32 * SPACING, z0 + row * 12);
        let ground = IVec3::new(x, top, z);
        if k == PALM || KINDS[k].ground == SAND {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    world.set_block_anywhere_later(IVec3::new(x + dx, top, z + dz), SAND);
                }
            }
        }
        let (mut logs, mut leaves) = (0, 0);
        let mut put = |p: IVec3, b, replace: bool| {
            if b == KINDS[k].log {
                logs += 1;
            } else if b == KINDS[k].leaves {
                leaves += 1;
            }
            if replace || world.block_anywhere_or_generate(p) == AIR {
                world.set_block_anywhere_later(p, b);
            }
        };
        if row == 0 {
            grow(k, ground, height_of(k, 3), SEED, &mut put);
            spots.push((k, ground, KINDS[k].log));
        } else {
            grow_young(k, ground, 5, SEED, &mut put);
        }
        println!("kind {k} row {row}: x {x} z {z} logs {logs} leaves {leaves}");
    }
    for w in 0..crate::wood::WOODS {
        let block = |form| crate::wood::item(w, form).0 as crate::block::BlockId;
        let x = x0 + w as i32 * 4;
        let z = z0 - 8;
        use crate::wood::Form;
        for (p, b) in [
            ((x, top + 1), Form::Planks),
            ((x, top + 2), Form::Planks),
            ((x + 1, top + 1), Form::Floor),
            ((x + 2, top + 1), Form::Door),
        ] {
            world.set_block_anywhere_later(IVec3::new(p.0, p.1, z), block(b));
        }
    }
    let stand = IVec3::new(x0 - 8, top + 1, z0);
    g.teleport(stand.x as f64 + 0.5, stand.y as f64, stand.z as f64 + 0.5);
    g.set_look(std::f64::consts::FRAC_PI_2, -0.1);
    g.toggle_fly();
    g.run_ticks(5);
    println!("player at {stand:?} facing +x; ground y {top}");

    let bytes = g.save();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tree-gallery.raw.ocworld");
    std::fs::write(&path, &bytes).unwrap();
    println!("wrote {} ({} bytes)", path.display(), bytes.len());

    let mut back = Game::load(&bytes, 6).expect("saved gallery loads");
    assert_eq!(back.worldgen_version(), crate::worldgen::WORLDGEN_VERSION);
    for (k, ground, log) in spots {
        let at = back.sim.world.block_anywhere_or_generate(ground + IVec3::new(0, 1, 0));
        assert_eq!(at, log, "kind {k} has no log above its foot");
    }
}
