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
    Mesher::new().mesh(&refs, &DAYLIGHT, None)
}

fn decode(v: u32) -> (u32, u32, u32, u32, u32) {
    (v & 63, (v >> 6) & 63, (v >> 12) & 63, (v >> 18) & 7, (v >> 21) & 3)
}

#[test]
fn turf_sides_draw_green_in_daylight_and_soil_in_shade() {
    use crate::block::{GRASS, RUSTY_SOIL};
    use face_layer::GREEN_SKY;
    let c = [3, 4, 5];
    let (top, soil) = (face_layer(GRASS, 2, c, 15), face_layer(GRASS, 3, c, 0));
    assert_ne!(top, soil);
    assert_eq!(face_layer(GRASS, 0, c, 15), top, "a side in daylight is as green as the top");
    assert_eq!(face_layer(GRASS, 3, c, GREEN_SKY), top, "so is an underside a little into shade");
    assert_eq!(face_layer(GRASS, 4, c, GREEN_SKY - 1), soil, "deeper in shade: soil");
    assert_eq!(face_layer(GRASS, 2, c, 0), top, "the top stays green under a canopy");
    assert_eq!(face_layer(RUSTY_SOIL, 1, c, 15), face_layer(RUSTY_SOIL, 2, c, 15), "stained soils too");
    assert_eq!(face_layer(STONE, 0, c, 0), FACE_TEX[STONE as usize][0], "other blocks keep their own faces");
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
fn leaves_go_to_the_shaped_list_not_the_cubes() {
    let m = mesh_single(|c| c.set(5, 5, 5, LEAVES));
    assert_eq!(m.vertex_count(), 0);
    assert_eq!((m.shaped_opaque, m.shaped_cutout), (0, 3), "three cutout cards");
    // A leaf in the corner is the chunk's own: all its cards are drawn here.
    let corner = mesh_single(|c| c.set(0, 0, 0, LEAVES));
    assert_eq!((corner.shaped_opaque, corner.shaped_cutout), (0, 3));
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
    let m = Mesher::new().mesh(&refs, &DAYLIGHT, None);
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
    let m = Mesher::new().mesh(&refs, &DAYLIGHT, None);
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
    let m = Mesher::new().mesh(&refs, &light, None);
    assert!(m.opaque_quads > 6, "the top can't be one quad any more");
    let top: Vec<u8> = (0..m.vertex_count()).filter(|&i| decode(m.verts[i]).3 == 2).map(|i| m.light(i)).collect();
    assert!(top.contains(&15) && top.contains(&5) && top.contains(&10), "the corners on the step average: {top:?}");
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

/// Cube vertices carry 12-bit layers (`quad.rs`, read by `shaders.ts`), while `block::turn::MASK` and shaped vertices
/// cap terrain block layers at 2048 (11 bits).
#[test]
fn cube_faces_use_layers_the_vertex_can_hold() {
    let cap = (crate::block::turn::MASK_WORDS * 16) as u16;
    assert_eq!(cap, 2048);
    for (id, def) in crate::block::BLOCK_DEFS.iter().enumerate() {
        assert!(def.faces.iter().all(|&l| l < cap), "block {id} ({}) has a face layer past {}", def.name, cap - 1);
    }
}

/// ROOM2: cube quads, plants and shaped vertices encode layers above 600 (up to 2047) alongside full 5-bit surface
/// paint and light, and greedy merge distinguishes layers that share the same low 9 bits.
#[test]
fn cube_and_shaped_vertices_encode_layers_above_600_with_paint() {
    let corners: u64 = (0..4).fold(0u64, |acc, i| acc | ((3u64 | (0xA5u64 << 2)) << (i * CORNER_BITS)));
    let make_key = |layer: u16| KEY_FLAG | (u64::from(layer) << KEY_LAYER) | corners;

    // Layers sharing the same low 9 bits (89 vs 601 = 89 + 512) produce distinct merge keys.
    assert_ne!(make_key(89), make_key(601));

    for (layer, paint) in [(601u16, 0u8), (601, 5), (1024, 16), (2047, 11), (4095, 16)] {
        let mut verts = Vec::new();
        let mut lights = Vec::new();
        emit_quad((&mut verts, &mut lights), 2, (1, 2, 0), 5, (3, 4), (1, 1), (make_key(layer), paint));
        for pair in lights.chunks_exact(4) {
            verts.push(u32::from_le_bytes([pair[0], pair[1], pair[2], pair[3]]));
        }
        let m = MeshOutput {
            verts,
            opaque_quads: 1,
            cutout_quads: 0,
            liquid_quads: 0,
            translucent_quads: 0,
            shaped: Vec::new(),
            shaped_opaque: 0,
            shaped_cutout: 0,
            shaped_translucent: 0,
        };
        for i in 0..4 {
            assert_eq!(m.layer(i), layer, "layer round-trip failed for ({layer}, paint={paint})");
            assert_eq!(m.paint(i), paint, "paint round-trip failed for ({layer}, paint={paint})");
            assert_eq!(m.light(i), 0xA5, "light corrupted for ({layer}, paint={paint})");
        }
    }

    // Plant crossed quads carry layers above 600 through the same split encoding.
    let mut mesher = Mesher::new();
    mesher.push_plant(4, 5, 6, 650, 0x0F);
    let mut verts = mesher.cutout.clone();
    for pair in mesher.cutout_light.chunks_exact(4) {
        verts.push(u32::from_le_bytes([pair[0], pair[1], pair[2], pair[3]]));
    }
    let pm = MeshOutput {
        verts,
        opaque_quads: 0,
        cutout_quads: 4,
        liquid_quads: 0,
        translucent_quads: 0,
        shaped: Vec::new(),
        shaped_opaque: 0,
        shaped_cutout: 0,
        shaped_translucent: 0,
    };
    for i in 0..pm.vertex_count() {
        assert_eq!(pm.layer(i), 650);
        assert_eq!(pm.paint(i), 0);
        assert_eq!(pm.light(i), 0x0F);
    }

    // Shaped vertex word 2 holds 11-bit layer (bits 16..27) and 5-bit paint (bits 27..32).
    for (layer, paint) in [(601u16, 5u8), (2047, 16)] {
        let packed = (layer & 0x7FF) | (u16::from(paint) << 11);
        let w = shaped::pack(&shaped::Vert {
            cell: [1, 2, 3],
            off: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            u: 0.0,
            kind: shaped::KIND_PLANT,
            occlusion: 7,
            light: 0x0F,
            layer: packed,
        });
        assert_eq!((w[2] >> 16) & 2047, u32::from(layer));
        assert_eq!(w[2] >> 27, u32::from(paint));
    }
}
