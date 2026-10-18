use super::*;
use crate::block::{flow, STONE, WATER};
use crate::chunk::Chunk;
use crate::mesher::{neighbor_index, MeshOutput, Mesher, PAD_VOLUME};

static DAYLIGHT: [u8; PAD_VOLUME] = [15; PAD_VOLUME];

/// A liquid vertex: position, face, drop in sixteenths and the kind and direction (`kind | dir << 2`).
struct Vertex {
    pos: (u32, u32, u32),
    face: u32,
    drop: u32,
    info: u32,
}

/// Meshes a stone floor (y 3, x and z 0..16) with `setup`'s water on it, and returns the liquid vertices.
fn water_vertices(setup: impl FnOnce(&mut Chunk)) -> Vec<Vertex> {
    let air = Chunk::uniform(0);
    let mut c = Chunk::uniform(0);
    for z in 0..16 {
        for x in 0..16 {
            c.set(x, 3, z, STONE);
        }
    }
    setup(&mut c);
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    let m: MeshOutput = Mesher::new().mesh(&refs, &DAYLIGHT, None);
    let start = ((m.opaque_quads + m.cutout_quads) * 4) as usize;
    m.verts[start..m.vertex_count()]
        .iter()
        .map(|&v| Vertex {
            pos: (v & 63, (v >> 6) & 63, (v >> 12) & 63),
            face: (v >> 18) & 7,
            drop: (v >> 21) & 3 | ((v >> 28) & 3) << 2,
            info: (v >> 23) & 31,
        })
        .collect()
}

/// The vertices on the top faces (+Y).
fn tops(v: &[Vertex]) -> Vec<&Vertex> {
    v.iter().filter(|v| v.face == 2).collect()
}

#[test]
fn heights_step_evenly_from_a_source_to_the_thinnest_flow() {
    let heights: Vec<u32> = [WATER, flow(7), flow(6), flow(5), flow(4), flow(3), flow(2), flow(1)].map(height).to_vec();
    assert_eq!(heights, [15, 13, 11, 9, 7, 5, 3, 1]);
}

#[test]
fn a_pool_is_level_inside_and_slopes_down_at_its_open_edge() {
    let v = water_vertices(|c| {
        for z in 4..9 {
            for x in 4..9 {
                c.set(x, 4, z, WATER);
            }
        }
    });
    let drops: Vec<u32> = tops(&v).iter().map(|v| v.drop).collect();
    assert_eq!(drops.iter().min(), Some(&1), "inside: a source's height, 15/16");
    let corner = tops(&v).into_iter().find(|v| v.pos == (4, 5, 4)).unwrap().drop;
    assert_eq!(drops.iter().max(), Some(&corner), "the outer corner sits lowest");
    assert!((3..=8).contains(&corner), "but a gentle meniscus: {corner}");
}

#[test]
fn a_spreading_flow_thins_out_smoothly() {
    // A strip three cells wide: levels 7 down to 1 along x.
    let v = water_vertices(|c| {
        for k in 0..7 {
            for z in 4..7 {
                c.set(4 + k, 4, z, flow(7 - k as u8));
            }
        }
    });
    let drops: Vec<u32> = (5..11).map(|x| tops(&v).into_iter().find(|v| v.pos == (x, 5, 5)).unwrap().drop).collect();
    assert!(drops.windows(2).all(|w| w[0] < w[1]), "each boundary lower than the last: {drops:?}");
    assert!(drops[0] <= 4 && drops[5] >= 13, "from nearly full to nearly nothing: {drops:?}");
}

/// The bug in the lake-edge screenshot: where a high cell stood beside a lower one the wall above the lower
/// surface was culled (water against water), so the shore showed through the gap. The surface is now one sheet:
/// every vertex at a shared position has the same drop, and no wall stands between the two cells.
#[test]
fn neighbouring_water_cells_share_their_edge_and_leave_no_gap() {
    let v = water_vertices(|c| {
        c.set(5, 4, 5, WATER);
        c.set(6, 4, 5, flow(2));
    });
    let top = tops(&v);
    for a in &top {
        for b in &top {
            if a.pos == b.pos {
                assert_eq!(a.drop, b.drop, "a split surface at {:?}", a.pos);
            }
        }
    }
    let wall = v.iter().filter(|v| v.face < 2 && v.pos.0 == 6).count();
    assert_eq!(wall, 0, "no face between two water cells");
    // And the walls that do show (towards air) all reach down to the floor of the block.
    assert!(v.iter().any(|v| v.face == 4 && v.pos.1 == 4), "side faces towards the air are still drawn");
}

#[test]
fn still_flowing_rapid_and_falling_water_are_told_apart() {
    let v = water_vertices(|c| {
        // A pool, a gentle slope away from it, and a waterfall: a source on a ledge over a column of falling water.
        for z in 4..7 {
            for x in 2..5 {
                c.set(x, 4, z, WATER);
            }
            for k in 0..3 {
                c.set(5 + k, 4, z, flow(6 - k as u8));
            }
        }
        for y in 4..8 {
            c.set(12, y, 10, if y == 7 { WATER } else { flow(7) });
        }
        c.set(13, 4, 10, flow(7));
    });
    let info_at = |x, y, z| tops(&v).into_iter().find(|v| v.pos == (x, y, z)).map(|v| v.info);
    assert_eq!(info_at(3, 5, 5).map(|i| i & 3), Some(KIND_STILL), "the pool's middle");
    let slope = info_at(6, 5, 5).unwrap();
    assert_eq!((slope & 3, slope >> 2), (KIND_LAMINAR, 0), "a gentle slope towards +X");
    assert!(
        v.iter().any(|v| v.face < 2 && (12..=13).contains(&v.pos.0) && v.info & 3 == KIND_FALLING),
        "the fall's walls"
    );
    let foot = info_at(14, 5, 10).or(info_at(13, 5, 10)).unwrap();
    assert_eq!(foot & 3, KIND_TURBULENT, "splash beside the foot of a fall");
}

#[test]
fn a_cell_under_water_is_full_height_and_shows_no_surface() {
    let v = water_vertices(|c| {
        c.set(5, 4, 5, WATER);
        c.set(5, 5, 5, WATER);
    });
    assert!(v.iter().all(|v| v.face != 2 || v.pos.1 == 6), "only the upper cell has a top");
    let low = v.iter().filter(|v| v.face == 4 && v.pos.1 == 5).map(|v| v.drop);
    assert!(low.clone().count() > 0 && low.into_iter().all(|d| d == 0), "the lower cell's wall is full height");
}
