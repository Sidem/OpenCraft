use std::collections::HashMap;

use super::SIDES;
use crate::block::{AIR, LOG, SNAG_LOG, STONE};
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

const SIDE_Y: u32 = 2; // kind: a side around y
const END_Y: u32 = 5; // kind: an end across y
const SIDE_X: u32 = 1;
const END_X: u32 = 4;

fn count(out: &MeshOutput, kind: u32) -> usize {
    quads(out).iter().filter(|q| q.1 == kind).count()
}

#[test]
fn a_floating_log_is_a_closed_cylinder() {
    let m = mesh_single(|c| c.set(5, 5, 5, LOG));
    assert_eq!(m.vertex_count(), 0, "no cube faces");
    assert_eq!((count(&m, SIDE_Y), count(&m, END_Y)), (SIDES, SIDES), "a side all round, a cap at each end");
    // Closed: every edge, by position, runs both ways as often.
    let mut edges: HashMap<([u32; 3], [u32; 3]), i32> = HashMap::new();
    for (q, _) in quads(&m) {
        for i in 0..4 {
            let (a, b) = (q[i], q[(i + 1) % 4]);
            if a != b {
                *edges.entry((a, b)).or_default() += 1;
                *edges.entry((b, a)).or_default() -= 1;
            }
        }
    }
    assert!(edges.values().all(|&n| n == 0), "the cylinder is closed and wound outwards");
}

#[test]
fn a_trunk_on_the_ground_flares_and_shows_only_its_top_end() {
    let m = mesh_single(|c| {
        c.set(5, 4, 5, STONE);
        for y in 5..8 {
            c.set(5, y, 5, LOG);
        }
    });
    // Three logs: the foot has two rings of sides (the flare), the others one; one cap, on top.
    assert_eq!(count(&m, SIDE_Y), SIDES * 4);
    assert_eq!(count(&m, END_Y), SIDES / 2);
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
    assert_eq!(count(&m, SIDE_X), SIDES, "the stub runs along x");
    assert_eq!(count(&m, END_X), SIDES / 2, "capped at its free end only");
    // Its sides reach from half into the trunk (x = 5.5) to its free end (x = 7).
    let xs: Vec<u32> = quads(&m).iter().filter(|q| q.1 == SIDE_X).flat_map(|q| q.0.map(|p| p[0])).collect();
    let fixed = |x: f32| ((x + 2.0) * 100.0) as u32;
    assert_eq!((*xs.iter().min().unwrap(), *xs.iter().max().unwrap()), (fixed(5.5), fixed(7.0)));
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
