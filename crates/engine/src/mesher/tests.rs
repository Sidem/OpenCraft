use super::*;
use crate::block::{LEAVES, STONE, WATER};

/// Full sky light everywhere.
static DAYLIGHT: [u8; PAD_VOLUME] = [15; PAD_VOLUME];

fn mesh_single(setup: impl FnOnce(&mut Chunk)) -> MeshOutput {
    let air = Chunk::uniform(AIR);
    let mut c = Chunk::uniform(AIR);
    setup(&mut c);
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    Mesher::new().mesh(&refs, &DAYLIGHT)
}

fn decode(v: u32) -> (u32, u32, u32, u32, u32) {
    (v & 63, (v >> 6) & 63, (v >> 12) & 63, (v >> 18) & 7, (v >> 21) & 3)
}

#[test]
fn single_block_has_six_unoccluded_faces() {
    let m = mesh_single(|c| c.set(5, 5, 5, STONE));
    assert_eq!(m.opaque_quads, 6);
    assert_eq!(m.cutout_quads, 0);
    for &v in &m.verts[..m.vertex_count()] {
        let (x, y, z, _, ao) = decode(v);
        assert!((5..=6).contains(&x) && (5..=6).contains(&y) && (5..=6).contains(&z));
        assert_eq!(ao, 3);
    }
}

#[test]
fn row_of_blocks_is_merged() {
    let m = mesh_single(|c| {
        for x in 0..10 {
            c.set(x, 3, 3, STONE);
        }
    });
    assert_eq!(m.opaque_quads, 6, "a 10x1x1 bar should become 6 quads");
}

#[test]
fn slab_is_merged_into_six_quads() {
    let m = mesh_single(|c| {
        for z in 0..32 {
            for x in 0..32 {
                c.set(x, 0, z, STONE);
            }
        }
    });
    assert_eq!(m.opaque_quads, 6);
}

#[test]
fn hidden_faces_are_culled() {
    let m = mesh_single(|c| {
        c.set(1, 1, 1, STONE);
        c.set(2, 1, 1, STONE);
    });
    // Two touching cubes: 10 visible faces, merged down to 6.
    assert_eq!(m.opaque_quads, 6);
}

#[test]
fn leaves_go_to_cutout_pass() {
    let m = mesh_single(|c| c.set(0, 0, 0, LEAVES));
    assert_eq!(m.opaque_quads, 0);
    assert_eq!(m.cutout_quads, 6);
    assert_eq!(m.vertex_count(), 24);
}

#[test]
fn corner_occlusion_darkens_vertex() {
    // A block with another block diagonally above-adjacent: the top face gets one darker edge.
    let m = mesh_single(|c| {
        c.set(4, 4, 4, STONE);
        c.set(5, 5, 4, STONE);
    });
    let top_face_min_ao = m
        .verts
        .iter()
        .map(|&v| decode(v))
        .filter(|&(_, y, _, f, _)| f == 2 && y == 5)
        .map(|(.., ao)| ao)
        .min()
        .unwrap();
    assert!(top_face_min_ao < 3);
}

#[test]
fn machines_are_left_to_the_host_and_do_not_hide_faces() {
    use crate::block::{BELT, MINER, STORAGE};
    let m = mesh_single(|c| {
        c.set(4, 4, 4, MINER);
        c.set(5, 4, 4, STONE);
        c.set(8, 4, 4, BELT);
    });
    // Only the stone is meshed, with all six faces (the miner beside it hides nothing).
    assert_eq!(m.opaque_quads, 6);
    let boxed = mesh_single(|c| c.set(1, 1, 1, STORAGE));
    assert_eq!(boxed.opaque_quads, 6, "storage boxes are plain cubes");
}

#[test]
fn uniform_solid_neighbourhood_is_empty() {
    let stone = Chunk::uniform(STONE);
    let refs = [&stone; 27];
    let m = Mesher::new().mesh(&refs, &DAYLIGHT);
    assert_eq!(m.verts.len(), 0);
}

#[test]
fn faces_on_chunk_border_respect_neighbours() {
    let air = Chunk::uniform(AIR);
    let stone = Chunk::uniform(STONE);
    let mut c = Chunk::uniform(AIR);
    c.set(31, 0, 0, STONE);
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    refs[neighbor_index(1, 0, 0)] = &stone;
    let m = Mesher::new().mesh(&refs, &DAYLIGHT);
    // +X face is hidden by the solid neighbour chunk.
    assert!(m.verts[..m.vertex_count()].iter().all(|&v| decode(v).3 != 0));
    assert_eq!(m.opaque_quads, 5);
}

#[test]
fn every_vertex_carries_its_light() {
    let m = mesh_single(|c| c.set(5, 5, 5, STONE));
    assert!((0..m.vertex_count()).all(|i| m.light(i) == 15));
}

#[test]
fn a_change_in_light_stops_merging_and_is_smoothed() {
    let air = Chunk::uniform(AIR);
    let mut c = Chunk::uniform(AIR);
    for z in 0..32 {
        for x in 0..32 {
            c.set(x, 0, z, STONE);
        }
    }
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    // Bright sky over the west half (padded x below 17), dim over the east half.
    let light: Vec<u8> = (0..PAD_VOLUME).map(|i| if i % PAD < 17 { 15 } else { 5 }).collect();
    let m = Mesher::new().mesh(&refs, &light);
    assert!(m.opaque_quads > 6, "the top can't be one quad any more");
    let top: Vec<u8> = (0..m.vertex_count()).filter(|&i| decode(m.verts[i]).3 == 2).map(|i| m.light(i)).collect();
    assert!(top.contains(&15) && top.contains(&5) && top.contains(&10), "the corners on the step average: {top:?}");
}

#[test]
fn water_shows_only_towards_open_cells_with_a_lowered_surface() {
    // A 3 × 2 × 3 pool standing on a stone floor, open to the air on every other side.
    let m = mesh_single(|c| {
        for z in 3..8 {
            for x in 3..8 {
                c.set(x, 3, z, STONE);
            }
        }
        for y in 4..6 {
            for z in 4..7 {
                for x in 4..7 {
                    c.set(x, y, z, WATER);
                }
            }
        }
    });
    // One top quad, and per side a lower and a surface row (they differ in surface corners); none below.
    assert_eq!(m.liquid_quads, 9);
    let start = ((m.opaque_quads + m.cutout_quads) * 4) as usize;
    for &v in &m.verts[start..m.vertex_count()] {
        let (_, y, _, face, surface) = decode(v);
        assert_ne!(face, 3, "no face towards the floor");
        assert_eq!(surface, u32::from(y == 6), "only the top edge is the water line");
    }
    // The floor still shows through the water: its top face is meshed where water covers it.
    assert!(m.opaque_quads >= 6);
}

#[test]
fn water_inside_water_is_trivially_empty() {
    let water = Chunk::uniform(WATER);
    let air = Chunk::uniform(AIR);
    let mut refs = [&water; 27];
    assert!(Mesher::is_trivially_empty(&refs));
    refs[neighbor_index(0, 1, 0)] = &air;
    assert!(!Mesher::is_trivially_empty(&refs));
}
