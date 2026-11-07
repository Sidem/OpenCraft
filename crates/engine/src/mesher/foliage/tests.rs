use std::collections::HashMap;

use crate::block::{BlockId, AIR, BIRCH_LEAVES, LEAVES, LOG};
use crate::chunk::Chunk;
use crate::mesher::{neighbor_index, MeshOutput, Mesher, PAD_VOLUME};

static DAYLIGHT: [u8; PAD_VOLUME] = [15; PAD_VOLUME];

/// Chunks -2..=1 on each axis, filled from `block(x, y, z)` in world blocks.
fn world(block: impl Fn(i32, i32, i32) -> BlockId) -> HashMap<[i32; 3], Chunk> {
    let mut out = HashMap::new();
    for cy in -2..=1 {
        for cz in -2..=1 {
            for cx in -2..=1 {
                let mut c = Chunk::uniform(AIR);
                for y in 0..32 {
                    for z in 0..32 {
                        for x in 0..32 {
                            let b = block(cx * 32 + x as i32, cy * 32 + y as i32, cz * 32 + z as i32);
                            if b != AIR {
                                c.set(x, y, z, b);
                            }
                        }
                    }
                }
                out.insert([cx, cy, cz], c);
            }
        }
    }
    out
}

fn mesh(m: &mut Mesher, w: &HashMap<[i32; 3], Chunk>, c: [i32; 3]) -> MeshOutput {
    let air = Chunk::uniform(AIR);
    let mut refs = [&air; 27];
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(n) = w.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) {
                    refs[neighbor_index(dx, dy, dz)] = n;
                }
            }
        }
    }
    m.set_origin(c.map(|v| v * 32));
    m.mesh(&refs, &DAYLIGHT, None)
}

/// The shell's quads (kind 0 in the shaped cutout list) as world positions in fixed point (`shaped.rs`), four a quad.
fn shell_positions(out: &MeshOutput, c: [i32; 3]) -> Vec<[i32; 3]> {
    let start = out.shaped_opaque as usize * 12;
    out.shaped[start..]
        .chunks(3)
        .filter(|w| w[0] >> 24 & 7 == 0)
        .map(|w| {
            let local = [w[0] & 4095, w[0] >> 12 & 4095, w[1] & 4095];
            std::array::from_fn(|k| local[k] as i32 + c[k] * 32 * 100)
        })
        .collect()
}

fn hash(x: i32, y: i32, z: i32) -> u32 {
    let h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ (z as u32).wrapping_mul(0xC2B2_AE3D);
    (h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D) >> 16
}

/// A ragged canopy round the world origin, so it crosses the borders of the eight chunks there on every axis: most
/// cells within 6 blocks, two kinds of leaves, a trunk up into it, and stray leaves around.
fn ragged(x: i32, y: i32, z: i32) -> BlockId {
    let d2 = x * x + y * y + z * z;
    if x == 0 && z == 0 && (-9..=2).contains(&y) {
        return LOG;
    }
    let h = hash(x, y, z) % 100;
    if d2 <= 36 && h < 80 || d2 <= 100 && h < 3 {
        if x > 2 {
            BIRCH_LEAVES
        } else {
            LEAVES
        }
    } else {
        AIR
    }
}

#[test]
fn a_canopy_across_chunk_borders_is_closed_with_no_crack() {
    let w = world(ragged);
    let mut m = Mesher::new();
    // Every directed edge of the shell must be matched by as many running back: a closed surface, wound consistently,
    // whose vertices on a border are the same bits in both chunks.
    let mut edges: HashMap<([i32; 3], [i32; 3]), i32> = HashMap::new();
    let mut quads = 0;
    for cy in -1..=0 {
        for cz in -1..=0 {
            for cx in -1..=0 {
                let out = mesh(&mut m, &w, [cx, cy, cz]);
                let p = shell_positions(&out, [cx, cy, cz]);
                quads += p.len() / 4;
                for q in p.chunks(4) {
                    for i in 0..4 {
                        let (a, b) = (q[i], q[(i + 1) % 4]);
                        *edges.entry((a, b)).or_default() += 1;
                        *edges.entry((b, a)).or_default() -= 1;
                    }
                }
            }
        }
    }
    assert!(quads > 300, "the canopy has a shell: {quads} quads");
    let open: Vec<_> = edges.iter().filter(|(_, &n)| n != 0).collect();
    assert!(open.is_empty(), "{} of {} edges are open, e.g. {:?}", open.len(), edges.len(), open.first());
}

#[test]
fn a_chunk_meshes_the_same_whatever_was_meshed_before() {
    let w = world(ragged);
    let fresh = mesh(&mut Mesher::new(), &w, [0, 0, 0]);
    let mut m = Mesher::new();
    mesh(&mut m, &w, [-1, -1, -1]);
    mesh(&mut m, &w, [-1, 0, 0]);
    let again = mesh(&mut m, &w, [0, 0, 0]);
    assert!(fresh.shaped_cutout > 0);
    assert_eq!((fresh.shaped, fresh.shaped_opaque), (again.shaped, again.shaped_opaque));
}

/// The share of the shell of a cube of leaves `side` blocks wide that faces within 10° of an axis, by area.
fn square_share(side: i32) -> f64 {
    let (lo, hi) = (17 - (side + 1) / 2, 17 - (side + 1) / 2 + side);
    let w = world(|x, y, z| if [x, y, z].iter().all(|v| (lo..hi).contains(v)) { LEAVES } else { AIR });
    let p = shell_positions(&mesh(&mut Mesher::new(), &w, [0, 0, 0]), [0, 0, 0]);
    let (mut all, mut square) = (0.0, 0.0);
    for q in p.chunks(4) {
        let q = q.iter().map(|v| v.map(f64::from)).collect::<Vec<_>>();
        for [a, b, c] in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]] {
            let (u, v): ([f64; 3], [f64; 3]) =
                (std::array::from_fn(|k| b[k] - a[k]), std::array::from_fn(|k| c[k] - a[k]));
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            all += len;
            if n.iter().any(|&c| c.abs() > len * 10f64.to_radians().cos()) {
                square += len;
            }
        }
    }
    square / all
}

#[test]
fn a_cube_of_leaves_comes_out_rounded() {
    // Drawn as cubes, all of it would face an axis. A shell keeps within half a block of its leaves, so a tree-sized
    // cube becomes a ball and a hedge-sized one a cushion with rounded edges and lumps.
    let (small, big) = (square_share(3), square_share(7));
    assert!(small < 0.25, "3 blocks: {:.0}% faces an axis", small * 100.0);
    assert!(big < 0.5, "7 blocks: {:.0}% faces an axis", big * 100.0);
}

#[test]
fn the_hull_lies_inside_the_shell_and_the_fringe_outside() {
    let w = world(|x, y, z| if [x, y, z].iter().all(|v| (13..20).contains(v)) { LEAVES } else { AIR });
    let out = mesh(&mut Mesher::new(), &w, [0, 0, 0]);
    // Per edge: a shell quad and a fringe quad in the cutout list, a hull quad in the opaque one.
    assert_eq!(out.shaped_opaque * 2, out.shaped_cutout);
    let n = out.shaped_opaque as usize * 4;
    let occlusion = |v: usize| out.shaped[v * 3] >> 27 & 7;
    let radius = |v: usize| {
        let w = &out.shaped[v * 3..v * 3 + 3];
        let p = [w[0] & 4095, w[0] >> 12 & 4095, w[1] & 4095].map(|c| c as f32 / 100.0 - 2.0 - 16.5);
        (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
    };
    for hull in 0..n {
        let shell = n + (hull / 4 * 2) * 4 + hull % 4;
        let fringe = shell + 4;
        assert!(radius(hull) < radius(shell) && radius(shell) < radius(fringe), "vertex {hull}: hull, shell, fringe");
        assert!(occlusion(hull) <= occlusion(shell));
    }
}
