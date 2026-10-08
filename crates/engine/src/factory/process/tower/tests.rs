//! The cooling tower: pairing with datacenters, the loop's water loss over an hour, the status lines, the save, and the
//! whole thing placed in a bare factory (ports, pipes, networks). Pump water is set directly: no world needed.

use super::super::datacenter::{COOLANT_UNIT, DATACENTER_SPEC};
use super::super::steam::draw_water;
use super::*;
use crate::block::{COOLING_TOWER, DATACENTER, PIPE, PUMP};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::footprint::Role;
use crate::factory::pipes::{Part as Piece, Pipework};
use crate::factory::{Factory, Machine, DIRS};
use crate::math::IVec3;
use crate::world::World;
use crate::TICK_RATE;

fn tower() -> Processor {
    let mut p = Processor::new(IVec3::ZERO, &TOWER_SPEC, 0);
    p.steam.nets = vec![0];
    p.status = Status::Working;
    p
}

fn hall() -> Processor {
    let mut p = Processor::new(IVec3::ZERO, &DATACENTER_SPEC, 0);
    p.steam.nets = vec![0];
    p
}

fn pump(held: u32) -> Pipework {
    let mut p = Pipework::new(IVec3::ZERO, PUMP, 0);
    p.held = held;
    p
}

#[test]
fn the_spec_is_a_3x3x5_tower_with_a_water_inlet() {
    assert_eq!(TOWER_SPEC.footprint.size, [3, 3, 5]);
    assert_eq!(TOWER_SPEC.footprint.ports.len(), 1);
    assert!(tower().takes_water() && tower().wants_power(&[], &Default::default()));
}

#[test]
fn a_tower_cools_two_datacenters_on_its_network_and_not_a_third() {
    let mut ps = vec![hall(), hall(), hall(), tower()];
    ps[2].steam.nets = vec![0];
    seat(&mut ps);
    assert_eq!(ps[3].steam.boilers, [0, 1]);
    assert_eq!((ps[0].steam.tower, ps[1].steam.tower, ps[2].steam.tower), (Some(3), Some(3), None));
}

#[test]
fn a_tower_on_another_network_does_not_cool_it() {
    let mut ps = vec![hall(), tower()];
    ps[1].steam.nets = vec![7];
    seat(&mut ps);
    assert_eq!((ps[0].steam.tower, ps[1].steam.boilers.len()), (None, 0));
}

/// Pump units taken in an hour of full load, with or without a tower, and whether the loop showed as closed.
fn hour(with_tower: bool) -> (u32, bool) {
    let mut ps = vec![hall()];
    if with_tower {
        ps.push(tower());
    }
    seat(&mut ps);
    let mut pipes = vec![pump(2)];
    let (mut taken, mut looped) = (0, false);
    for _ in 0..3600 * TICK_RATE {
        pipes[0].held = 2;
        draw_water(&mut ps, &mut pipes);
        taken += 2 - pipes[0].held;
        ps[0].run_datacenter(1000);
        assert_ne!(ps[0].status, Status::Overheated, "it never overheats with water");
        looped |= ps[0].steam.looped;
    }
    (taken, looped)
}

#[test]
fn a_looped_datacenter_costs_the_pump_a_twentieth_of_what_an_open_one_does() {
    let (open, looped) = hour(false);
    assert!((355..=365).contains(&open), "an hour open takes {open} units");
    assert!(!looped);
    let (closed, looped) = hour(true);
    assert!(looped);
    assert!(closed <= open / LOOP_LOSS + 5, "an hour looped takes {closed} units");
}

#[test]
fn an_unpowered_tower_or_one_without_water_leaves_the_loop_open() {
    for fault in 0..2 {
        let mut ps = vec![hall(), tower()];
        seat(&mut ps);
        match fault {
            0 => ps[1].status = Status::NoPower,
            _ => ps[1].steam.water = 0,
        }
        let mut pipes = vec![pump(2)];
        draw_water(&mut ps, &mut pipes);
        assert!(!ps[0].steam.looped, "fault {fault}");
        assert_eq!(ps[0].steam.water, COOLANT_UNIT, "the datacenter fell back to the pump");
    }
}

#[test]
fn the_status_lines_say_what_is_missing() {
    let mut p = tower();
    assert!(p.tower_text().unwrap().starts_with("Out of makeup water"));
    p.steam.water = 3;
    assert!(p.tower_text().unwrap().starts_with("No datacenter"));
    p.steam.boilers = vec![0];
    assert_eq!(p.tower_text().unwrap(), "Cooling 1 of 2 datacenters · 3 units of makeup water");
    let mut d = hall();
    d.status = Status::Working;
    d.steam.water = 1;
    d.steam.looped = true;
    assert!(d.datacenter_text().unwrap().ends_with("cooling loop closed"));
}

#[test]
fn the_tank_and_circulation_count_survive_a_save() {
    let mut p = tower();
    p.steam.water = 3;
    p.progress = 7;
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.steam.water, back.progress, back.spec.block), (3, 7, COOLING_TOWER));
}

#[test]
fn placed_and_piped_together_they_pair_up_and_the_tower_fills_from_the_pump() {
    let mut f = Factory::default();
    let mut world = World::new(1, 2);
    let (dc, tw) = (IVec3::new(0, 0, 0), IVec3::new(0, 0, 8));
    for (block, pos) in [(DATACENTER, dc), (COOLING_TOWER, tw)] {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    }
    let inlet = |p: &Processor| {
        let (cell, side) = p.spec.footprint.faces(p.pos, p.dir, Role::Water)[0];
        cell + DIRS[side as usize]
    };
    let (from, to) = (inlet(&f.processors[0]), inlet(&f.processors[1]));
    assert_eq!((from.x, from.y), (to.x, to.y), "the inlets line up along z");
    for z in from.z..=to.z {
        let at = IVec3::new(from.x, from.y, z);
        f.place(&mut world, PIPE, at, 0, at - IVec3::new(0, 1, 0), 0);
    }
    let at = IVec3::new(from.x - 1, from.y, from.z);
    f.place(&mut world, PUMP, at, 0, at - IVec3::new(0, 1, 0), 0);
    let pump = f.pipework.iter().position(|p| p.part == Piece::Pump).unwrap();
    f.pipework[pump].held = 2;
    let mut events = Vec::new();
    for t in 0..5 {
        f.update(&mut world, t, &mut events);
    }
    assert_eq!(f.processors[0].steam.tower, Some(1));
    assert_eq!(f.processors[1].steam.boilers, [0]);
    assert!(f.processors[1].steam.water > 0, "the tower drew makeup water from the pump");
}
