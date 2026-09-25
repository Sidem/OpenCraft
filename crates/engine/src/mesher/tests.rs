use super::*;
use crate::block::{LEAVES, STONE};

fn mesh_single(setup: impl FnOnce(&mut Chunk)) -> MeshOutput {
    let air = Chunk::uniform(AIR);
    let mut c = Chunk::uniform(AIR);
    setup(&mut c);
    let mut refs = [&air; 27];
    refs[neighbor_index(0, 0, 0)] = &c;
    Mesher::new().mesh(&refs)
}

fn decode(v: u32) -> (u32, u32, u32, u32, u32) {
    (v & 63, (v >> 6) & 63, (v >> 12) & 63, (v >> 18) & 7, (v >> 21) & 3)
}

#[test]
fn single_block_has_six_unoccluded_faces() {
    let m = mesh_single(|c| c.set(5, 5, 5, STONE));
    assert_eq!(m.opaque_quads, 6);
    assert_eq!(m.cutout_quads, 0);
    for &v in &m.verts {
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
    assert_eq!(m.verts.len(), 24);
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
    let m = Mesher::new().mesh(&refs);
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
    let m = Mesher::new().mesh(&refs);
    // +X face is hidden by the solid neighbour chunk.
    assert!(m.verts.iter().all(|&v| decode(v).3 != 0));
    assert_eq!(m.opaque_quads, 5);
}
