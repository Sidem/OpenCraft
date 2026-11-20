//! Frame cost of a real save, piece by piece (ignored; needs a save):
//!   OC_SAVE=path\to\world.ocworld cargo test --release -q bench_save_frame -- --ignored --nocapture
//! Times each presentation step `Game::update` runs after the ticks, so a slow frame can be pinned on one of them.
//! `bench_port_view_and_labels` needs no save: the worst cases of the placement ghost, port marks and box labels
//!   cargo test --release -q bench_port_view_and_labels -- --ignored --nocapture

use std::time::Instant;

use super::run_until_ready;
use crate::factory;
use crate::Game;

fn ms<T>(name: &str, runs: u32, mut f: impl FnMut() -> T) {
    let t = Instant::now();
    for _ in 0..runs {
        std::hint::black_box(f());
    }
    println!("{name:34} {:8.3} ms", t.elapsed().as_secs_f64() * 1000.0 / f64::from(runs));
}

#[test]
#[ignore]
fn bench_save_frame() {
    let Ok(path) = std::env::var("OC_SAVE") else { panic!("set OC_SAVE to a .ocworld file") };
    let bytes = std::fs::read(path).expect("save");
    if bytes.starts_with(&[0x1f, 0x8b]) {
        panic!("unpack the save first: this bench reads raw save bytes");
    }
    let mut g = Game::from_save(&bytes, 8).expect("load");
    run_until_ready(&mut g);
    // Warm up static box lighting & wire routing like a settled frame.
    for _ in 0..80 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {}
        while g.next_static_box_event() != 0 {}
    }
    println!(
        "dynamic instances {} static {} chunks {}",
        g.instances.len() / factory::INSTANCE_FLOATS,
        g.static_box_total(),
        g.sim.world.loaded_count()
    );
    let eye = g.render_eye;
    let time = g.sim.tick as f64 / 60.0;
    let range = g.sim.world.view_distance();
    ms("update(1/60) whole", 20, || g.update(1.0 / 60.0));
    let mut evs = 0;
    while g.next_static_box_event() != 0 {
        evs += 1;
    }
    println!("  -> static events over 20 steady frames: {evs}");
    ms("wire_props.step", 20, || g.wire_props.step(&g.sim.world, &g.sim.factory, eye));
    let (strat, chg) = (g.strategy.active(), g.minimap.atlas.changes as usize);
    ms("static_boxes.update", 20, || {
        let atlas = &g.minimap.atlas;
        g.static_boxes.update(&g.sim.factory, &g.wire_props, &mut g.sim.world, eye, strat, chg, |cx, cz| {
            atlas.tile(cx, cz).is_some()
        });
    });
    ms("stream_around_players", 10, || g.stream_around_players());
    ms("write_item_instances", 10, || g.write_item_instances(1.0 / 60.0, eye));
    ms("write_drone_instances", 10, || g.write_drone_instances(eye, 0.5));
    let mut out = Vec::new();
    ms("factory.write_dynamic_instances", 20, || {
        out.clear();
        g.sim.factory.write_dynamic_instances(&g.static_boxes.index, &mut out, eye, time, range)
    });
    println!("  -> {} dynamic factory boxes", out.len() / factory::INSTANCE_FLOATS);
    ms("light_boxes (dynamic, warm)", 20, || factory::light_boxes(&mut out, eye, |c| g.sim.world.light_at(c)));
    let mut round = Vec::new();
    ms("split_round (dynamic)", 20, || factory::split_round(&out, &mut round));
    let mut full = Vec::new();
    ms("factory.write_instances (ref)", 10, || {
        full.clear();
        g.sim.factory.write_instances(&mut full, eye, time, range)
    });
    println!("  -> {} full factory boxes", full.len() / factory::INSTANCE_FLOATS);
    ms("hide_unexplored_instances", 10, || g.hide_unexplored_instances(eye));
    ms("write_strategy_fog", 10, || g.write_strategy_fog(eye));
    ms("refresh_some", 10, || g.minimap.atlas.refresh_some(&g.sim.world));
    ms("catch_up", 10, || g.catch_up());
    ms("write_line_preview", 10, || g.write_line_preview(eye, time));
}

#[test]
#[ignore]
fn bench_port_view_and_labels() {
    use crate::factory::{box_label, port_marks};
    use crate::math::{IVec3, Vec3};
    let (eye, time) = (Vec3::new(0.5, 66.0, 8.0), 3.0);
    let (mut solid, mut ghost) = (Vec::new(), Vec::new());
    // The ghost of the machine with the most ports, and the marks of 30 machines built near, a belt in hand.
    let most = (1..=255u16).max_by_key(|&b| port_marks::port_faces(b, IVec3::ZERO, 0).len()).unwrap();
    ms("ghost + marks, most ports", 1000, || {
        solid.clear();
        ghost.clear();
        port_marks::draw_ghost(most, IVec3::new(0, 64, 0), 1, eye, time, |_| true, &mut solid, &mut ghost)
    });
    println!(
        "  -> {} solid, {} see-through",
        solid.len() / factory::INSTANCE_FLOATS,
        ghost.len() / factory::INSTANCE_FLOATS
    );
    let mut world = crate::world::World::new(1, 2);
    let mut f = factory::Factory::default();
    let many: Vec<u16> = (1..=255u16).filter(|&b| !port_marks::port_faces(b, IVec3::ZERO, 0).is_empty()).collect();
    for i in 0..30 {
        let at = IVec3::new((i % 6) * 6 - 15, 64, (i / 6) * 6 - 15);
        f.place(&mut world, many[i as usize % many.len()], at, (i % 4) as u8, at, 0);
    }
    ms("port marks of 30 machines", 1000, || {
        solid.clear();
        ghost.clear();
        f.write_port_marks(eye, time, |_| true, &mut solid, &mut ghost)
    });
    // 100 full boxes of mixed items, each seen from a corner (two labelled sides).
    let slots: Vec<_> =
        (0..48).map(|i| crate::inventory::Stack { item: crate::item::ItemId(256 + i % 7), count: 30 }).collect();
    ms("100 box labels (48 mixed slots)", 1000, || {
        solid.clear();
        for i in 0..100 {
            let item = box_label::shown(&slots, box_label::AUTO).unwrap();
            box_label::draw(&mut solid, item, Vec3::new(3.0 + f64::from(i % 10), 0.0, -3.0 - f64::from(i / 10)));
        }
    });
    println!("  -> {} boxes", solid.len() / factory::INSTANCE_FLOATS);
}
