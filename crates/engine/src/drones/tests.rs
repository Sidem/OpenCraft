use super::*;
use crate::action::Action;
use crate::block::{AIR, BELT, COAL_ORE, DIRT, DRONE_PORT, GENERATOR, POLE, STORAGE};
use crate::item::{ItemId, DRONE};

const P: PlayerId = PlayerId(0);

fn at(y: i32, dx: i32, dz: i32) -> IVec3 {
    IVec3::new(900 + dx, y, -900 + dz)
}

fn put(sim: &mut Sim, item: ItemId, pos: IVec3) {
    sim.apply(P, Action::Give { item, count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos });
}

fn run(sim: &mut Sim, seconds: u32) {
    for _ in 0..seconds * TICK_RATE {
        sim.step();
    }
}

/// A powered port with `drones` at home and a supply box beside it, in the air; returns (port anchor, box cell).
fn base(sim: &mut Sim, drones: u32) -> (IVec3, IVec3) {
    base_at(sim, drones, 200)
}

/// The same on the ground: the first free cell above the highest block near the spot.
fn base_on_ground(sim: &mut Sim, drones: u32) -> (IVec3, IVec3) {
    let top = (895..925)
        .flat_map(|x| (-905..-885).map(move |z| (x, z)))
        .map(|(x, z)| {
            (0..255).rev().find(|&y| sim.world.block_anywhere_or_generate(IVec3::new(x, y, z)) != AIR).unwrap_or(0)
        })
        .max()
        .unwrap();
    base_at(sim, drones, top + 1)
}

fn base_at(sim: &mut Sim, drones: u32, y: i32) -> (IVec3, IVec3) {
    let (gen, pole, port) = (at(y, 0, 0), at(y, 4, 0), at(y, 9, 0));
    put(sim, GENERATOR.into(), gen);
    put(sim, POLE.into(), pole);
    put(sim, DRONE_PORT.into(), port);
    sim.apply(P, Action::Give { item: COAL_ORE.into(), count: 20 });
    sim.apply(P, Action::Insert { pos: gen, item: COAL_ORE.into() });
    sim.apply(P, Action::Give { item: DRONE, count: drones });
    sim.apply(P, Action::Insert { pos: port, item: DRONE });
    let centre = sim.factory.ports()[0].centre;
    let supply = IVec3::new(centre.x.floor() as i32 + 2, port.y, centre.z.floor() as i32);
    put(sim, STORAGE.into(), supply);
    run(sim, 1);
    (port, supply)
}

fn fleet(sim: &Sim) -> u32 {
    sim.factory.ports().iter().map(|p| p.home).sum::<u32>() + sim.drones.list.len() as u32
}

fn ghost(sim: &mut Sim, pos: IVec3) {
    sim.apply(P, Action::Give { item: BELT.into(), count: 1 });
    sim.apply(P, Action::PlaceGhost { pos, slot: 0, facing: 1 });
    sim.apply(P, Action::DropSelected { count: 1 });
}

#[test]
fn a_drone_builds_a_ghost_from_a_box_and_the_fleet_is_conserved() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base(&mut sim, 2);
    assert_eq!(sim.factory.ports()[0].home, 2);
    let target = port + IVec3::new(0, 0, 8);
    ghost(&mut sim, target);
    assert_eq!(sim.ghosts.len(), 1);
    sim.factory.stock(supply, BELT.into(), 3);
    let mut seen_flying = false;
    for _ in 0..TICK_RATE * 12 {
        sim.step();
        seen_flying |= !sim.drones.list.is_empty();
        assert_eq!(fleet(&sim), 2, "no drone appears or vanishes");
    }
    assert!(seen_flying, "a drone flew out");
    assert_eq!(sim.world.block_anywhere(target), Some(BELT), "and built the belt");
    assert_eq!(sim.ghosts.len(), 0);
    assert_eq!(sim.factory.box_count(supply, BELT.into()), 2, "one belt came from the box");
    assert_eq!(sim.drones.list.len(), 0, "and it is home again");
}

#[test]
fn a_port_does_nothing_without_the_item_power_or_reach() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base(&mut sim, 1);
    let target = port + IVec3::new(0, 0, 8);
    ghost(&mut sim, target);
    run(&mut sim, 4);
    assert_eq!((sim.drones.list.len(), sim.ghosts.len()), (0, 1), "the box has no belts");
    let far = port + IVec3::new(0, 0, 40);
    sim.factory.stock(supply, BELT.into(), 1);
    sim.apply(P, Action::RemoveGhost { pos: target });
    ghost(&mut sim, port + IVec3::new(0, 0, 80));
    run(&mut sim, 4);
    assert_eq!((sim.drones.list.len(), sim.factory.box_count(supply, BELT.into())), (0, 1), "out of reach (Mk1: 32)");
    let _ = far;
}

#[test]
fn a_tear_down_mark_breaks_the_block_into_the_box() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base(&mut sim, 1);
    let target = port + IVec3::new(0, 0, 6);
    put(&mut sim, STORAGE.into(), target);
    assert_eq!(sim.world.block_anywhere(target), Some(STORAGE));
    sim.apply(P, Action::MarkRemoval { pos: target });
    assert_eq!(sim.ghosts.len(), 1);
    run(&mut sim, 12);
    assert_eq!(sim.world.block_anywhere(target), Some(crate::block::AIR), "gone");
    assert_eq!(sim.ghosts.len(), 0, "the mark is done");
    assert_eq!(sim.factory.box_count(supply, STORAGE.into()), 1, "the storage box landed in the supply box");
    assert_eq!(fleet(&sim), 1);
}

#[test]
fn a_mark_on_nothing_is_not_made() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::MarkRemoval { pos: at(200, 3, 3) });
    assert_eq!(sim.ghosts.len(), 0);
}

#[test]
fn breaking_the_port_brings_its_drones_home_as_items() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base(&mut sim, 2);
    sim.factory.stock(supply, BELT.into(), 2);
    ghost(&mut sim, port + IVec3::new(0, 0, 8));
    ghost(&mut sim, port + IVec3::new(0, 0, 10));
    run(&mut sim, 2);
    assert!(!sim.drones.list.is_empty());
    sim.apply(P, Action::BreakBlock { pos: port });
    assert!(sim.drones.list.is_empty());
    let dropped: u32 = sim
        .events
        .iter()
        .filter_map(|e| match e {
            crate::sim::SimEvent::Dropped { item, count, .. } if *item == DRONE => Some(*count),
            _ => None,
        })
        .sum();
    assert_eq!(dropped, 2, "the stored and the flying drones all dropped");
}

#[test]
fn drones_save_and_load() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base(&mut sim, 1);
    sim.factory.stock(supply, BELT.into(), 1);
    ghost(&mut sim, port + IVec3::new(0, 0, 8));
    run(&mut sim, 2);
    assert_eq!(sim.drones.list.len(), 1);
    let mut w = ByteWriter::default();
    sim.drones.write_state(&mut w);
    let back = Drones::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back, sim.drones);
}

#[test]
fn the_same_flight_gives_the_same_state() {
    let go = || {
        let mut sim = Sim::new(7, 2);
        let (port, supply) = base(&mut sim, 3);
        sim.factory.stock(supply, BELT.into(), 3);
        for dz in [7, 9, 11] {
            ghost(&mut sim, port + IVec3::new(0, 0, dz));
        }
        run(&mut sim, 15);
        sim.state_hash()
    };
    assert_eq!(go(), go());
}

#[test]
fn ghost_mode_left_click_marks_and_unmarks_and_drones_are_drawn() {
    use crate::raycast::RayHit;
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let spot = g.body().eye().floor() + IVec3::new(3, 3, 3);
    put(&mut g.sim, BELT.into(), spot);
    g.run_ticks(1);
    // One press of the left button on the belt: `held` presses in a row before the tick are one hold.
    let click = |g: &mut crate::Game, held: u32| {
        g.mining = true;
        for _ in 0..held {
            g.target = Some(RayHit { block: spot, normal: IVec3::new(0, 1, 0), id: BELT });
            g.update_mining(0.016);
        }
        g.mining = false;
        g.update_mining(0.016);
        g.run_ticks(1);
    };
    click(&mut g, 1);
    assert_eq!(g.sim.ghosts.len(), 0, "mining is normal outside ghost mode");
    g.toggle_ghost_mode();
    click(&mut g, 3);
    assert_eq!(g.sim.ghosts.covering(spot).map(|m| m.block), Some(crate::block::AIR), "marked once");
    assert!(g.ghost_boxes().ends_with(&[spot.x, spot.y, spot.z, spot.x, spot.y, spot.z, 0xff4d4d]));
    let label = g.ghost_label();
    assert!(label.contains("1 marked for tear-down"), "{label}");
    click(&mut g, 1);
    assert_eq!(g.sim.ghosts.len(), 0, "a second click clears the mark");
    let eye = g.body().eye();
    let drawn = |g: &mut crate::Game| {
        g.instances.clear();
        g.write_drone_instances(eye, 0.5);
        g.instances.len()
    };
    assert_eq!(drawn(&mut g), 0);
    let near = eye + crate::math::Vec3::new(0.0, 0.0, 3.0);
    g.sim.drones.list.push(Drone {
        port: spot,
        pos: near,
        prev: near,
        target: spot,
        phase: Phase::Out,
        timer: 0,
        load: Stack::default(),
    });
    assert!(drawn(&mut g) > 0, "a drone near the eye is drawn");
}

/// `cargo test --release bench_drones -- --ignored --nocapture`: a port with a full fleet and 2,000 ghosts nobody can
/// build (no items in the box), the worst case for choosing jobs.
#[test]
#[ignore]
fn bench_drones() {
    let mut sim = Sim::new(7, 2);
    let (port, _) = base(&mut sim, 4);
    for i in 0..2000 {
        let pos = port + IVec3::new(i % 40 - 20, 0, 4 + i / 40);
        ghost(&mut sim, pos);
    }
    let (mut total, mut worst) = (std::time::Duration::ZERO, std::time::Duration::ZERO);
    let ticks = TICK_RATE * 20;
    for _ in 0..ticks {
        let t = std::time::Instant::now();
        sim.step();
        let d = t.elapsed();
        total += d;
        worst = worst.max(d);
    }
    println!("{} ghosts: {:?} a tick, worst {worst:?}", sim.ghosts.len(), total / ticks);
}

fn sites_gone(sim: &Sim) -> bool {
    sim.factory.sites.list.is_empty()
}

fn top(sim: &mut Sim, x: i32, z: i32) -> i32 {
    (0..255).rev().find(|&y| sim.world.block_anywhere_or_generate(IVec3::new(x, y, z)) != AIR).unwrap_or(0)
}

/// How many of the site's cells hold something solid.
fn solid_cells(sim: &mut Sim) -> usize {
    let s = sim.factory.sites.list[0];
    (0..s.cells()).filter(|&k| !block::replaceable(sim.world.block_anywhere_or_generate(s.cell(k)))).count()
}

#[test]
fn drones_dig_a_site_into_the_boxes_and_then_it_is_done() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base_on_ground(&mut sim, 4);
    let (a, b) = ((port.x + 6, port.z + 6), (port.x + 8, port.z + 8));
    let low =
        (a.0..=b.0).flat_map(|x| (a.1..=b.1).map(move |z| (x, z))).map(|(x, z)| top(&mut sim, x, z)).min().unwrap();
    let level = low - 1;
    sim.apply(P, Action::MarkSite { a, b, level, job: crate::factory::Job::Dig });
    assert_eq!(sim.factory.sites.list.len(), 1);
    assert!(sim.factory.sites.list[0].cells() >= 18, "at least two layers to cut");
    for _ in 0..TICK_RATE * 300 {
        sim.step();
        assert_eq!(fleet(&sim), 4);
        if sites_gone(&sim) {
            break;
        }
    }
    assert!(sites_gone(&sim), "finished");
    for x in a.0..=b.0 {
        for z in a.1..=b.1 {
            let above = sim.world.block_anywhere_or_generate(IVec3::new(x, level + 1, z));
            assert!(top(&mut sim, x, z) <= level || !crate::factory::cut_takes(above));
        }
    }
    assert!(sim.factory.box_total(supply) > 0, "the spoil is in the box");
}

#[test]
fn drones_fill_a_site_from_the_boxes_and_stop_when_they_run_out() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base_on_ground(&mut sim, 4);
    let (a, b) = ((port.x + 6, port.z + 6), (port.x + 7, port.z + 7));
    let level = port.y + 1;
    sim.apply(P, Action::MarkSite { a, b, level, job: crate::factory::Job::Fill });
    sim.factory.stock(supply, DIRT.into(), 8);
    let before = solid_cells(&mut sim);
    run(&mut sim, 120);
    assert_eq!(sim.factory.box_count(supply, DIRT.into()), 0, "all the dirt was used");
    assert_eq!(sim.factory.sites.list.len(), 1, "the site waits for more");
    assert_eq!(fleet(&sim), 4);
    assert_eq!(solid_cells(&mut sim), before + 8, "eight blocks went on the ground");
}

/// A block of stone under the air the port stands in (heights 190..=199 beside it).
fn stone_below(sim: &mut Sim) {
    for y in 190..=199 {
        for z in -2..=10 {
            for x in -2..=16 {
                sim.world.set_block_anywhere(at(y, x, z), crate::block::STONE);
            }
        }
    }
}

fn is_open(sim: &mut Sim, pos: IVec3) -> bool {
    block::replaceable(sim.world.block_anywhere_or_generate(pos))
}

#[test]
fn drones_bore_a_tunnel_into_the_boxes_and_wait_while_water_is_beside_it() {
    let mut sim = Sim::new(7, 2);
    stone_below(&mut sim);
    let (_, supply) = base(&mut sim, 4);
    let (from, to) = (at(196, 14, 4), at(196, 6, 4));
    sim.apply(P, Action::MarkTunnel { from, to, size: 0 });
    assert_eq!(sim.factory.sites.list.len(), 1);
    let cells = sim.factory.sites.list[0].cells();
    assert_eq!(cells, 18, "nine blocks long, one wide, two tall");
    // Water beside the path's last block: the cells touching it are left alone.
    let wet = at(196, 6, 5);
    sim.world.set_block_anywhere(wet, crate::block::WATER);
    for _ in 0..TICK_RATE * 200 {
        sim.step();
        assert_eq!(fleet(&sim), 4);
    }
    let site = sim.factory.sites.list[0];
    let open = (0..cells).filter(|&k| is_open(&mut sim, site.cell(k))).count();
    assert_eq!(open, 17, "all but the cell beside the water");
    assert!(is_open(&mut sim, at(196, 7, 4)) && !is_open(&mut sim, at(196, 6, 4)), "the face stands");
    assert_eq!(sim.factory.sites.list.len(), 1, "the site waits");
    assert!(sim.factory.box_total(supply) > 0, "the rock is in the box");
    assert_eq!(sim.world.block_anywhere(wet), Some(crate::block::WATER), "the water stayed put");

    sim.world.set_block_anywhere(wet, AIR);
    for _ in 0..TICK_RATE * 40 {
        sim.step();
        if sites_gone(&sim) {
            break;
        }
    }
    assert!(sites_gone(&sim), "finished when the water is gone");
    assert!(is_open(&mut sim, at(196, 6, 4)) && is_open(&mut sim, at(197, 6, 4)));
}

/// `cargo test --release bench_earthworks -- --ignored --nocapture`: a 64 × 64 flatten nine layers deep whose boxes are
/// full, so no cell can be worked and every launch scans them all: the worst case for choosing site jobs.
#[test]
#[ignore]
fn bench_earthworks() {
    let mut sim = Sim::new(7, 2);
    for y in 150..=199 {
        for z in 2..66 {
            for x in 0..64 {
                sim.world.set_block_anywhere(at(y, x, z), crate::block::STONE);
            }
        }
    }
    let (_, supply) = base(&mut sim, 4);
    let (a, b) = (at(0, 0, 2), at(0, 63, 65));
    sim.apply(P, Action::MarkSite { a: (a.x, a.z), b: (b.x, b.z), level: 190, job: crate::factory::Job::Flatten });
    sim.factory.stock(supply, crate::block::STONE.into(), 24 * 64);
    let cells = sim.factory.sites.list[0].cells();
    let (mut total, mut worst) = (std::time::Duration::ZERO, std::time::Duration::ZERO);
    let ticks = TICK_RATE * 20;
    for _ in 0..ticks {
        let t = std::time::Instant::now();
        sim.step();
        let d = t.elapsed();
        total += d;
        worst = worst.max(d);
    }
    println!("{cells} cells: {:?} a tick, worst {worst:?}", total / ticks);
}

#[test]
fn storing_and_landing_report_what_really_happened() {
    let mut sim = Sim::new(7, 2);
    let (port, supply) = base(&mut sim, 1);
    let left = sim.factory.store_in_boxes(&[supply], Stack { item: BELT.into(), count: 3 });
    assert_eq!((left.count, sim.factory.box_count(supply, BELT.into())), (0, 3), "all three fit");
    let big = Stack { item: BELT.into(), count: 24 * 64 };
    assert_eq!(sim.factory.store_in_boxes(&[supply], big).count, 3, "a full box leaves the rest");
    assert!(sim.factory.port_take(port, DRONE));
    assert!(sim.factory.port_land(port, DRONE), "the slot is free again");
}
