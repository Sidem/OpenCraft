use super::*;
use crate::action::Action;
use crate::block::{COPPER_ORE, DRONE_PORT, IRON_ORE, TOWN_DEPOT};
use crate::bytes::{ByteReader, ByteWriter};
use crate::crafting::Step;
use crate::item::{ItemId, COPPER_INGOT, HAULER_PACK, IRON_PICKAXE, IRON_PLATE, SCREW, STEEL_PLATE};
use crate::math::IVec3;
use crate::recipes::RECIPES;
use crate::settlements::{RIVER_TOWN, WORKSHOP};
use crate::sim::{PlayerId, Sim};

const P: PlayerId = PlayerId(0);
const AT: IVec3 = IVec3::new(3000, 120, -3000);
const DOWN: IVec3 = IVec3::new(0, 1, 0);

/// A town of `kind` with its camp built, at `AT` far from spawn, that never runs out of upkeep and builds nothing.
fn camp(kind: u8) -> Sim {
    let mut sim = Sim::new(2024, 2);
    sim.factory.place(&mut sim.world, TOWN_DEPOT, AT, 0, AT - DOWN, 0);
    let mut t = Town::new(AT, kind);
    (t.built, t.planned, t.due) = (1, 1, u64::MAX);
    t.delivered = vec![0; t.order().len()];
    sim.settlements.towns.push(t);
    sim.step();
    sim
}

fn steps(sim: &mut Sim, n: u64) {
    (0..n).for_each(|_| sim.step());
}

fn held(sim: &mut Sim, item: ItemId) -> (u32, u32) {
    let (input, out) = sim.factory.depot_perk(AT).unwrap();
    (input.count(item), out.count(item))
}

#[test]
fn a_forge_gives_half_again_the_plates_out_of_its_front_and_pauses_while_stalled() {
    let mut sim = camp(FORGE);
    // A belt leading away on both z sides of the depot, each into a box: only the front gives.
    let cells = sim.factory.footprint_at(AT).unwrap().2;
    let lo = cells.iter().fold(cells[0], |a, c| IVec3::new(a.x.min(c.x), a.y.min(c.y), a.z.min(c.z)));
    let hi = cells.iter().fold(cells[0], |a, c| IVec3::new(a.x.max(c.x), a.y.max(c.y), a.z.max(c.z)));
    let boxes = [IVec3::new(lo.x + 1, lo.y, lo.z - 2), IVec3::new(lo.x + 1, lo.y, hi.z + 2)];
    sim.factory.add_belt(IVec3::new(lo.x + 1, lo.y, lo.z - 1), 0);
    sim.factory.add_belt(IVec3::new(lo.x + 1, lo.y, hi.z + 1), 2);
    boxes.iter().for_each(|&b| sim.factory.add_storage(b));

    let ore = ItemId::block(IRON_ORE);
    assert_eq!(period(1), 720, "a camp: 4 ore every 12 s, 20 a minute");
    assert_eq!(sim.factory.insert(AT, ore, 40), 20, "it holds a minute's ore");
    assert_eq!(sim.factory.depot_store(AT).unwrap().count(ore), 0, "apart from the town's store");
    steps(&mut sim, 6 * period(1) + 300);
    let plates = boxes.map(|b| sim.factory.storage_count_at(b, IRON_PLATE));
    assert_eq!(plates.iter().sum::<u32>(), 15, "20 ore make 15 plates: {plates:?}");
    assert!(plates.contains(&0), "out of one side only: {plates:?}");

    // Copper ore gives ingots, half again a smelter's.
    sim.factory.insert(AT, ItemId::block(COPPER_ORE), 4);
    steps(&mut sim, period(1));
    let ingots: u32 = boxes.iter().map(|&b| sim.factory.storage_count_at(b, COPPER_INGOT)).sum();
    assert_eq!(ingots, 6);

    // Stalled, it keeps the ore and gives nothing until supplied again.
    sim.settlements.towns[0].stalled = true;
    sim.factory.insert(AT, ore, 8);
    steps(&mut sim, 3 * period(1));
    assert_eq!(held(&mut sim, ore).0, 8);
    sim.settlements.towns[0].stalled = false;
    steps(&mut sim, 3 * period(1));
    assert_eq!(held(&mut sim, ore).0, 0);

    // A city works 30 times as fast.
    assert_eq!(period(5), 24);
    let text = sim.settlements.towns[0].perk_text().unwrap();
    assert!(text.starts_with("Perk: 20 iron or copper ore a minute in"), "{text}");
    sim.settlements.towns[0].stalled = true;
    assert!(sim.settlements.towns[0].perk_text().unwrap().ends_with("(paused while stalled)"));
}

#[test]
fn only_a_forge_with_a_camp_takes_ore() {
    let ore = ItemId::block(IRON_ORE);
    assert_eq!(camp(0).factory.insert(AT, ore, 4), 0, "a harbour");
    let mut sim = camp(FORGE);
    sim.settlements.towns[0].built = 0;
    sim.step();
    assert_eq!(sim.factory.insert(AT, ore, 4), 0, "before its camp");
}

#[test]
fn a_river_town_doubles_the_reach_of_drone_ports_near_it_while_supplied() {
    let mut sim = camp(RIVER_TOWN);
    let (near, far) = (AT + IVec3::new(100, 0, 60), AT + IVec3::new(0, 0, 140));
    for p in [near, far] {
        sim.factory.place(&mut sim.world, DRONE_PORT, p, 0, p - DOWN, 0);
    }
    let reach = |sim: &mut Sim, at: IVec3| {
        sim.step();
        sim.factory.ports().iter().find(|p| p.anchor == at).map(|p| (p.reach, p.cargo_range)).unwrap()
    };
    assert_eq!(reach(&mut sim, near), (64.0, 400.0), "116 blocks off: twice a Mk1 port's");
    assert_eq!(reach(&mut sim, far), (32.0, 200.0), "140 blocks off");
    sim.settlements.towns[0].stalled = true;
    assert_eq!(reach(&mut sim, near), (32.0, 200.0), "paused while stalled");
}

#[test]
fn a_workshop_mends_worn_tools_and_halves_its_gear_while_supplied() {
    let mut sim = camp(WORKSHOP);
    assert_eq!(sim.factory.insert(AT, IRON_PICKAXE, 30), 30, "a worn pickaxe");
    steps(&mut sim, period(1));
    assert_eq!(held(&mut sim, IRON_PICKAXE), (0, 600), "comes out whole");

    sim.factory.research.complete_all();
    let pack = RECIPES.iter().position(|r| r.output == HAULER_PACK).unwrap() as u16;
    let give = |sim: &mut Sim| {
        for (item, n) in [(IRON_PLATE, 3), (STEEL_PLATE, 1), (SCREW, 5)] {
            sim.apply(P, Action::Give { item, count: n });
        }
    };
    give(&mut sim);
    sim.apply(P, Action::Craft { recipe: pack, times: 1 });
    let crafts = &sim.player(P).unwrap().crafts;
    assert_eq!(crafts.orders.len(), 1, "half the plates and screws pay for a pack");
    assert_eq!(crafts.orders[0].steps[0], Step { recipe: pack, times: 1, deliver: true, half: true });

    // The order keeps its price through a save and a stall.
    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut back = Sim::new(2024, 2);
    back.read_state(&mut ByteReader::new(&w.bytes)).expect("loads");
    assert!(back.player(P).unwrap().crafts.orders[0].steps[0].half);
    back.settlements.towns[0].stalled = true;
    steps(&mut back, 30 * crate::TICK_RATE as u64);
    assert_eq!(back.player(P).unwrap().inventory.count(HAULER_PACK), 1);

    // A stalled workshop: full price.
    give(&mut back);
    back.apply(P, Action::Craft { recipe: pack, times: 1 });
    assert!(back.player(P).unwrap().crafts.orders.is_empty());
}
