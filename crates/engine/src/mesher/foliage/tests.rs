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

/// The cards (kind 0 in the cutout list): per quad its four positions in chunk-local blocks and its normal.
fn cards(out: &MeshOutput) -> Vec<([[f32; 3]; 4], [f32; 3])> {
    let start = out.shaped_opaque as usize * 12;
    out.shaped[start..]
        .chunks(12)
        .filter(|q| q[0] >> 24 & 7 == 0)
        .map(|q| {
            let p = std::array::from_fn(|i| {
                let w = &q[i * 3..i * 3 + 3];
                [w[0] & 4095, w[0] >> 12 & 4095, w[1] & 4095].map(|v| v as f32 / 100.0 - 2.0)
            });
            (p, octahedral(q[2] & 0xFFFF))
        })
        .collect()
}

/// `shaped::octahedral` undone (as `render/shaped.ts` does).
fn octahedral(bits: u32) -> [f32; 3] {
    let e = [(bits & 255) as f32 / 255.0 * 2.0 - 1.0, (bits >> 8) as f32 / 255.0 * 2.0 - 1.0];
    let mut n = [e[0], 1.0 - e[0].abs() - e[1].abs(), e[1]];
    if n[1] < 0.0 {
        let (x, z) = (n[0], n[2]);
        n[0] = (1.0 - z.abs()) * x.signum();
        n[2] = (1.0 - x.abs()) * z.signum();
    }
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    n.map(|v| v / l)
}

/// A ball of leaves round (16, 16, 16) of radius `r`, with a trunk up into it.
fn ball(r: i32) -> impl Fn(i32, i32, i32) -> BlockId {
    move |x, y, z| {
        let (dx, dy, dz) = (x - 16, y - 16, z - 16);
        if dx == 0 && dz == 0 && (6..=16).contains(&y) {
            LOG
        } else if dx * dx + dy * dy + dz * dz <= r * r {
            if dx > 1 {
                BIRCH_LEAVES
            } else {
                LEAVES
            }
        } else {
            AIR
        }
    }
}

#[test]
fn a_lone_leaf_is_three_cards_and_a_buried_one_one() {
    let lone = world(|x, y, z| if (x, y, z) == (5, 5, 5) { LEAVES } else { AIR });
    assert_eq!(cards(&mesh(&mut Mesher::new(), &lone, [0, 0, 0])).len(), 3);
    // A 3×3×3 block of leaves: 26 that show, one inside them.
    let cube =
        world(|x, y, z| if (4..7).contains(&x) && (4..7).contains(&y) && (4..7).contains(&z) { LEAVES } else { AIR });
    assert_eq!(cards(&mesh(&mut Mesher::new(), &cube, [0, 0, 0])).len(), 26 * 3 + 1);
}

#[test]
fn a_chunk_meshes_the_same_whatever_was_meshed_before() {
    let w = world(ball(6));
    let fresh = mesh(&mut Mesher::new(), &w, [0, 0, 0]);
    let mut m = Mesher::new();
    mesh(&mut m, &w, [-1, -1, -1]);
    mesh(&mut m, &w, [0, 0, -1]);
    let again = mesh(&mut m, &w, [0, 0, 0]);
    assert!(fresh.shaped_cutout > 0);
    assert_eq!((fresh.shaped, fresh.shaped_opaque), (again.shaped, again.shaped_opaque));
}

#[test]
fn a_crown_is_lit_as_a_ball() {
    // Cards on the outer shell of the ball have normals pointing out of it, so it shades as one rounded volume.
    let out = mesh(&mut Mesher::new(), &world(ball(6)), [0, 0, 0]);
    let (mut outer, mut out_facing) = (0, 0);
    for (p, n) in cards(&out) {
        let c: [f32; 3] = std::array::from_fn(|k| p.iter().map(|v| v[k]).sum::<f32>() / 4.0 - 16.5);
        let d = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        if d > 4.5 {
            outer += 1;
            out_facing += usize::from(c[0] * n[0] + c[1] * n[1] + c[2] * n[2] > 0.3 * d);
        }
    }
    assert!(outer > 300 && out_facing * 100 >= outer * 90, "{out_facing} of {outer} outer cards face out");
}

#[test]
fn leaves_on_a_chunk_border_are_drawn_once() {
    // A slab of leaves across x = 0: each chunk draws the cards of its own leaves only.
    let w = world(|x, y, z| if (-2..2).contains(&x) && y == 5 && (4..8).contains(&z) { LEAVES } else { AIR });
    let left = cards(&mesh(&mut Mesher::new(), &w, [-1, 0, 0])).len();
    let right = cards(&mesh(&mut Mesher::new(), &w, [0, 0, 0])).len();
    assert_eq!((left, right), (2 * 4 * 3, 2 * 4 * 3));
}
