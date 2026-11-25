use crate::action::Action;
use crate::block::{
    self, BLUE_CONCRETE, CONCRETE, GLASS, GLASS_KILN, GREEN_ROOF, OAK_TREE_LOG, PLANKS, PLASTER, QUARTZ_ORE,
    RED_CONCRETE, RED_ROOF, SAND, STAINED_GLASS_COUNT, STAINED_GLASS_FIRST, STONE_BRICKS, WOOD_DOOR, WOOD_FLOOR,
};
use crate::chunk::Chunk;
use crate::colour::{
    all_faces, colour_of_pigment, face_of_normal, get_face, is_legacy_concrete, is_legacy_concrete_item, is_paintable,
    legacy_concrete_colour, pigment_of, set_face, PaintCharges, COUNT, FACES_PER_PIGMENT, NONE,
};
use crate::inventory::Inventory;
use crate::item::{self, ItemId, GLASS_KILN_ITEM, PAINT_SPRAYER, PIGMENT_BLUE, PIGMENT_RED};
use crate::math::IVec3;
use crate::mesher::Mesher;
use crate::mode::{creative_items, Mode};
use crate::sim::{PlayerId, Sim};

#[test]
fn face_packing_and_pigment_bijection() {
    let mut p = 0u32;
    for f in 0..6 {
        assert_eq!(get_face(p, f), NONE);
        p = set_face(p, f, (f as u8) + 1);
    }
    for f in 0..6 {
        assert_eq!(get_face(p, f), (f as u8) + 1);
    }
    p = set_face(p, 2, 16);
    assert_eq!(get_face(p, 2), 16);
    assert_eq!(get_face(p, 3), 4);
    assert_eq!(all_faces(0), 0);
    for f in 0..6 {
        assert_eq!(get_face(all_faces(11), f), 11);
    }
    assert_eq!(face_of_normal(IVec3::new(0, 1, 0)), Some(2));
    assert_eq!(face_of_normal(IVec3::new(-1, 0, 0)), Some(1));
    assert_eq!(face_of_normal(IVec3::ZERO), None);

    assert_eq!(pigment_of(0), None);
    for c in 1..=COUNT as u8 {
        let pig = pigment_of(c).expect("pigment for colour");
        assert_eq!(colour_of_pigment(pig), Some(c));
    }
}

#[test]
fn paintable_rules_keep_woods_natural_and_glass_kiln_only() {
    for b in [CONCRETE, PLASTER, STONE_BRICKS, RED_ROOF, GREEN_ROOF] {
        assert!(is_paintable(b), "{} should be paintable", block::def(b).name);
    }
    // Woods stay natural-only.
    for b in [PLANKS, WOOD_FLOOR, WOOD_DOOR, OAK_TREE_LOG] {
        assert!(!is_paintable(b), "{} must stay natural-only", block::def(b).name);
    }
    for w in 0..block::OTHER_WOODS {
        assert!(!is_paintable(block::planks_of(w)));
        assert!(!is_paintable(block::floor_of(w)));
        assert!(!is_paintable(block::door_of(w)));
    }
    // Glass and stained glass are made in the Glass Kiln, not spray-painted.
    assert!(!is_paintable(GLASS));
    for i in 0..STAINED_GLASS_COUNT as u16 {
        assert!(!is_paintable(STAINED_GLASS_FIRST + i));
    }
}

#[test]
fn paint_charges_spend_one_pigment_per_24_surfaces_and_strip_is_free() {
    let mut charges = PaintCharges::default();
    let mut inv = Inventory::default();
    assert!(!charges.spend(1, &mut inv, false));
    inv.add(PIGMENT_RED, 1);
    for i in 0..FACES_PER_PIGMENT {
        assert!(charges.spend(1, &mut inv, false), "surface {i}");
    }
    assert_eq!(inv.count(PIGMENT_RED), 0);
    assert_eq!(charges.remaining(1), 0);
    assert!(!charges.spend(1, &mut inv, false));
    // Stripping and creative painting cost nothing.
    assert!(charges.spend(NONE, &mut inv, false));
    assert!(charges.spend(1, &mut inv, true));
}

#[test]
fn paint_face_action_paints_only_one_surface_and_ignores_wood() {
    let mut sim = Sim::new(42, 2);
    let pos = IVec3::new(4, 30, 4);
    let wood_pos = IVec3::new(5, 30, 4);
    sim.world.set_block_anywhere(pos, CONCRETE);
    sim.world.set_block_anywhere(wood_pos, PLANKS);
    sim.players[0].as_mut().unwrap().inventory.add(PIGMENT_RED, 1);

    // Spray +Y face red: only +Y gets painted, other 5 faces stay unpainted.
    sim.queue(0, PlayerId(0), Action::PaintFace { pos, face: 2, colour: 1 });
    // Re-spraying same face red costs nothing extra.
    sim.queue(0, PlayerId(0), Action::PaintFace { pos, face: 2, colour: 1 });
    // Wood refuses paint.
    sim.queue(0, PlayerId(0), Action::PaintFace { pos: wood_pos, face: 2, colour: 1 });
    sim.step();

    assert_eq!(get_face(sim.world.paint_anywhere(pos), 2), 1);
    for f in [0, 1, 3, 4, 5] {
        assert_eq!(get_face(sim.world.paint_anywhere(pos), f), NONE);
    }
    assert_eq!(sim.paint.remaining(1), FACES_PER_PIGMENT - 1);
    assert_eq!(sim.world.paint_anywhere(wood_pos), 0);

    // Stripping face 2 back to unpainted is free.
    sim.queue(1, PlayerId(0), Action::PaintFace { pos, face: 2, colour: NONE });
    sim.step();
    assert_eq!(sim.world.paint_anywhere(pos), 0);
    assert_eq!(sim.paint.remaining(1), FACES_PER_PIGMENT - 1);
}

#[test]
fn legacy_concrete_items_hidden_from_creative_and_migrate_on_place_and_chunk_load() {
    let items = creative_items();
    assert!(items.contains(&GLASS_KILN_ITEM));
    assert!(items.contains(&PAINT_SPRAYER));
    assert!(!items.contains(&ItemId::block(RED_CONCRETE)));
    assert!(!items.contains(&ItemId::block(BLUE_CONCRETE)));
    assert!(is_legacy_concrete(RED_CONCRETE) && is_legacy_concrete_item(ItemId::block(BLUE_CONCRETE)));
    assert_eq!(legacy_concrete_colour(RED_CONCRETE), Some(1));
    assert_eq!(legacy_concrete_colour(BLUE_CONCRETE), Some(6));

    // Chunk migration rewrites placed legacy dyed concrete to painted plain CONCRETE.
    let mut chunk = Chunk::uniform(block::AIR);
    chunk.set(2, 3, 4, RED_CONCRETE);
    chunk.set(3, 3, 4, BLUE_CONCRETE);
    chunk.migrate_legacy_concretes();
    assert_eq!(chunk.get(2, 3, 4), CONCRETE);
    assert_eq!(chunk.get_paint(2, 3, 4), all_faces(1));
    assert_eq!(chunk.get(3, 3, 4), CONCRETE);
    assert_eq!(chunk.get_paint(3, 3, 4), all_faces(6));

    // Placing a legacy dyed concrete item from an old save places painted CONCRETE.
    let mut sim = Sim::new(7, 2);
    let pos = IVec3::new(3, 40, 3);
    sim.world.set_block_anywhere(pos, block::AIR);
    sim.players[0].as_mut().unwrap().inventory.add(ItemId::block(BLUE_CONCRETE), 1);
    sim.queue(0, PlayerId(0), Action::PlaceBlock { pos, slot: 0, facing: 0, against: IVec3::ZERO });
    sim.step();
    assert_eq!(sim.world.block_anywhere_or_generate(pos), CONCRETE);
    assert_eq!(sim.world.paint_anywhere(pos), all_faces(6));
}

#[test]
fn glass_kiln_melts_glass_sand_and_quartz_with_pigment_into_stained_glass() {
    let mut sim = Sim::new(99, 2);
    sim.set_mode(Mode::Creative);
    let pos = IVec3::new(6, 40, 6);
    sim.world.set_block_anywhere(pos, block::AIR);
    sim.world.set_block_anywhere(pos - IVec3::new(0, 1, 0), block::STONE);
    assert!(sim.put_block(PlayerId(0), pos, GLASS_KILN_ITEM, 0, IVec3::ZERO));
    assert_eq!(sim.world.block_anywhere_or_generate(pos), GLASS_KILN);

    for (silica, qty, pigment, expected_block) in [
        (ItemId::block(GLASS), 4u32, PIGMENT_RED, STAINED_GLASS_FIRST),
        (ItemId::block(SAND), 4u32, PIGMENT_RED, STAINED_GLASS_FIRST),
        (ItemId::block(QUARTZ_ORE), 2u32, PIGMENT_BLUE, STAINED_GLASS_FIRST + 5),
    ] {
        // Put coal fuel + silica + pigment into the Glass Kiln.
        assert_eq!(sim.factory.insert(pos, ItemId::block(block::COAL_ORE), 4), 4);
        assert_eq!(sim.factory.insert(pos, silica, qty), qty);
        assert_eq!(sim.factory.insert(pos, pigment, 1), 1);
        for _ in 0..300 {
            sim.step();
        }
        let mut got = 0u32;
        sim.factory.take_contents(pos, |it, n| {
            if it == ItemId::block(expected_block) {
                got += n;
            }
            n
        });
        assert_eq!(
            got,
            4,
            "kiln should melt {} + {} into {}",
            item::name(silica),
            item::name(pigment),
            block::def(expected_block).name
        );
    }
}

#[test]
fn mesher_emits_per_face_paint_on_cubes_and_shaped_blocks() {
    let air = Chunk::uniform(block::AIR);
    let mut centre = Chunk::uniform(block::AIR);
    centre.set(10, 10, 10, CONCRETE);
    centre.set(11, 10, 10, CONCRETE);
    // Paint +Y face of (10,10,10) red (1) and +Y face of (11,10,10) blue (6): top faces must NOT merge.
    centre.set_paint(10, 10, 10, set_face(0, 2, 1));
    centre.set_paint(11, 10, 10, set_face(0, 2, 6));
    let light = vec![0x0Fu8; 34 * 34 * 34];
    let mut mesher = Mesher::new();
    let diff = {
        let mut chunks = [&air; 27];
        chunks[13] = &centre;
        mesher.mesh(&chunks, &light, None)
    };
    let paints: Vec<u8> = (0..diff.vertex_count()).map(|i| diff.paint(i)).collect();
    assert!(paints.contains(&1) && paints.contains(&6) && paints.contains(&0));

    // Paint both +Y faces red (1): they greedy-merge into one fewer quad.
    centre.set_paint(11, 10, 10, set_face(0, 2, 1));
    let same = {
        let mut chunks = [&air; 27];
        chunks[13] = &centre;
        mesher.mesh(&chunks, &light, None)
    };
    assert_eq!(same.opaque_quads, diff.opaque_quads - 1, "same-painted faces greedy-merge");

    // Shaped slab carries its face paint in w2 bits 27..32.
    centre.set(10, 12, 10, CONCRETE);
    centre.set_shape(10, 12, 10, crate::shape::SLAB_B2);
    centre.set_paint(10, 12, 10, set_face(0, 2, 5));
    let shaped = {
        let mut chunks = [&air; 27];
        chunks[13] = &centre;
        mesher.mesh(&chunks, &light, None)
    };
    let has_yellow_top = shaped.shaped.chunks_exact(3).any(|w| ((w[2] >> 27) & 31) == 5);
    assert!(has_yellow_top, "shaped slab top face carries yellow paint id in w2");
}
