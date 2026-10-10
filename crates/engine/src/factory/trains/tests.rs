use super::*;
use crate::block::{LOADING_DOCK, RAIL, UNLOADING_DOCK};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::render::INSTANCE_FLOATS;
use crate::factory::tests::run;
use crate::inventory::Stack;
use crate::item::{IRON_INGOT, LOCOMOTIVE, WAGON};
use crate::math::Vec3;
use crate::world::World;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

const N: u8 = 0;
const E: u8 = 64;

/// A factory with rail nodes (cell, heading) and tracks laid between the listed pairs.
fn line(nodes: &[(IVec3, u8)], pairs: &[(usize, usize)]) -> Factory {
    let mut f = Factory::default();
    for &(c, yaw) in nodes {
        f.place(&mut World::new(1, 2), RAIL, c, yaw, c, 0);
    }
    for &(a, b) in pairs {
        f.lay_track(nodes[a].0, nodes[b].0);
    }
    assert_eq!(f.tracks.len(), pairs.len(), "every track fits");
    f
}

fn head(f: &Factory) -> Vec3 {
    f.head_point(&f.trains[0]).expect("on the track")
}

/// What must always hold of a train: its edges are laid and join end to end, its head is inside the last one and
/// the path covers the whole train.
fn sound(f: &Factory) {
    let t = &f.trains[0];
    assert!(t.path.iter().all(|&(a, b)| f.track_between(a, b)), "{:?}", t.path);
    assert!(t.path.windows(2).all(|w| w[0].1 == w[1].0), "{:?}", t.path);
    let &(a, b) = t.path.last().unwrap();
    assert!((0..=f.edge_mm(a, b).unwrap()).contains(&t.head));
    assert!(f.tail_room(t).unwrap() >= 0, "the tail is off the path: {:?}", t.path);
    assert_eq!(t.cargo.slots.len(), usize::from(t.cars) * CAR_SLOTS);
}

const AB: [(IVec3, u8); 3] = [(IVec3::new(0, 0, 0), E), (IVec3::new(12, 0, 0), E), (IVec3::new(24, 0, 0), E)];

/// A line of three nodes, a loading dock by the first (fed from a box of `n` iron ingots by a belt) and an
/// unloading dock by the last (into a box through a belt), and a locomotive with `cars` wagons at the first node.
fn freight(n: u32, cars: u8) -> Factory {
    let mut f = line(&AB, &[(0, 1), (1, 2)]);
    let mut world = World::new(1, 2);
    f.place(&mut world, LOADING_DOCK, v(0, 0, 3), 0, v(0, 0, 3), 0);
    f.add_storage(v(0, 0, 5));
    f.stock(v(0, 0, 5), IRON_INGOT, n);
    f.add_belt(v(0, 0, 4), 0);
    f.place(&mut world, UNLOADING_DOCK, v(22, 0, 3), 0, v(22, 0, 3), 0);
    f.add_belt(v(22, 0, 4), 2);
    f.add_storage(v(22, 0, 5));
    assert!(f.place_train(v(0, 0, 0)));
    for _ in 0..cars {
        assert!(f.couple(v(0, 0, 0)), "couples at the first node");
    }
    f
}

#[test]
fn wagons_couple_on_behind_up_to_six_and_the_train_stays_sound() {
    let mut f = line(&AB, &[(0, 1), (1, 2)]);
    assert_eq!(f.couple_spot(v(0, 0, 0)), Spot::NoTrain);
    assert!(f.place_train(v(0, 0, 0)));
    for n in 1..=MAX_CARS {
        assert_eq!(f.couple_spot(v(0, 0, 0)), Spot::Ok, "wagon {n}");
        assert!(f.couple(v(0, 0, 0)));
        sound(&f);
        assert_eq!(f.trains[0].cars, n);
    }
    assert_eq!(f.couple_spot(v(0, 0, 0)), Spot::Full);
    assert!(!f.couple(v(0, 0, 0)));
    for _ in 0..3000 {
        f.step_trains();
        sound(&f);
    }
    assert_eq!(f.take_train(v(0, 0, 0)).map(|s| s.len()), Some(2), "a locomotive and its wagons, no cargo");
}

#[test]
fn a_wagon_has_to_stand_on_track() {
    let mut f = line(&[(v(0, 0, 0), E), (v(12, 0, 0), E)], &[(0, 1)]);
    assert!(f.place_train(v(0, 0, 0)));
    let mut cars = 0;
    while f.couple(v(0, 0, 0)) {
        cars += 1;
        sound(&f);
    }
    assert_eq!(cars, 3, "12 blocks of track hold a locomotive and three wagons");
    assert_eq!(f.couple_spot(v(0, 0, 0)), Spot::NoRoom);
}

#[test]
fn a_wagon_goes_on_behind_when_there_is_track_behind() {
    let mut f = line(&AB, &[(0, 1), (1, 2)]);
    assert!(f.place_train(v(12, 0, 0)));
    let head = f.trains[0].head;
    assert!(f.couple(v(12, 0, 0)));
    assert_eq!(f.trains[0].head, head, "the locomotive stays where it is");
    assert_eq!(f.trains[0].path.len(), 2, "the track behind it is put under the wagon");
    sound(&f);
}

#[test]
fn ore_goes_from_a_box_by_belt_into_a_train_across_the_line_and_out_by_belt_into_a_box() {
    let mut f = freight(40, 2);
    let mut stops = 0;
    run(&mut f, 300.0, |f| {
        stops += f.trains[0].idle.is_some() as u32;
        let held = f.trains[0].cargo.count(IRON_INGOT)
            + f.processor_at(v(0, 0, 3), LOADING_DOCK).out.count(IRON_INGOT)
            + f.processor_at(v(22, 0, 3), UNLOADING_DOCK).out.count(IRON_INGOT)
            + f.storage_count_at(v(0, 0, 5), IRON_INGOT)
            + f.storage_count_at(v(22, 0, 5), IRON_INGOT)
            + f.belts.iter().map(|b| b.items.len() as u32).sum::<u32>();
        assert_eq!(held, 40, "no ingot appears or disappears");
    });
    assert!(stops > 0, "the train stopped at the docks");
    assert_eq!(f.storage_count_at(v(22, 0, 5), IRON_INGOT), 40, "all of it arrived");
    sound(&f);
}

#[test]
fn a_bare_locomotive_does_not_stop_at_docks() {
    let mut f = freight(10, 0);
    run(&mut f, 60.0, |f| assert!(f.trains[0].idle.is_none()));
}

#[test]
fn a_train_waits_five_idle_seconds_at_each_dock_and_shuttles_between_them() {
    let mut f = freight(0, 1);
    let mut ticks = Vec::new();
    let mut at = 0;
    run(&mut f, 40.0, |f| {
        at += 1;
        if f.trains[0].idle == Some(0) {
            ticks.push(at);
        }
    });
    assert!(ticks.len() >= 3, "it shuttles between the docks: {ticks:?}");
    assert!(ticks.windows(2).all(|w| w[1] - w[0] > 5 * 60), "five seconds at each stop: {ticks:?}");
}

#[test]
fn picking_a_train_up_gives_back_the_locomotive_wagons_and_cargo() {
    let mut f = freight(40, 2);
    f.trains[0].cargo.add(IRON_INGOT, 100);
    let stacks = f.take_train(v(0, 0, 0)).expect("a train is near");
    assert_eq!(stacks[0], Stack { item: LOCOMOTIVE, count: 1 });
    assert_eq!(stacks[1], Stack { item: WAGON, count: 2 });
    assert_eq!(stacks[2..].iter().map(|s| s.count).sum::<u32>(), 100);
    // The same when the track goes.
    let mut f = freight(0, 1);
    f.trains[0].cargo.add(IRON_INGOT, 5);
    let given = f.remove(v(12, 0, 0));
    assert!(given.contains(&Stack { item: WAGON, count: 1 }) && given.contains(&Stack { item: IRON_INGOT, count: 5 }));
}

#[test]
fn wagons_cargo_and_a_stop_save_and_load() {
    let mut f = freight(40, 2);
    run(&mut f, 70.0, |_| {});
    f.trains[0].cargo.add(IRON_INGOT, 7);
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).expect("reads back");
    let same = |a: &Factory, b: &Factory| {
        let (x, y) = (&a.trains[0], &b.trains[0]);
        assert_eq!((x.cars, x.head, x.idle, &x.path), (y.cars, y.head, y.idle, &y.path));
        assert_eq!(x.cargo.contents(), y.cargo.contents());
    };
    same(&f, &back);
    run(&mut f, 20.0, |_| {});
    run(&mut back, 20.0, |_| {});
    same(&f, &back);
}

#[test]
fn locomotives_from_a_version_32_save_load_without_wagons() {
    let mut f = line(&AB, &[(0, 1), (1, 2)]);
    let mut w = ByteWriter::default();
    w.count(1);
    w.count(1);
    w.ivec3(v(0, 0, 0));
    w.ivec3(v(12, 0, 0));
    w.u32(2500);
    let mut r = ByteReader::new(&w.bytes);
    r.version = 32;
    f.read_trains(&mut r).expect("reads");
    assert_eq!((f.trains.len(), f.trains[0].cars, f.trains[0].idle), (1, 0, None));
}

/// A fork: nodes 0 - 1, then 1 to 2 (left) and 1 to 3 (right); a dock beside each of 0, 2 and 3; a locomotive and one
/// wagon at node 0. Returns the factory and the docks' cells (by node 0, 2, 3).
fn fork() -> (Factory, [IVec3; 3]) {
    let nodes = [(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, -4), E), (v(24, 0, 6), E)];
    let mut f = line(&nodes, &[(0, 1), (1, 2), (1, 3)]);
    let docks = [v(0, 0, 3), v(22, 0, -1), v(22, 0, 9)];
    let mut world = World::new(1, 2);
    for d in docks {
        f.place(&mut world, UNLOADING_DOCK, d, 0, d, 0);
    }
    assert!(f.place_train(v(0, 0, 0)) && f.couple(v(0, 0, 0)));
    (f, docks)
}

/// The nodes a train stopped at, in order, over `seconds`.
fn stops(f: &mut Factory, seconds: f64) -> Vec<IVec3> {
    let mut seen = Vec::new();
    run(f, seconds, |f| {
        if f.trains[0].idle == Some(0) {
            seen.push(f.trains[0].path.last().unwrap().1);
        }
    });
    seen
}

#[test]
fn a_schedule_sends_a_train_to_its_docks_in_order_across_a_junction() {
    let (mut f, docks) = fork();
    for d in [docks[1], docks[2], docks[0]] {
        assert!(f.set_stop(v(0, 0, 0), d, false));
    }
    let seen = stops(&mut f, 260.0);
    let (n0, n2, n3) = (v(0, 0, 0), v(24, 0, -4), v(24, 0, 6));
    assert!(seen.len() >= 6, "{seen:?}");
    assert_eq!(seen[..6], [n2, n3, n0, n2, n3, n0], "left branch, right branch, the start, round again");
}

#[test]
fn a_train_passes_the_docks_that_are_not_next_on_its_schedule() {
    let (mut f, docks) = fork();
    assert!(f.set_stop(v(0, 0, 0), docks[2], false));
    let seen = stops(&mut f, 120.0);
    assert!(!seen.is_empty() && seen.iter().all(|&n| n == v(24, 0, 6)), "only the right branch's dock: {seen:?}");
}

#[test]
fn schedules_are_edited_by_dock_and_limited() {
    let (mut f, docks) = fork();
    assert!(!f.set_stop(v(0, 0, 0), v(5, 0, 5), false), "no dock there");
    assert!(!f.set_stop(v(24, 0, 6), docks[0], false), "needs a train near the node");
    assert!(f.set_stop(v(0, 0, 0), v(23, 0, -2), false), "any cell of the dock names it");
    assert_eq!(f.schedule_near(v(0, 0, 0)), vec![docks[1]]);
    assert!(!f.set_stop(v(0, 0, 0), docks[1], false), "not twice in a row");
    for i in 0..10 {
        f.set_stop(v(0, 0, 0), docks[i % 2], false);
    }
    assert_eq!(f.schedule_near(v(0, 0, 0)).len(), schedule::MAX_STOPS);
    assert!(f.set_stop(v(0, 0, 0), docks[0], true));
    assert!(f.schedule_near(v(0, 0, 0)).is_empty());
}

#[test]
fn a_dock_that_is_gone_is_skipped_and_the_schedule_saves() {
    let (mut f, docks) = fork();
    for d in [docks[1], docks[2]] {
        f.set_stop(v(0, 0, 0), d, false);
    }
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!(back.trains[0].schedule, vec![docks[1], docks[2]]);
    f.remove(docks[1]);
    let seen = stops(&mut f, 120.0);
    assert!(!seen.is_empty() && seen.iter().all(|&n| n == v(24, 0, 6)), "the lost dock is skipped: {seen:?}");
}

#[test]
fn wagons_are_drawn_too() {
    let mut f = freight(0, 2);
    let (mut bare, mut full) = (Vec::new(), Vec::new());
    f.trains[0].cars = 0;
    f.write_train_models(&mut bare, Vec3::new(5.0, 3.0, 5.0), 0.0, 60.0);
    f.trains[0].cars = 2;
    f.write_train_models(&mut full, Vec3::new(5.0, 3.0, 5.0), 0.0, 60.0);
    assert!(full.len() >= bare.len() + 2 * 9 * INSTANCE_FLOATS, "{} vs {}", full.len(), bare.len());
}

#[test]
fn a_locomotive_runs_to_the_end_of_the_track_turns_round_and_comes_back() {
    let mut f = line(&[(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, 0), E)], &[(0, 1), (1, 2)]);
    assert_eq!(f.train_spot(v(0, 0, 0)), Spot::Ok);
    assert!(f.place_train(v(0, 0, 0)));
    assert!((head(&f).x - 3.0).abs() < 1e-6, "its rear is on the node, the front a locomotive on");
    let mut xs = Vec::new();
    for _ in 0..900 {
        f.step_trains();
        sound(&f);
        xs.push(head(&f).x);
    }
    let (lo, hi) = xs.iter().fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
    assert!(hi > 24.4 && hi < 24.6, "the front reaches the last node and no further: {hi}");
    assert!(lo > 0.4 && lo < 3.2, "and comes back to the first: {lo}");
    let turns = xs.windows(3).filter(|w| (w[1] - w[0]) * (w[2] - w[1]) < 0.0).count();
    assert!(turns >= 2, "it turned round at both ends ({turns})");
    // 150 mm a tick, 9 blocks a second.
    assert!((xs[1] - xs[0] - 0.15).abs() < 1e-3);
}

#[test]
fn on_a_bend_it_keeps_its_speed() {
    let mut f = line(&[(v(0, 0, 0), N), (v(16, 0, -16), E)], &[(0, 1)]);
    assert!(f.place_train(v(0, 0, 0)));
    let mut last = head(&f);
    for _ in 0..60 {
        f.step_trains();
        let now = head(&f);
        let step = (now - last).length();
        assert!((step - 0.15).abs() < 0.01, "moved {step}");
        last = now;
    }
}

#[test]
fn at_a_junction_it_takes_the_straightest_track() {
    // 0 -- 1 forks to 2 (slightly left) and 3 (more to the right).
    let nodes = [(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, -4), E), (v(24, 0, 6), E)];
    let mut f = line(&nodes, &[(0, 1), (1, 2), (1, 3)]);
    assert!(f.place_train(v(0, 0, 0)));
    for _ in 0..120 {
        f.step_trains();
    }
    sound(&f);
    assert!(f.trains[0].path.contains(&(v(12, 0, 0), v(24, 0, -4))), "{:?}", f.trains[0].path);
    assert!(!f.trains[0].path.iter().any(|e| e.1 == v(24, 0, 6)));
}

#[test]
fn the_tail_edges_are_dropped_and_a_long_run_stays_sound() {
    let mut f =
        line(&[(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, 0), E), (v(36, 0, 0), E)], &[(0, 1), (1, 2), (2, 3)]);
    assert!(f.place_train(v(0, 0, 0)));
    for _ in 0..3000 {
        f.step_trains();
        sound(&f);
        assert!(f.trains[0].path.len() <= 2, "a locomotive covers at most two edges of 12 blocks");
    }
}

#[test]
fn placing_taking_and_what_stands_in_the_way() {
    let mut f = line(&[(v(0, 0, 0), E), (v(12, 0, 0), E), (v(40, 0, 5), E)], &[(0, 1)]);
    assert_eq!(f.train_spot(v(40, 0, 5)), Spot::NoTrack, "no track");
    assert_eq!(f.train_spot(v(3, 3, 3)), Spot::NoTrack, "no node");
    assert!(!f.place_train(v(40, 0, 5)));
    assert!(f.place_train(v(0, 0, 0)));
    assert_eq!(f.train_spot(v(0, 0, 0)), Spot::Occupied);
    assert!(!f.place_train(v(0, 0, 0)), "one train to a node at a time");
    assert!(f.train_on(v(12, 0, 0), v(0, 0, 0)));
    f.cut_track(v(0, 0, 0), v(12, 0, 0));
    assert!(f.track_between(v(0, 0, 0), v(12, 0, 0)), "a track with a train on it cannot be cut");
    assert!(f.take_train(v(40, 0, 5)).is_none(), "no train near");
    assert!(f.take_train(v(30, 0, 0)).is_none(), "too far from the node it was aimed at");
    assert_eq!(f.take_train(v(0, 0, 0)), Some(vec![Stack { item: LOCOMOTIVE, count: 1 }]), "its front is near");
    assert!(f.trains.is_empty());
    f.cut_track(v(0, 0, 0), v(12, 0, 0));
    assert!(!f.track_between(v(0, 0, 0), v(12, 0, 0)));
}

#[test]
fn removing_a_node_under_a_train_gives_the_locomotive_back() {
    let mut f = line(&[(v(0, 0, 0), E), (v(12, 0, 0), E)], &[(0, 1)]);
    assert!(f.place_train(v(0, 0, 0)));
    let given = f.remove(v(12, 0, 0));
    assert_eq!(given, vec![Stack { item: LOCOMOTIVE, count: 1 }]);
    assert!(f.trains.is_empty() && f.tracks.is_empty());
}

#[test]
fn trains_save_and_load_where_they_were() {
    let mut f = line(&[(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, 0), E)], &[(0, 1), (1, 2)]);
    assert!(f.place_train(v(0, 0, 0)));
    for _ in 0..100 {
        f.step_trains();
    }
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!((back.trains[0].path.clone(), back.trains[0].head), (f.trains[0].path.clone(), f.trains[0].head));
    // Both then run on the same.
    for _ in 0..200 {
        f.step_trains();
        back.step_trains();
    }
    assert_eq!((back.trains[0].path.clone(), back.trains[0].head), (f.trains[0].path.clone(), f.trains[0].head));
}

#[test]
fn a_locomotive_draws_whole_boxes_within_range_only() {
    let mut f = line(&[(v(0, 0, 0), E), (v(12, 0, 0), E)], &[(0, 1)]);
    assert!(f.place_train(v(0, 0, 0)));
    let (mut near, mut far) = (Vec::new(), Vec::new());
    f.write_train_models(&mut near, Vec3::new(5.0, 3.0, 5.0), 0.0, 40.0);
    f.write_train_models(&mut far, Vec3::new(500.0, 3.0, 5.0), 0.0, 40.0);
    assert!(near.len() >= 8 * INSTANCE_FLOATS && near.len() % INSTANCE_FLOATS == 0);
    assert!(far.is_empty());
}

/// A passing loop: a line 0 - 1, the loop 1 - 2 - 4 (upper) and 1 - 3 - 4 (lower), then 4 - 5. A locomotive at each
/// end. With `signals` the loop's two ends (nodes 1 and 4) carry signals.
fn loop_line(signals: bool) -> Factory {
    let nodes =
        [(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, -8), E), (v(24, 0, 8), E), (v(36, 0, 0), E), (v(48, 0, 0), E)];
    let mut f = line(&nodes, &[(0, 1), (1, 2), (1, 3), (2, 4), (3, 4), (4, 5)]);
    if signals {
        assert!(f.set_signal(nodes[1].0, true) && f.set_signal(nodes[4].0, true));
    }
    assert!(f.place_train(nodes[0].0) && f.place_train(nodes[5].0));
    f
}

/// Whether the two trains cover a track in common.
fn share_track(f: &Factory) -> bool {
    let (a, b) = (&f.trains[0], &f.trains[1]);
    a.path.iter().any(|&(p, q)| b.path.iter().any(|&(r, s)| (p, q) == (r, s) || (p, q) == (s, r)))
}

#[test]
fn two_trains_on_a_line_with_a_passing_loop_pass_without_meeting_when_it_has_signals() {
    let mut f = loop_line(true);
    let (mut passed, mut met) = (false, false);
    run(&mut f, 400.0, |f| {
        met |= share_track(f);
        let (a, b) = (head(f).x, f.head_point(&f.trains[1]).unwrap().x);
        passed |= a > b + 3.0;
    });
    assert!(!met, "the trains never share a track");
    assert!(passed, "the one from the west gets past the one from the east");
}

#[test]
fn the_same_line_without_signals_has_the_trains_meet() {
    let mut f = loop_line(false);
    let mut met = false;
    run(&mut f, 400.0, |f| met |= share_track(f));
    assert!(met);
}

#[test]
fn trains_wait_at_a_signal_for_the_track_ahead_and_drive_on_when_it_clears() {
    // A line 0 - 1 - 2 with a signal at 1 and a train at each end: each would enter the stretch the other is in, so
    // both stop at the signal (a layout without a loop can deadlock).
    let nodes = [(v(0, 0, 0), E), (v(12, 0, 0), E), (v(24, 0, 0), E)];
    let mut f = line(&nodes, &[(0, 1), (1, 2)]);
    f.set_signal(nodes[1].0, true);
    assert!(f.place_train(nodes[0].0) && f.place_train(nodes[2].0));
    run(&mut f, 30.0, |_| {});
    for t in &f.trains {
        let &(a, b) = t.path.last().unwrap();
        assert!(b == nodes[1].0 && Some(t.head) == f.edge_mm(a, b), "held at the signal: {:?} {}", t.path, t.head);
    }
    // One is taken away: the other drives on through the signal.
    f.trains.remove(0);
    let mut through = false;
    run(&mut f, 20.0, |f| through |= f.trains[0].path.contains(&(nodes[1].0, nodes[0].0)));
    assert!(through);
}

#[test]
fn signals_are_put_taken_back_and_saved() {
    let nodes = [(v(0, 0, 0), E), (v(12, 0, 0), E)];
    let mut f = line(&nodes, &[(0, 1)]);
    assert!(!f.set_signal(v(5, 0, 5), true), "only a rail node takes one");
    assert!(f.set_signal(nodes[0].0, true) && !f.set_signal(nodes[0].0, true));
    assert!(f.signal_at(nodes[0].0) && !f.signal_at(nodes[1].0));
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).expect("reads back");
    assert!(back.signal_at(nodes[0].0) && !back.signal_at(nodes[1].0));
    // Taking the node gives the signal back.
    assert_eq!(f.remove(nodes[0].0), vec![Stack { item: crate::item::RAIL_SIGNAL, count: 1 }]);
}

/// `cargo test --release bench_trains -- --ignored --nocapture`: 14 trains with wagons on one 60-node line (720
/// blocks), once without signals and once with a signal on every node, the worst case for sections and routing.
#[test]
#[ignore]
fn bench_trains() {
    let nodes: Vec<(IVec3, u8)> = (0..60).map(|i| (v(i * 12, 0, 0), E)).collect();
    let pairs: Vec<(usize, usize)> = (0..59).map(|i| (i, i + 1)).collect();
    for signals in [false, true] {
        let mut f = line(&nodes, &pairs);
        for i in (2..58).step_by(4) {
            assert!(f.place_train(nodes[i].0));
            while f.couple(nodes[i].0) {}
        }
        if signals {
            nodes.iter().for_each(|n| assert!(f.set_signal(n.0, true)));
        }
        let (mut total, mut worst) = (std::time::Duration::ZERO, std::time::Duration::ZERO);
        let ticks = 60 * 60;
        for _ in 0..ticks {
            let t = std::time::Instant::now();
            f.step_trains();
            let d = t.elapsed();
            total += d;
            worst = worst.max(d);
        }
        println!("{} trains, signals {signals}: {:?} a tick, worst {worst:?}", f.trains.len(), total / ticks);
    }
}
