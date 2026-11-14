use std::collections::HashMap;

use crate::block::{BlockId, AIR, LEAVES, LOG, SNAG_LOG, STONE};
use crate::chunk::Chunk;
use crate::mesher::{neighbor_index, MeshOutput, Mesher, PAD_VOLUME};

static DAYLIGHT: [u8; PAD_VOLUME] = [15; PAD_VOLUME];

fn mesh_single(setup: impl FnOnce(&mut Chunk)) -> MeshOutput {
    let air = Chunk::uniform(AIR);
    let mut c = Chunk::uniform(AIR);
    setup(&mut c);
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    Mesher::new().mesh(&refs, &DAYLIGHT, None)
}

/// The shaped list's quads as fixed-point positions and their kinds (`shaped.rs`).
fn quads(out: &MeshOutput) -> Vec<([[u32; 3]; 4], u32)> {
    out.shaped
        .chunks(12)
        .map(|q| {
            let p = std::array::from_fn(|i| {
                let w = &q[i * 3..i * 3 + 3];
                [w[0] & 4095, w[0] >> 12 & 4095, w[1] & 4095]
            });
            (p, q[0] >> 24 & 7)
        })
        .collect()
}

const SIDE_Y: u32 = 2; // kind: a side mostly along y
const END_Y: u32 = 5; // kind: an end across y
const SIDE_X: u32 = 1;
const END_X: u32 = 4;
const fn fixed(v: f32) -> u32 {
    ((v + 2.0) * 100.0) as u32
}

/// The sides of wood `r` thick (an oak log is 0.4, a snag 0.35), and the quads of one of its caps.
fn sides(r: f32) -> usize {
    super::ring::detail(r).0
}

fn count(out: &MeshOutput, kind: u32) -> usize {
    quads(out).iter().filter(|q| q.1 == kind).count()
}

/// Every edge, by position, runs both ways as often: a closed surface wound one way.
fn closed(quads: &[([[u32; 3]; 4], u32)]) -> bool {
    let mut edges: HashMap<([u32; 3], [u32; 3]), i32> = HashMap::new();
    for (q, _) in quads {
        for i in 0..4 {
            let (a, b) = (q[i], q[(i + 1) % 4]);
            if a != b {
                *edges.entry((a, b)).or_default() += 1;
                *edges.entry((b, a)).or_default() -= 1;
            }
        }
    }
    edges.values().all(|&n| n == 0)
}

#[test]
fn a_floating_log_is_a_closed_cylinder() {
    let m = mesh_single(|c| c.set(5, 5, 5, LOG));
    assert_eq!(m.vertex_count(), 0, "no cube faces");
    assert_eq!((count(&m, SIDE_Y), count(&m, END_Y)), (sides(0.4), sides(0.4)), "a side all round, a cap at each end");
    assert!(closed(&quads(&m)), "the cylinder is closed and wound outwards");
}

#[test]
fn a_trunk_on_the_ground_flares_and_shows_only_its_top_end() {
    let m = mesh_single(|c| {
        c.set(5, 4, 5, STONE);
        for y in 5..8 {
            c.set(5, y, 5, LOG);
        }
    });
    // The foot: flare, foot to middle, its half up; the middle: one stretch; the top: its half down, up to its top.
    assert_eq!(count(&m, SIDE_Y), sides(0.4) * 6);
    assert_eq!(count(&m, END_Y), sides(0.4) / 2, "one cap, on top");
    // The stone's top face still shows round the trunk: a log hides nothing.
    assert_eq!(m.opaque_quads, 6);
}

#[test]
fn a_stub_reaches_into_its_trunk_and_caps_its_free_end() {
    let m = mesh_single(|c| {
        for y in 5..9 {
            c.set(5, y, 5, SNAG_LOG);
        }
        c.set(6, 7, 5, SNAG_LOG);
    });
    assert_eq!(count(&m, END_X), sides(0.35) / 2, "capped at its free end only");
    let xs: Vec<u32> = quads(&m).iter().filter(|q| q.1 == SIDE_X).flat_map(|q| q.0.map(|p| p[0])).collect();
    let (lo, hi) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap());
    assert!(lo < fixed(5.6) && hi == fixed(7.0), "from inside the trunk to the stub's end: {lo}..{hi}");
}

#[test]
fn a_leaning_stem_bends_gently_not_at_an_elbow() {
    // A palm's stem: straight up from (5, 3, 5), then over a block in x at y 8 and again at y 10.
    let cells = [[5, 3], [5, 4], [5, 5], [5, 6], [5, 7], [6, 8], [6, 9], [7, 10], [7, 11]];
    let mut c = Chunk::uniform(AIR);
    for [x, y] in cells {
        c.set(x as usize, y as usize, 5, crate::block::PALM_LOG);
    }
    let air = Chunk::uniform(AIR);
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    let mut g = crate::mesher::tree_grid::TreeGrid::default();
    g.fill(&refs);
    let mut nodes = super::node::Nodes::default();
    let centres: Vec<[f32; 3]> = cells.iter().map(|&[x, y]| nodes.of(&g, [x, y, 5]).centre).collect();
    // The sharpest turn between one stretch and the next.
    let way = |a: [f32; 3], b: [f32; 3]| super::unit(super::sub(b, a));
    let sharpest = centres
        .windows(3)
        .map(|w| super::dot(way(w[0], w[1]), way(w[1], w[2])).clamp(-1.0, 1.0).acos().to_degrees())
        .fold(0.0, f32::max);
    assert!(sharpest < 30.0, "a {sharpest} degree elbow: {centres:?}");
}

#[test]
fn a_whorl_where_four_branches_leave_the_trunk_is_lit() {
    // A pine's trunk with a branch out of each side at one height: the log there has wood on all six sides.
    let m = mesh_single(|c| {
        for y in 3..9 {
            c.set(8, y, 8, crate::block::PINE_LOG);
        }
        for (x, z) in [(9, 8), (7, 8), (8, 9), (8, 7), (10, 8), (6, 8), (8, 10), (8, 6)] {
            c.set(x, 5, z, crate::block::PINE_LOG);
        }
    });
    let dark = m.shaped.chunks(3).filter(|v| v[1] >> 24 & 15 == 0).count();
    assert_eq!(dark, 0, "{dark} vertices of {} unlit", m.shaped.len() / 3);
}

#[test]
fn a_log_walled_in_by_stone_draws_nothing_of_itself() {
    let m = mesh_single(|c| {
        for y in 4..7 {
            for z in 4..7 {
                for x in 4..7 {
                    c.set(x, y, z, STONE);
                }
            }
        }
        c.set(5, 5, 5, LOG);
    });
    assert_eq!(m.shaped_opaque + m.shaped_cutout, 0);
}

#[test]
fn a_staircase_of_logs_is_one_slanting_limb() {
    // Three logs touching on edges only: one limb, no caps but at its two ends, no corner between.
    let m = mesh_single(|c| {
        for i in 0..3 {
            c.set(5 + i, 5 + i, 5, LOG);
        }
    });
    let all = quads(&m);
    let ends = all.iter().filter(|q| q.1 >= 4).count();
    assert_eq!(ends, sides(0.4), "a cut at each end and nowhere else");
    // The middle log is one straight stretch from join to join, so the limb runs from (5.5, 5.5) to (7.5, 7.5).
    let ys: Vec<u32> = all.iter().flat_map(|q| q.0.map(|p| p[1])).collect();
    assert!(*ys.iter().max().unwrap() >= fixed(7.5) && *ys.iter().min().unwrap() <= fixed(5.5));
    assert!(closed(&all), "the limb is one closed surface");
}

#[test]
fn a_trunk_under_leaves_tapers_to_its_top() {
    let m = mesh_single(|c| {
        c.set(8, 4, 8, STONE);
        for y in 5..12 {
            c.set(8, y, 8, LOG);
        }
        for y in 10..13 {
            for z in 6..11 {
                for x in 6..11 {
                    if c.get(x, y, z) == AIR {
                        c.set(x, y, z, LEAVES);
                    }
                }
            }
        }
    });
    // Half the trunk's width at a height: how far its bark reaches out in x from its middle (8.5).
    let width_at = |y: f32| {
        quads(&m)
            .iter()
            .filter(|q| q.1 == SIDE_Y)
            .flat_map(|q| q.0)
            .filter(|p| p[1].abs_diff(fixed(y)) <= 2)
            .map(|p| p[0].abs_diff(fixed(8.5)))
            .max()
            .unwrap_or(0)
    };
    let (low, high) = (width_at(6.0), width_at(11.0));
    assert!(low >= 38 && high < low * 3 / 4, "thick low down ({low}), thinner at the top ({high})");
}

#[test]
fn a_living_limb_hanging_down_tapers_with_no_cut_end() {
    // A trunk, a limb out along +x, its end hanging straight down a block, all among leaves.
    let m = mesh_single(|c| {
        c.set(8, 2, 8, STONE);
        for y in 3..10 {
            c.set(8, y, 8, LOG);
        }
        for x in 9..12 {
            c.set(x, 9, 8, LOG);
        }
        c.set(12, 8, 8, LOG);
        c.set(12, 7, 8, LOG);
        for y in 6..12 {
            for z in 5..12 {
                for x in 5..15 {
                    if c.get(x, y, z) == AIR {
                        c.set(x, y, z, LEAVES);
                    }
                }
            }
        }
    });
    // No end cut across y below the limb's lowest log's middle.
    let cut_below = quads(&m).iter().filter(|q| q.1 == END_Y && q.0.iter().all(|p| p[1] < fixed(7.5))).count();
    assert_eq!(cut_below, 0, "the hanging end is cut flat");
}

#[test]
fn a_square_of_logs_is_one_thick_trunk() {
    let m = mesh_single(|c| {
        for y in 5..9 {
            for (x, z) in [(5, 5), (6, 5), (5, 6), (6, 6)] {
                c.set(x, y, z, LOG);
            }
        }
    });
    // Drawn round the square's middle (6, 6), out to nearly a block, in 24 sides.
    let xs: Vec<u32> = quads(&m).iter().filter(|q| q.1 == SIDE_Y).flat_map(|q| q.0.map(|p| p[0])).collect();
    let (lo, hi) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap());
    assert_eq!(lo + hi, 2 * fixed(6.0), "centred on the square");
    assert!(hi - lo > 180, "about two blocks across: {}", hi - lo);
    assert_eq!(count(&m, SIDE_Y) % 24, 0);
    assert!(closed(&quads(&m)));
}

/// Chunks 0 and 1 along x, filled from `block(x, y, z)`.
fn two_chunks(block: impl Fn(i32, i32, i32) -> BlockId) -> [Chunk; 2] {
    std::array::from_fn(|cx| {
        let mut c = Chunk::uniform(AIR);
        for y in 0..32 {
            for z in 0..32 {
                for x in 0..32 {
                    let b = block(cx as i32 * 32 + x as i32, y as i32, z as i32);
                    if b != AIR {
                        c.set(x, y, z, b);
                    }
                }
            }
        }
        c
    })
}

#[test]
fn a_branch_across_a_chunk_border_meets_itself() {
    // A level branch from x 28 to 35 through leaves (so it tapers), across the border x 32.
    let block = |x: i32, y: i32, z: i32| match (x, y, z) {
        (28..=35, 10, 8) => LOG,
        (26..=37, 9..=12, 6..=10) => LEAVES,
        _ => AIR,
    };
    let chunks = two_chunks(block);
    let air = Chunk::uniform(AIR);
    let mut sides: [Vec<[u32; 3]>; 2] = [Vec::new(), Vec::new()];
    for cx in 0..2 {
        let mut refs = [&air; 27];
        refs[neighbor_index(0, 0, 0)] = &chunks[cx];
        if cx == 0 {
            refs[neighbor_index(1, 0, 0)] = &chunks[1];
        } else {
            refs[neighbor_index(-1, 0, 0)] = &chunks[0];
        }
        let m = Mesher::new().mesh(&refs, &DAYLIGHT, None);
        let opaque = m.shaped_opaque as usize;
        for (q, _) in quads(&m).into_iter().take(opaque) {
            for mut p in q {
                p[0] += cx as u32 * 3200;
                if p[0] == fixed(32.0) {
                    sides[cx].push(p);
                }
            }
        }
    }
    for s in &mut sides {
        s.sort_unstable();
        s.dedup();
    }
    assert!(sides[0].len() >= 4, "the join's ring (4 sides at least) lies on the border: {:?}", sides[0]);
    assert_eq!(sides[0], sides[1], "both chunks draw the same ring there");
}
