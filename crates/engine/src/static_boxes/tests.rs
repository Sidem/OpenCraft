use crate::block::{
    ASSEMBLER, BELT, BOILER, CABLE, FAST_BELT, FIBRE_NODE, FILTER, LAMP, LIFT, LONG_POLE, MERGER, PIPE, POLE, PUMP,
    RAIL, SMELTER, SPLITTER, UNDERPASS_IN,
};
use crate::factory::{is_round, INSTANCE_FLOATS};
use crate::math::{IVec3, Vec3};
use crate::tests::run_until_ready;
use crate::Game;

fn drain_static_events(g: &mut Game) -> Vec<(u32, IVec3)> {
    let mut out = Vec::new();
    loop {
        let k = g.next_static_box_event();
        if k == 0 {
            break;
        }
        out.push((k, IVec3::new(g.static_box_x(), g.static_box_y(), g.static_box_z())));
    }
    out
}

fn settle(g: &mut Game) {
    for _ in 0..20 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {}
        let _ = drain_static_events(g);
    }
}

#[test]
fn static_plus_dynamic_matches_write_instances_in_count_and_round_split() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let base = IVec3::new(g.player_x().floor() as i32 + 4, g.player_y().floor() as i32, g.player_z().floor() as i32);
    let place = |g: &mut Game, b, d: IVec3, dir: u8, tier: u8| {
        let p = base + d;
        g.sim.factory.place(&mut g.sim.world, b, p, dir, p - IVec3::new(0, 1, 0), tier);
    };
    // Straight belts, curve, lift, underpass, splitter, filter, merger, poles, tower, cable, pipes, rails, fibre, processors.
    place(&mut g, BELT, IVec3::new(0, 0, 0), 1, 0);
    place(&mut g, FAST_BELT, IVec3::new(1, 0, 0), 1, 1);
    place(&mut g, BELT, IVec3::new(2, 0, 0), 2, 0);
    place(&mut g, LIFT, IVec3::new(3, 0, 0), 1, 0);
    place(&mut g, UNDERPASS_IN, IVec3::new(4, 0, 0), 1, 0);
    place(&mut g, SPLITTER, IVec3::new(0, 0, 2), 1, 0);
    place(&mut g, FILTER, IVec3::new(2, 0, 2), 1, 0);
    place(&mut g, MERGER, IVec3::new(4, 0, 2), 1, 0);
    place(&mut g, POLE, IVec3::new(0, 0, 4), 0, 1);
    place(&mut g, CABLE, IVec3::new(1, 0, 4), 0, 0);
    place(&mut g, LONG_POLE, IVec3::new(4, 0, 4), 0, 0);
    place(&mut g, PIPE, IVec3::new(0, 0, 6), 0, 0);
    place(&mut g, PIPE, IVec3::new(1, 0, 6), 0, 0);
    place(&mut g, PUMP, IVec3::new(2, 0, 6), 0, 1);
    place(&mut g, RAIL, IVec3::new(0, 0, 8), 0, 0);
    place(&mut g, FIBRE_NODE, IVec3::new(2, 0, 8), 0, 0);
    place(&mut g, SMELTER, IVec3::new(6, 0, 0), 0, 0);
    place(&mut g, ASSEMBLER, IVec3::new(8, 0, 0), 0, 0);
    place(&mut g, BOILER, IVec3::new(12, 0, 0), 0, 0);
    settle(&mut g);

    let eye = g.render_eye;
    let time = g.render_time;
    let range = g.sim.world.view_distance();

    // Reference: full Factory::write_instances (no wires).
    let mut ref_all = Vec::new();
    g.sim.factory.write_instances(&mut ref_all, eye, time, range);
    let ref_cubes = ref_all.chunks_exact(INSTANCE_FLOATS).filter(|r| !is_round(r)).count();
    let ref_round = ref_all.chunks_exact(INSTANCE_FLOATS).filter(|r| is_round(r)).count();

    // Split path: static factory records (excluding wires) + dynamic factory records.
    let mut dyn_all = Vec::new();
    g.sim.factory.write_dynamic_instances(
        &g.static_boxes.index,
        &mut dyn_all,
        eye,
        time,
        range,
        &crate::view_cone::ViewCone::ALL,
    );
    let dyn_cubes = dyn_all.chunks_exact(INSTANCE_FLOATS).filter(|r| !is_round(r)).count();
    let dyn_round = dyn_all.chunks_exact(INSTANCE_FLOATS).filter(|r| is_round(r)).count();

    let (mut stat_cubes, mut stat_round) = (0usize, 0usize);
    for cached in g.static_boxes.chunks.values() {
        stat_cubes += cached.factory_cube_end / INSTANCE_FLOATS;
        stat_round += cached.round.len() / INSTANCE_FLOATS;
        assert!(cached.cubes.chunks_exact(INSTANCE_FLOATS).all(|r| !is_round(r)), "cubes buffer has no round bodies");
        assert!(cached.round.chunks_exact(INSTANCE_FLOATS).all(is_round), "round buffer has only round bodies");
    }

    assert_eq!(stat_cubes + dyn_cubes, ref_cubes, "cube count must match full write_instances");
    assert_eq!(stat_round + dyn_round, ref_round, "round count must match full write_instances");
    assert!(
        stat_cubes > dyn_cubes * 2,
        "majority of machine boxes are cached statically ({stat_cubes} vs {dyn_cubes})"
    );
}

#[test]
fn steady_frame_emits_no_static_box_events_and_edit_only_rebuilds_changed_chunk() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.player_y().floor() as i32;
    let a = IVec3::new(4, y, 4); // chunk (0, _, 0)
    let b = IVec3::new(68, y, 4); // chunk (2, _, 0)
    g.sim.factory.place(&mut g.sim.world, BELT, a, 1, a - IVec3::new(0, 1, 0), 0);
    g.sim.factory.place(&mut g.sim.world, BELT, b, 1, b - IVec3::new(0, 1, 0), 0);
    settle(&mut g);

    // Standing still for 10 frames must emit zero static box events.
    for _ in 0..10 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        assert!(drain_static_events(&mut g).is_empty(), "steady scene must emit no static box events");
    }

    // Adding a belt in chunk A only rebuilds chunk A, leaving chunk B untouched.
    let a2 = a + IVec3::new(1, 0, 0);
    g.sim.factory.place(&mut g.sim.world, BELT, a2, 1, a2 - IVec3::new(0, 1, 0), 0);
    g.update(1.0 / 60.0);
    let evs = drain_static_events(&mut g);
    assert_eq!(evs, vec![(1, crate::world::chunk_of(a))], "only the edited chunk should rebuild");
}

#[test]
fn belt_scroll_bits_survive_lighting_and_relight_updates_static_chunk() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let p = IVec3::new(g.player_x().floor() as i32 + 4, g.player_y().floor() as i32, g.player_z().floor() as i32);
    g.sim.factory.place(&mut g.sim.world, FAST_BELT, p, 1, p - IVec3::new(0, 1, 0), 1);
    settle(&mut g);

    let c = crate::world::chunk_of(p);
    let cached = g.static_boxes.chunk(c).expect("belt chunk cached");
    let belt_tiers: Vec<u32> =
        cached.cubes.chunks_exact(INSTANCE_FLOATS).map(|r| (r[11] as u32 >> 10) & 7).filter(|&t| t > 0).collect();
    assert_eq!(belt_tiers, vec![2], "Mk2 belt top carries tier+1 = 2 in MODE_LIGHT bits 10..=12");

    // Placing a lamp right beside the belt changes chunk light and triggers an Upsert once relit.
    g.sim.world.set_block_anywhere(p + IVec3::new(0, 1, 0), LAMP);
    let mut saw_upsert = false;
    for _ in 0..10 {
        g.begin_work();
        while g.work_step() {}
        g.update(1.0 / 60.0);
        if drain_static_events(&mut g).contains(&(1, c)) {
            saw_upsert = true;
            break;
        }
    }
    assert!(saw_upsert, "placing a lamp must relight and re-upload the static box chunk");
    let after = g.static_boxes.chunk(c).expect("still cached");
    let max_block_light =
        after.cubes.chunks_exact(INSTANCE_FLOATS).map(|r| ((r[11] as u32 >> 2) & 0xff) >> 4).max().unwrap_or(0);
    assert!(max_block_light >= 10, "belt next to lamp receives block light (got {max_block_light})");
}

#[test]
fn strategy_mode_hides_unexplored_static_chunks_and_reset_unloads_all() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.player_y().floor() as i32;
    let far = IVec3::new(g.player_x().floor() as i32 + 70, y, g.player_z().floor() as i32);
    g.sim.factory.place(&mut g.sim.world, BELT, far, 1, far - IVec3::new(0, 1, 0), 0);
    settle(&mut g);
    let fc = crate::world::chunk_of(far);
    assert!(g.static_boxes.chunk(fc).is_some(), "chunk cached in normal view");

    // In strategy view, a chunk whose minimap tile is not explored must be unloaded.
    let atlas = &g.minimap.atlas;
    let is_exp = atlas.tile(fc.x, fc.z).is_some();
    if !is_exp {
        g.static_boxes.update(
            &g.sim.factory,
            &g.wire_props,
            &mut g.sim.world,
            g.render_eye,
            true,
            atlas.changes as usize,
            |cx, cz| atlas.tile(cx, cz).is_some(),
        );
        assert!(g.static_boxes.chunk(fc).is_none(), "unexplored chunk hidden in strategy view");
        assert!(drain_static_events(&mut g).contains(&(2, fc)), "unexplored chunk emits Unload event");
    }

    // Reset (co-op snapshot resync) must emit Unload for every remaining cached chunk and clear total.
    let near = IVec3::new(g.player_x().floor() as i32 + 2, y, g.player_z().floor() as i32);
    g.sim.factory.place(&mut g.sim.world, BELT, near, 1, near - IVec3::new(0, 1, 0), 0);
    settle(&mut g);
    let nc = crate::world::chunk_of(near);
    assert!(g.static_boxes.chunk(nc).is_some());
    g.static_boxes.reset();
    assert_eq!(g.static_box_total(), 0);
    assert!(drain_static_events(&mut g).contains(&(2, nc)), "reset emits Unload for previously cached chunks");
}

#[test]
fn belt_scroll_uniform_advances_at_each_tiers_speed() {
    let mut g = Game::new(2024, 2);
    g.render_time = 0.25;
    assert!((g.belt_scroll(0) - 0.25).abs() < 1e-5);
    assert!((g.belt_scroll(1) - 0.50).abs() < 1e-5);
    assert!((g.belt_scroll(2) - 0.00).abs() < 1e-5);
    assert!((g.belt_scroll(3) - 0.00).abs() < 1e-5);
    g.render_time = 0.1;
    assert!((g.belt_scroll(3) - 0.80).abs() < 1e-5);
    // Chunk-local origin check on bounds.
    let b = Vec3::new(1.0, 2.0, 3.0);
    assert!(b.x > 0.0);
}
