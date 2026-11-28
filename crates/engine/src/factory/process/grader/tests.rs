use crate::block::{
    AIR, COAL_ORE, CONCRETE, DIRT, GENERATOR, GRASS, IRON_ORE, LEAVES, LOG, POLE, SITE_GRADER, STONE, WATER,
};
use crate::item::{ItemId, SITE_GRADER_ITEM};
use crate::math::IVec3;
use crate::tests::run_until_ready;
use crate::Game;

#[test]
fn site_grader_conserves_every_block_spares_or_mines_ores_and_paves_chosen_floor() {
    let mut g = Game::new(42, 3);
    // Mid-summer so the high-altitude test pond stays unfrozen while the grader fills it.
    g.sim.tick = crate::daytime::DAY_TICKS * crate::daytime::YEAR_DAYS * 3 / 8;
    run_until_ready(&mut g);
    let base_y = 90;
    // Flat stone foundation across the entire site.
    g.fill_blocks(-14, base_y - 6, -22, 14, base_y, 12, STONE);
    g.fill_blocks(-14, base_y + 1, -22, 14, base_y + 16, 12, AIR);

    // Build a 3×3×3 hill (9 Stone + 9 Dirt + 9 Grass = 27 earth blocks) with 1 Iron Ore inside, plus a 4-log oak tree.
    g.fill_blocks(2, base_y + 1, -4, 4, base_y + 1, -2, STONE);
    g.fill_blocks(2, base_y + 2, -4, 4, base_y + 2, -2, DIRT);
    g.fill_blocks(2, base_y + 3, -4, 4, base_y + 3, -2, GRASS);
    // Replace one stone with IRON_ORE -> 8 Stone, 18 Dirt/Grass, 1 Iron Ore.
    g.sim.world.set_block_anywhere(IVec3::new(3, base_y + 1, -3), IRON_ORE);
    g.fill_blocks(2, base_y + 7, -4, 4, base_y + 8, -2, LEAVES);
    for y in 4..=7 {
        g.sim.world.set_block_anywhere(IVec3::new(3, base_y + y, -3), LOG);
    }

    // Carve a 2×2×2 water pond (8 water cells: 4 below grade_y, 4 at grade_y).
    g.fill_blocks(-4, base_y - 1, -3, -3, base_y, -2, WATER);

    // Place Site Grader facing North (dir = 0) at (-1, base_y + 1, 4): site covers x=-9..=9, z=-16..=1.
    let grader_pos = IVec3::new(-1, base_y + 1, 4);
    g.place_at(grader_pos.x, grader_pos.y, grader_pos.z, SITE_GRADER, 0);
    // Power it with two coal generators + pole.
    g.place_at(4, base_y + 1, 4, GENERATOR, 0);
    g.place_at(6, base_y + 1, 4, GENERATOR, 0);
    g.place_at(3, base_y + 1, 3, POLE, 0);
    g.stock_at(4, base_y + 1, 4, COAL_ORE, 64);
    g.stock_at(6, base_y + 1, 4, COAL_ORE, 64);
    g.connect_at(3, base_y + 1, 3, 4, base_y + 1, 4);
    g.connect_at(3, base_y + 1, 3, 6, base_y + 1, 4);
    g.connect_at(3, base_y + 1, 3, grader_pos.x, grader_pos.y, grader_pos.z);

    // Phase 1: Natural floor (AIR) + Spare Ore Deposits (true), custom 19×18 area.
    g.sim.factory.set_grader(grader_pos, true, AIR, 19, 18, 0);
    g.skip_time(45.0);

    // Hill & tree cleared, EXCEPT the spared Iron Ore block at (3, base_y + 1, -3) MUST remain untouched!
    assert_eq!(g.sim.world.block_anywhere_or_generate(IVec3::new(3, base_y + 1, -3)), IRON_ORE, "ore spared");
    assert_eq!(g.sim.world.block_anywhere_or_generate(IVec3::new(2, base_y + 1, -2)), AIR, "hill stone cut");
    assert_eq!(g.sim.world.block_anywhere_or_generate(IVec3::new(3, base_y + 5, -3)), AIR, "tree felled");
    // Pond filled to solid earth at and below grade_y.
    assert_ne!(g.sim.world.block_anywhere_or_generate(IVec3::new(-3, base_y - 1, -2)), WATER);
    assert_ne!(g.sim.world.block_anywhere_or_generate(IVec3::new(-3, base_y, -2)), WATER);

    // Verify 100% block conservation after hopper flush!
    // 26 natural earth blocks cut (8 Stone + 18 Dirt/Grass) - 8 filled into pond = EXACTLY 18 earth blocks in out + hopper!
    let proc = g.sim.factory.processor_at(grader_pos, SITE_GRADER);
    let info = proc.grader_info().unwrap();
    let out_earth = proc.out.count(ItemId::block(DIRT)) + proc.out.count(ItemId::block(STONE));
    assert_eq!(info.hopper, 0, "hopper flushed completely once no hollows remain");
    assert_eq!(info.fill_count, 8, "all 8 pond cells filled");
    assert_eq!(out_earth, 26 - 8, "100% conservation: every unused dirt and stone block reached output");
    assert_eq!(proc.out.count(ItemId::block(LOG)), 4, "all 4 trunk logs preserved");
    assert_eq!(proc.out.count(ItemId::block(IRON_ORE)), 0, "spared ore was not mined");
    assert_eq!((info.paved_count, info.trees_felled), (0, 1));
    assert!(info.cut_count >= 30);

    // Phase 2: Switch to Mine Ores (spare_ores = false) + Concrete floor + custom shifted 11×10 area!
    g.sim.factory.set_grader(grader_pos, false, CONCRETE, 11, 10, 0);
    let loaded = g.stock_at(grader_pos.x, grader_pos.y, grader_pos.z, CONCRETE, 128);
    assert_eq!(loaded, 128, "grader accepts chosen floor material Concrete");
    // Rejects wrong paving material when Concrete is chosen!
    assert_eq!(g.stock_at(grader_pos.x, grader_pos.y, grader_pos.z, DIRT, 16), 0, "rejects non-chosen floor");
    g.skip_time(30.0);

    // Now the Iron Ore block IS mined into output, and Concrete is paved onto the grade surface within the 11×10 area!
    assert_eq!(g.sim.world.block_anywhere_or_generate(IVec3::new(3, base_y + 1, -3)), AIR, "ore mined when unspared");
    assert_eq!(g.sim.world.block_anywhere_or_generate(IVec3::new(-3, base_y, -2)), CONCRETE, "concrete paved");
    // Outside the narrower 11-wide area (`x = -8`), ground was NOT paved!
    assert_ne!(g.sim.world.block_anywhere_or_generate(IVec3::new(-8, base_y, 0)), CONCRETE, "outside custom width");
    let proc = g.sim.factory.processor_at(grader_pos, SITE_GRADER);
    assert_eq!(proc.out.count(ItemId::block(IRON_ORE)), 1, "mined iron ore delivered to output");
    let info2 = proc.grader_info().unwrap();
    assert_eq!((info2.width, info2.depth, info2.offset), (11, 10, 0));

    // Verify full save/load round-trip preserves Grader state & settings identically.
    let before = g.state_hash();
    let bytes = g.save();
    let loaded_game = Game::from_save(&bytes, 3).expect("save loads cleanly");
    assert_eq!(loaded_game.state_hash(), before, "grader state round-trips bit-identically");
    assert_eq!(ItemId::block(SITE_GRADER), SITE_GRADER_ITEM);
}

#[test]
fn setup_grader_demo_builds_and_saves_cleanly() {
    let mut g = Game::new(202604, 3);
    run_until_ready(&mut g);
    let msg = g.setup_grader_demo();
    assert!(msg.contains("Site Grader"), "{msg}");
    let has_grader = g.sim.factory.processors.iter().any(|p| p.spec.block == SITE_GRADER);
    assert!(has_grader, "demo must actually place the Site Grader processor");
    // Verify bird's-eye survey scan returns full 65×47 grid with hill, water, ore, and tree flags.
    let gpos = g.sim.factory.processors.iter().find(|p| p.spec.block == SITE_GRADER).unwrap().pos;
    let survey = g.grader_survey(gpos.x, gpos.y, gpos.z);
    assert_eq!(survey.len(), 8 + 65 * 47 * 5, "65x47 survey grid");
    assert_eq!((survey[0], survey[1]), (65, 47));
    let mut seen_flags = 0u8;
    for chunk in survey[8..].chunks_exact(5) {
        seen_flags |= chunk[4];
    }
    assert_ne!(seen_flags & 1, 0, "survey sees hill cut tiles");
    assert_ne!(seen_flags & 4, 0, "survey sees water pond");
    assert_ne!(seen_flags & 8, 0, "survey sees trees");
    assert_ne!(seen_flags & 16, 0, "survey sees ore deposits");
    // Let it work for 15 seconds and verify it actively grades & paves!
    g.skip_time(15.0);
    let grader = g.sim.factory.processors.iter().find(|p| p.spec.block == SITE_GRADER).unwrap();
    let text = grader.status_text();
    assert!(
        text.contains("cut") || text.contains("paved") || text.contains("filled"),
        "expected active grading: {text}"
    );
    let save = g.save();
    let loaded = Game::from_save(&save, 3).expect("demo save loads");
    assert_eq!(loaded.state_hash(), g.state_hash());
}

#[test]
fn legacy_v2_grader_save_with_12_output_slots_loads_cleanly() {
    use crate::bytes::ByteWriter;
    let mut g = Game::new(202604, 3);
    run_until_ready(&mut g);
    g.setup_grader_demo();
    let v3 = g.save();

    // Locate the v3 grader state (`STATE_MAGIC = 0xFFFF, STATE_VER = 3`) in the save bytes and rewrite the
    // Site Grader processor to the unreleased v2 layout (12 output slots + v2 grader bytes without magic).
    let marker = [0xFFu8, 0xFF, 3];
    let gpos = g.sim.factory.processors.iter().find(|p| p.spec.block == SITE_GRADER).unwrap().pos;
    let info = g.sim.factory.grader_info(gpos).unwrap();
    let needle_tail = {
        let mut w = ByteWriter::default();
        w.u16(0xFFFF);
        w.u8(3);
        w.u16(0); // col
        w.bytes
    };
    let magic_at = v3
        .windows(needle_tail.len())
        .position(|w| w == needle_tail.as_slice())
        .or_else(|| v3.windows(marker.len()).position(|w| w == marker))
        .expect("grader v3 magic in save");

    // Before `extra.write_state` are: progress(4) + burn(4) + next_out(4) + status(1) = 13 bytes,
    // immediately preceded by `out` (8 slots of 6 bytes each). Insert 4 empty output slots (24 zero bytes) right
    // after `out`, and replace the v3 grader payload with the v2 layout.
    let out_end = magic_at - 13;
    let proc = g.sim.factory.processor_at(gpos, SITE_GRADER);
    let mut w_v3 = ByteWriter::default();
    if let super::super::Extra::Grader(gr) = &proc.extra {
        gr.write_state(&mut w_v3);
    }
    let mut w_v2 = ByteWriter::default();
    if let super::super::Extra::Grader(gr) = &proc.extra {
        w_v2.u16(gr.col);
        w_v2.ivec3(gr.cur);
        w_v2.ivec3(gr.prev);
        w_v2.u8(gr.active);
        w_v2.bool(gr.spare_ores);
        w_v2.block(gr.floor);
        w_v2.u8(gr.hopper.len() as u8);
        for &(b, n) in &gr.hopper {
            w_v2.block(b);
            w_v2.u16(n);
        }
        w_v2.u32(gr.cut_count);
        w_v2.u32(gr.fill_count);
        w_v2.u32(gr.paved_count);
        w_v2.u32(gr.trees_felled);
        w_v2.u16(gr.rest);
        w_v2.bool(gr.pass_worked);
    }

    let mut v2 = Vec::with_capacity(v3.len() + 24);
    v2.extend_from_slice(&v3[..out_end]);
    v2[4..8].copy_from_slice(&92u32.to_le_bytes()); // legacy v2 layout was save version 92
    v2.extend_from_slice(&[0u8; 4 * 6]); // 4 extra empty output slots (12 total)
    v2.extend_from_slice(&v3[out_end..magic_at]);
    v2.extend_from_slice(&w_v2.bytes);
    v2.extend_from_slice(&v3[magic_at + w_v3.bytes.len()..]);

    let loaded = Game::from_save(&v2, 3).expect("legacy v2 grader save loads cleanly");
    let loaded_info = loaded.sim.factory.grader_info(gpos).expect("grader restored");
    assert_eq!(loaded_info.spare_ores, info.spare_ores);
    assert_eq!(loaded_info.floor, info.floor);
    assert_eq!((loaded_info.width, loaded_info.depth, loaded_info.offset), (19, 18, 0));
}
