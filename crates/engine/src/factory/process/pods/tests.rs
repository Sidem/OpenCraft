//! Pod launchers and landing pads: what they take, why a launcher waits, a launch and a landing, channels, a full pad, the
//! falling capsule, and the tech and recipe that gate them.

use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::power::FULL_SPEED;
use crate::factory::{Factory, Machine};
use crate::item::{COMMS_SATELLITE, DROP_POD, IRON_INGOT, IRON_PLATE, SCREW, SURVEY_SATELLITE};
use crate::math::IVec3;
use crate::pods::{FLIGHT_TICKS, POD_STACKS};
use crate::recipes::{DROP_POD_RECIPE, MACHINE_RECIPES};
use crate::research::{Unlock, TECHS};
use crate::sim::SimEvent;
use crate::world::World;

use super::super::spec;
use super::*;

const ALL: [bool; MACHINE_RECIPES.len()] = [true; MACHINE_RECIPES.len()];
const LOAD: u64 = 900;

fn launcher(channel: u8) -> Processor {
    Processor { dir: channel, ..Processor::new(IVec3::ZERO, &LAUNCHER_SPEC, 0) }
}

fn pad_at(x: i32, channel: u8) -> Processor {
    Processor { dir: channel, ..Processor::new(IVec3::new(x, 0, 0), &PAD_SPEC, 0) }
}

fn cover() -> Satellites {
    let mut sats = Satellites::default();
    sats.launch(COMMS_SATELLITE, IVec3::ZERO, 0);
    sats
}

/// Runs `n` ticks from `from`, every launcher at full power, and returns the events (launches and touchdowns).
fn go(ps: &mut [Processor], pods: &mut Pods, sats: &Satellites, from: u64, n: u64) -> Vec<SimEvent> {
    let (mut events, kinds) = (Vec::new(), Kinds::of(ps));
    for t in from..from + n {
        ps.iter_mut().for_each(|p| p.speed = FULL_SPEED);
        run(ps, &kinds, pods, sats, t, &mut events);
    }
    events
}

fn load(p: &mut Processor, cargo: &[(ItemId, u32)]) {
    assert_eq!(p.insert(DROP_POD, 1, &ALL), 1, "the pod goes in");
    for &(item, n) in cargo {
        assert_eq!(p.insert(item, n, &ALL), n);
    }
}

#[test]
fn a_launcher_keeps_pods_apart_from_cargo_and_a_pad_takes_only_cargo_in() {
    let mut l = launcher(0);
    assert_eq!(l.insert(DROP_POD, 5, &ALL), l.input.space_for(DROP_POD).min(5));
    assert_eq!(l.insert(IRON_PLATE, 10, &ALL), 10);
    assert_eq!((l.input.count(DROP_POD) > 0, l.out.count(IRON_PLATE), l.out.count(DROP_POD)), (true, 10, 0));
    assert_eq!(l.input.count(IRON_PLATE), 0);
    let pad = pad_at(5, 0);
    assert_eq!(pad.input.slots.len(), 0, "a pad has no input slots");
    assert_eq!(LAUNCHER_SPEC.footprint.size, [3, 3, 3]);
    assert_eq!(PAD_SPEC.footprint.size, [3, 3, 1]);
    assert_eq!(launcher(0).power(), LAUNCHER_KW);
}

#[test]
fn it_says_what_it_lacks_in_order() {
    let mut l = launcher(1);
    assert!(l.status_text().contains("Needs a drop pod"), "{}", l.status_text());
    l.insert(DROP_POD, 1, &ALL);
    assert!(l.status_text().contains("waiting for cargo"), "{}", l.status_text());
    l.insert(IRON_PLATE, 1, &ALL);
    let (mut pods, mut ps) = (Pods::default(), vec![l]);
    go(&mut ps, &mut pods, &Satellites::default(), 0, 2);
    assert!(ps[0].status_text().contains("No comms satellite overhead"), "{}", ps[0].status_text());
    go(&mut ps, &mut pods, &cover(), 2, 2);
    assert!(ps[0].status_text().contains("No landing pad on channel east"), "{}", ps[0].status_text());
    assert_eq!((ps[0].status, ps[0].progress), (Status::NoInput, 0));
}

#[test]
fn a_launcher_loads_for_fifteen_seconds_then_sends_six_stacks_to_the_pad_twenty_seconds_away() {
    let mut l = launcher(0);
    load(&mut l, &[(IRON_PLATE, 100), (IRON_INGOT, 100), (SCREW, 30)]);
    let (mut ps, mut pods, sats) = (vec![l, pad_at(30, 0)], Pods::default(), cover());
    go(&mut ps, &mut pods, &sats, 0, LOAD - 2);
    assert!(pods.list.is_empty());
    assert!(ps[0].status_text().contains("loading the pod: 99%"), "{}", ps[0].status_text());
    let sent = go(&mut ps, &mut pods, &sats, LOAD - 2, 3);
    assert_eq!(sent, vec![SimEvent::PodSent { pos: IVec3::ZERO }], "the launch is reported once");
    assert_eq!(pods.list.len(), 1);
    assert_eq!((ps[0].input.total(), ps[0].out.total()), (0, 0), "the pod and the cargo went up");
    assert_eq!(ps[0].used.len(), 1, "the pod is reported as used");
    let pod = &pods.list[0];
    assert_eq!((pod.to, pod.channel, pod.lands - pod.launched), (IVec3::new(30, 0, 0), 0, FLIGHT_TICKS));
    assert_eq!(ps[1].out.total(), 0, "not landed yet");
    assert!(ps[1].pods_text().is_some());
    let landed = go(&mut ps, &mut pods, &sats, LOAD + 1, FLIGHT_TICKS);
    assert_eq!(landed, vec![SimEvent::PodLanded { pos: IVec3::new(30, 0, 0) }], "the touchdown is reported once");
    assert!(pods.list.is_empty());
    assert_eq!((ps[1].out.count(IRON_PLATE), ps[1].out.count(IRON_INGOT), ps[1].out.count(SCREW)), (100, 100, 30));
}

#[test]
fn a_pod_carries_at_most_six_stacks_and_the_rest_waits_for_the_next() {
    let mut l = launcher(0);
    l.insert(DROP_POD, 2, &ALL);
    for item in [IRON_PLATE, IRON_INGOT, SCREW, SURVEY_SATELLITE, crate::item::COPPER_INGOT, crate::item::IRON_ROD] {
        l.insert(item, 10, &ALL);
    }
    l.insert(crate::item::COPPER_WIRE, 10, &ALL);
    let (mut ps, mut pods, sats) = (vec![l, pad_at(30, 0)], Pods::default(), cover());
    go(&mut ps, &mut pods, &sats, 0, LOAD + 1);
    assert_eq!(pods.list[0].cargo.len(), POD_STACKS);
    assert_eq!(ps[0].out.total(), 10, "one stack is left for the next pod");
    assert_eq!(ps[0].input.count(DROP_POD), 1);
}

#[test]
fn a_pod_goes_only_to_a_pad_on_its_channel_and_to_the_nearest() {
    let mut l = launcher(2);
    load(&mut l, &[(IRON_PLATE, 10)]);
    let mut ps = vec![l, pad_at(10, 0), pad_at(200, 2), pad_at(50, 2)];
    let (mut pods, sats) = (Pods::default(), cover());
    go(&mut ps, &mut pods, &sats, 0, LOAD + 1);
    assert_eq!(pods.list[0].to, IVec3::new(50, 0, 0), "the nearer of the two south pads, not the north one");
    go(&mut ps, &mut pods, &sats, LOAD + 1, FLIGHT_TICKS);
    assert_eq!((ps[1].out.total(), ps[2].out.total(), ps[3].out.total()), (0, 0, 10));
}

#[test]
fn a_pad_outside_comms_cover_is_not_a_destination_and_a_stranded_pod_waits_for_one() {
    let mut l = launcher(0);
    load(&mut l, &[(IRON_PLATE, 10)]);
    // 3000 blocks away the sky is the other side of the 4096-block wrap, outside the 1024-block circle.
    let mut ps = vec![l, pad_at(3000, 0)];
    let (mut pods, sats) = (Pods::default(), cover());
    go(&mut ps, &mut pods, &sats, 0, LOAD + 1);
    assert!(pods.list.is_empty(), "no covered pad, no launch");
    assert!(ps[0].status_text().contains("No landing pad"), "{}", ps[0].status_text());
    ps.push(pad_at(20, 0));
    go(&mut ps, &mut pods, &sats, LOAD + 1, 2);
    ps.remove(2);
    ps[0].progress = 0;
    ps[0].store.flow = 0;
    // A pod in flight whose pad goes away stays up (nothing is lost) until a pad is there again.
    let mut ps = vec![launcher(0), pad_at(20, 0)];
    ps[0].insert(DROP_POD, 1, &ALL);
    ps[0].insert(IRON_PLATE, 5, &ALL);
    let mut pods = Pods::default();
    go(&mut ps, &mut pods, &sats, 0, LOAD + 1);
    let pad = ps.remove(1);
    go(&mut ps, &mut pods, &sats, LOAD + 1, FLIGHT_TICKS + 100);
    assert_eq!((pods.list.len(), pods.list[0].cargo[0].count), (1, 5), "still in the sky");
    ps.push(pad);
    go(&mut ps, &mut pods, &sats, LOAD + FLIGHT_TICKS + 101, 2);
    assert!(pods.list.is_empty());
    assert_eq!(ps[1].out.count(IRON_PLATE), 5);
}

#[test]
fn a_full_pad_takes_what_fits_and_the_pod_lands_the_rest_when_it_is_emptied() {
    let mut l = launcher(0);
    load(&mut l, &[(IRON_PLATE, 30)]);
    let mut pad = pad_at(10, 0);
    for slot in 0..PAD_SLOTS {
        pad.out.slots[slot] = Stack { item: IRON_INGOT, count: 64 };
    }
    pad.out.take(0, 64);
    pad.out.add(IRON_PLATE, 64 - 10);
    let mut ps = vec![l, pad];
    let (mut pods, sats) = (Pods::default(), cover());
    go(&mut ps, &mut pods, &sats, 0, LOAD + FLIGHT_TICKS + 5);
    assert_eq!(ps[1].out.count(IRON_PLATE), 64, "ten more plates fit");
    assert_eq!(pods.list[0].cargo[0].count, 20, "twenty stay in the pod");
    ps[1].out.take(0, 64);
    go(&mut ps, &mut pods, &sats, LOAD + FLIGHT_TICKS + 5, 2);
    assert!(pods.list.is_empty());
    assert_eq!(ps[1].out.count(IRON_PLATE), 20);
}

#[test]
fn a_launcher_draws_power_only_while_it_loads() {
    let unlocked = ALL;
    let research = crate::research::Research::default();
    let mut l = launcher(0);
    assert!(!l.wants_power(&unlocked, &research));
    load(&mut l, &[(IRON_PLATE, 5)]);
    let (mut ps, mut pods) = (vec![l, pad_at(10, 0)], Pods::default());
    go(&mut ps, &mut pods, &cover(), 0, 2);
    assert!(ps[0].wants_power(&unlocked, &research));
    ps[0].speed = 0;
    let kinds = Kinds::of(&ps);
    run(&mut ps, &kinds, &mut pods, &cover(), 3, &mut Vec::new());
    assert_eq!(ps[0].status, Status::NoPower);
    assert!(ps[0].progress > 0 || ps[0].status == Status::NoPower, "an unwired launcher holds its progress");
}

#[test]
fn pods_in_flight_survive_a_save_and_a_launcher_keeps_its_load() {
    let mut l = launcher(3);
    load(&mut l, &[(IRON_PLATE, 5)]);
    let (mut ps, mut pods, sats) = (vec![l, pad_at(10, 3)], Pods::default(), cover());
    go(&mut ps, &mut pods, &sats, 0, 100);
    let mut w = ByteWriter::default();
    ps[0].write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(
        (back.dir, back.progress, back.input.count(DROP_POD), back.out.count(IRON_PLATE)),
        (3, ps[0].progress, 1, 5)
    );
    assert!(back.progress > 0, "a load in progress is saved");
    go(&mut ps, &mut pods, &sats, 100, LOAD);
    assert_eq!(pods.list.len(), 1);
    let mut w = ByteWriter::default();
    pods.write_state(&mut w);
    let back = Pods::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.list.len(), back.list[0].channel, back.list[0].cargo.len()), (1, 3, 1));
    assert_eq!(back.list[0].lands, pods.list[0].lands);
}

#[test]
fn a_capsule_is_drawn_only_in_the_last_four_seconds_above_its_pad() {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    f.place(&mut world, crate::block::LANDING_PAD, IVec3::new(4, 10, 4), 0, IVec3::ZERO, 0);
    f.pods.send(IVec3::ZERO, IVec3::new(4, 10, 4), 0, vec![Stack { item: IRON_PLATE, count: 1 }], 0);
    assert!(f.falling_pods(100.0).is_empty(), "still far up");
    let near = f.falling_pods(FLIGHT_TICKS as f64 - 120.0);
    let base = f.falling_pods(FLIGHT_TICKS as f64);
    assert_eq!((near.len(), base.len()), (1, 1));
    assert!(near[0].y > base[0].y + 10.0, "higher up an earlier tick: {} against {}", near[0].y, base[0].y);
    // The pad's cells run right and away from the placer: x 4..=6, z 2..=4, so its middle is (5.5, 3.5).
    assert!((base[0].x - 5.5).abs() < 0.1 && (base[0].z - 3.5).abs() < 0.1, "above the pad: {:?}", base[0]);
}

#[test]
fn a_capsule_rises_out_of_its_launcher_for_the_first_three_seconds() {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let at = IVec3::new(4, 10, 4);
    f.place(&mut world, POD_LAUNCHER, at, 0, IVec3::ZERO, 0);
    f.pods.send(at, IVec3::new(90, 10, 4), 0, vec![Stack { item: IRON_PLATE, count: 1 }], 100);
    assert!(f.rising_pods(99.0).is_empty(), "not launched yet");
    let (low, high) = (f.rising_pods(110.0), f.rising_pods(100.0 + crate::pods::RISE_TICKS as f64 - 1.0));
    assert_eq!((low.len(), high.len()), (1, 1));
    assert!(low[0].y > 12.0 && low[0].y < 15.0, "at the tube's mouth, 3 high: {}", low[0].y);
    assert!(high[0].y > low[0].y + 100.0, "far up by the end: {}", high[0].y);
    assert!(f.rising_pods(100.0 + crate::pods::RISE_TICKS as f64 + 1.0).is_empty(), "out of sight after");
}

#[test]
fn the_drop_pods_tech_unlocks_both_machines_and_the_pod_recipe_for_the_assembler() {
    let tech = TECHS.iter().position(|t| t.name == "Drop Pods").unwrap();
    assert_eq!(tech, 83, "saved by index");
    for need in ["Rocketry", "Orbital Comms"] {
        let n = TECHS.iter().position(|t| t.name == need).unwrap() as u8;
        assert!(TECHS[tech].needs.contains(&n), "needs {need}");
    }
    let unlocks = |b| TECHS[tech].unlocks.iter().any(|u| matches!(u, Unlock::Recipe(i) if *i == ItemId::block(b)));
    assert!(unlocks(crate::block::POD_LAUNCHER) && unlocks(crate::block::LANDING_PAD));
    assert!(TECHS[tech].unlocks.iter().any(|u| matches!(u, Unlock::MachineRecipe(i) if *i == DROP_POD_RECIPE)));
    assert_eq!(MACHINE_RECIPES[DROP_POD_RECIPE as usize].outputs, &[(DROP_POD, 1)]);
    assert!(spec(crate::block::ASSEMBLER).unwrap().recipe(DROP_POD_RECIPE).is_some());
    assert!(MACHINE_RECIPES[DROP_POD_RECIPE as usize].inputs.len() <= 3, "the assembler has three input slots");
}
