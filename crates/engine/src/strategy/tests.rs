use super::*;
use crate::block::{AIR, STONE, WATER};
use crate::tests::run_until_ready;

fn flat() -> Game {
    let mut g = Game::new(1337, 2);
    run_until_ready(&mut g);
    for z in -4..=8 {
        for x in -4..=12 {
            g.sim.world.set_block_anywhere_later(IVec3::new(x, 200, z), STONE);
            for y in 201..=205 {
                g.sim.world.set_block_anywhere_later(IVec3::new(x, y, z), AIR);
            }
        }
    }
    g.body_mut().pos = Vec3::new(0.5, 201.0, 0.5);
    g.prev_eye = g.body().eye();
    g.switch_view(2);
    g
}

fn walk(g: &mut Game, goal: IVec3) {
    let start = g.body().pos.floor();
    let path = navigation::find_path(&g.sim.world, start, goal);
    assert!(path.is_some(), "route exists");
    g.strategy.navigation.order(path, goal);
    for _ in 0..1200 {
        g.update(TICK);
        if g.move_order_status() != 1 {
            break;
        }
    }
    assert_eq!(g.move_order_status(), 2, "normal physics reaches the destination");
    let target = goal.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
    assert!((g.body().pos - target).length() < 0.4);
}

#[test]
fn cycling_views_and_free_follow_shortcuts_preserve_the_body() {
    let mut g = Game::new(5, 2);
    let at = g.body().pos;
    for expected in [1, 2, 0] {
        g.cycle_view();
        assert_eq!(g.view_mode(), expected);
    }
    g.toggle_free_camera();
    assert_eq!(g.view_mode(), 3);
    g.toggle_free_camera();
    assert_eq!(g.view_mode(), 2);
    g.recenter_camera();
    assert_eq!(g.view_mode(), 2);
    assert_eq!(g.body().pos, at);
}

#[test]
fn height_changes_are_eased_and_limited_even_at_cliffs() {
    let mut s = Strategy { mode: 3, ground: 70.0, ..Strategy::default() };
    s.advance(Vec3::ZERO, Some(120.0), TICK);
    assert!((s.ground - 70.0).abs() <= HEIGHT_SPEED * TICK + 1e-8);
    for _ in 0..1200 {
        s.advance(Vec3::ZERO, Some(120.0), TICK);
    }
    assert!((s.ground - 120.0).abs() < 0.01);
    let previous = s.ground;
    s.advance(Vec3::ZERO, Some(30.0), TICK);
    assert!((s.ground - previous).abs() <= HEIGHT_SPEED * TICK + 1e-8);
    s.advance(Vec3::ZERO, None, TICK);
    assert!(s.ground.is_finite());
}

#[test]
fn cursor_rays_match_camera_projection_and_zoom_bounds() {
    let mut g = Game::new(5, 2);
    g.strategy.render_yaw = YAW;
    g.strategy.render_pitch = PITCH;
    g.strategy_cursor(0.0, 0.0, 1.25, 2.0);
    assert!((g.strategy.cursor_ray() - direction(YAW, PITCH)).length() < 1e-8);
    g.strategy_cursor(1.0, 0.0, 1.25, 2.0);
    let ray = g.strategy.cursor_ray();
    assert!(ray.x > 0.0 && ray.z > direction(YAW, PITCH).z);
    assert!((ray.length() - 1.0).abs() < 1e-8);
    g.zoom_strategy(-1000.0);
    assert_eq!(g.strategy.height, 12.0);
    g.zoom_strategy(1000.0);
    assert_eq!(g.strategy.height, 64.0);
}

#[test]
fn switching_to_overhead_moves_and_turns_gradually() {
    let mut g = Game::new(5, 2);
    let before = g.render_eye;
    g.switch_view(2);
    g.update(TICK);
    assert!((g.render_eye - before).length() < 5.0);
    assert!(g.camera_pitch() > PITCH + 0.5);
    for _ in 0..120 {
        g.update(TICK);
    }
    assert!((g.camera_pitch() - PITCH).abs() < 0.001);
}

#[test]
fn orders_walk_around_walls_using_the_real_body() {
    let mut g = flat();
    for z in -1..=1 {
        for y in 201..=203 {
            g.sim.world.set_block_anywhere_later(IVec3::new(3, y, z), STONE);
        }
    }
    walk(&mut g, IVec3::new(7, 201, 0));
}

#[test]
fn orders_jump_up_one_block_and_walk_down() {
    let mut g = flat();
    for z in -4..=8 {
        for x in 3..=6 {
            g.sim.world.set_block_anywhere_later(IVec3::new(x, 201, z), STONE);
        }
    }
    walk(&mut g, IVec3::new(5, 202, 0));
    walk(&mut g, IVec3::new(9, 201, 0));
}

#[test]
fn routes_reject_water_unloaded_ground_and_low_ceilings() {
    let mut g = flat();
    let goal = IVec3::new(7, 201, 0);
    g.sim.world.set_block_anywhere_later(goal, WATER);
    assert!(navigation::find_path(&g.sim.world, g.body().pos.floor(), goal).is_none());
    g.sim.world.set_block_anywhere_later(goal, AIR);
    g.sim.world.set_block_anywhere_later(goal + IVec3::new(0, 1, 0), STONE);
    assert!(navigation::find_path(&g.sim.world, g.body().pos.floor(), goal).is_none());
    assert!(navigation::find_path(&g.sim.world, g.body().pos.floor(), IVec3::new(5000, 201, 0)).is_none());
}

#[test]
fn changing_terrain_stops_a_route_safely_and_direct_view_cancels_it() {
    let mut g = flat();
    let goal = IVec3::new(7, 201, 0);
    let path = navigation::find_path(&g.sim.world, g.body().pos.floor(), goal);
    g.strategy.navigation.order(path, goal);
    g.sim.world.set_block_anywhere_later(IVec3::new(1, 201, 0), STONE);
    g.update(TICK);
    assert_eq!(g.move_order_status(), 4);
    assert_eq!(g.body().input.forward, 0.0);
    g.switch_view(0);
    assert_eq!(g.move_order_status(), 0);
}

#[test]
fn free_panning_never_moves_the_character_or_explores_new_columns() {
    let mut g = flat();
    for _ in 0..100 {
        g.minimap.atlas.refresh_some(&g.sim.world);
    }
    let known = g.minimap.atlas.len();
    let body = g.body().pos;
    g.pan_camera(1.0, 0.0, true);
    for _ in 0..600 {
        g.update(TICK);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {}
    }
    for _ in 0..100 {
        g.minimap.atlas.refresh_some(&g.sim.world);
    }
    assert!((g.body().pos - body).length() < 0.01);
    assert_eq!(g.minimap.atlas.len(), known);
    assert!((g.strategy.focus - body).length() > 400.0);
}

#[test]
fn free_view_reloads_known_terrain_without_extending_the_map() {
    let mut g = flat();
    for _ in 0..100 {
        g.minimap.atlas.refresh_some(&g.sim.world);
    }
    let original = g.explored_map();
    g.body_mut().pos = Vec3::new(512.5, 201.0, 0.5);
    g.prev_eye = g.body().eye();
    g.switch_view(0);
    run_until_ready(&mut g);
    for _ in 0..100 {
        g.minimap.atlas.refresh_some(&g.sim.world);
    }
    let known = g.minimap.atlas.len();
    assert!(g.sim.world.get_block(IVec3::new(0, 200, 0)).is_none());
    g.switch_view(3);
    g.strategy.focus = Vec3::new(0.5, 201.0, 0.5);
    g.update(TICK);
    g.begin_work();
    while g.work_step() {}
    while g.next_event() != 0 {}
    for _ in 0..100 {
        g.minimap.atlas.refresh_some(&g.sim.world);
    }
    assert_eq!(g.sim.world.get_block(IVec3::new(0, 200, 0)), Some(STONE));
    assert_eq!(g.minimap.atlas.len(), known);
    assert!(!original.is_empty());
    // Map queries over generation buffers must not discover them either.
    g.world_map_draw(-192, -192, 16, 64, 64);
    assert_eq!(g.minimap.atlas.len(), known);
}

#[test]
fn overhead_mining_keeps_body_reach_and_line_of_sight() {
    let mut g = flat();
    g.strategy.render_yaw = 0.0;
    g.strategy.render_pitch = -std::f64::consts::FRAC_PI_2;
    g.strategy_cursor(0.0, 0.0, 1.25, 1.0);
    g.render_eye = Vec3::new(0.5, 225.0, 0.5);
    g.strategy_target();
    assert_eq!(g.target.unwrap().block, IVec3::new(0, 200, 0));
    g.render_eye.x = 9.5;
    g.strategy_target();
    assert!(g.target.is_none(), "camera cannot increase reach");
}

#[test]
fn terrain_height_ignores_trees_and_factory_roofs_and_zoom_is_eased() {
    let mut g = flat();
    for (x, z) in [(0, 0), (-2, 0), (2, 0), (0, -2), (0, 2)] {
        g.sim.world.set_block_anywhere_later(IVec3::new(x, 204, z), block::LOG);
        g.sim.world.set_block_anywhere_later(IVec3::new(x, 205, z), block::LEAVES);
        g.sim.world.set_block_anywhere_later(IVec3::new(x, 206, z), block::MINER);
    }
    assert_eq!(navigation::camera_ground(&g.sim.world, g.body().pos), Some(201.0));
    let before = g.strategy.eye();
    g.zoom_strategy(4.0);
    assert_eq!(g.strategy.eye(), before, "wheel input does not snap the camera");
    g.strategy.advance(g.body().pos, Some(201.0), TICK);
    assert!(g.strategy.zoom > 24.0 && g.strategy.zoom < 26.0);
}

#[test]
fn camera_controls_do_not_change_core_state_and_shoulder_setting_survives_strategy() {
    let mut g = Game::new(1337, 2);
    let before = g.state_hash();
    g.set_view_mode(3);
    g.pan_camera(1.0, 1.0, true);
    g.zoom_strategy(5.0);
    g.strategy_cursor(0.4, -0.2, 1.5, 1.7);
    g.set_shoulder_distance(7.0);
    assert_eq!(g.state_hash(), before);
    g.set_view_mode(1);
    assert_eq!(g.third_person.distance, 7.0);
}

#[test]
fn unknown_strategy_cover_has_no_terrain_information() {
    let mut g = flat();
    g.strategy.focus = Vec3::new(4096.5, 201.0, 4096.5);
    assert!(!g.camera_area_known());
    g.instances.clear();
    g.write_strategy_fog(Vec3::ZERO);
    let stride = crate::factory::INSTANCE_FLOATS;
    let first_y = g.instances[1];
    for cell in g.instances.chunks_exact(stride) {
        assert_eq!(cell[1], first_y);
        assert_eq!(cell[8], f32::from(block::tex::STRATEGY_FOG));
    }
    assert!(!g.instances.is_empty());
}
