//! The water wheel in a bare factory high above the ground (so only the water the tests place is around it): what it
//! counts, its cap, that it gives only what the grid lacks, that it reads the world every tick and that it saves
//! nothing of its own.

use super::*;
use crate::block::{flow, CONSTRUCTOR, FLOW_7, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tests::recipe_for;
use crate::factory::{Factory, Machine};
use crate::item::{IRON_INGOT, IRON_ROD};

const WHEEL: IVec3 = IVec3::new(0, 100, 0);
const LOAD: IVec3 = IVec3::new(8, 100, 0);

/// A wheel, a pole and a 15 kW load (a constructor pressing rods).
fn plant() -> (Factory, World) {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let mut put = |f: &mut Factory, block, pos: IVec3, tier| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
    };
    put(&mut f, POLE, IVec3::new(4, 100, 4), 3);
    put(&mut f, WATER_WHEEL, WHEEL, 0);
    put(&mut f, CONSTRUCTOR, LOAD, 0);
    assert!(f.set_recipe(LOAD, Some(recipe_for(IRON_ROD))).is_some());
    f.insert(LOAD, IRON_INGOT, 64);
    (f, world)
}

/// The cells just outside the wheel, where its water is read.
fn around(f: &Factory) -> Vec<IVec3> {
    let cells = f.processors.iter().find(|p| p.energy() == Energy::Hydro).unwrap().cells();
    let mut out = Vec::new();
    for &c in &cells {
        out.extend(FACES.iter().map(|&d| c + d).filter(|n| !cells.contains(n)));
    }
    out
}

fn tick(f: &mut Factory, world: &mut World, n: u64) {
    let mut events = Vec::new();
    for t in 0..n {
        f.update(world, t, &mut events);
    }
}

fn wheel(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.energy() == Energy::Hydro).unwrap()
}

#[test]
fn a_dry_wheel_gives_nothing_and_says_so() {
    let (mut f, mut world) = plant();
    tick(&mut f, &mut world, 3);
    assert_eq!((wheel(&f).store.flow, f.power.supply[0]), (0, 0));
    assert!(wheel(&f).status_text().starts_with("Dry"), "{}", wheel(&f).status_text());
}

#[test]
fn water_blocks_touching_it_give_2_kw_each_and_flowing_water_double() {
    let (mut f, mut world) = plant();
    let ring = around(&f);
    assert_eq!(ring.len(), 30, "two faces of nine and four edges of three");
    for &c in &ring[..3] {
        world.set_block_anywhere(c, WATER);
    }
    tick(&mut f, &mut world, 2);
    assert_eq!(wheel(&f).store.flow, 6);
    for &c in &ring[3..6] {
        world.set_block_anywhere(c, flow(FLOW_7 - crate::block::FLOW_1 + 1));
    }
    tick(&mut f, &mut world, 2);
    assert_eq!(wheel(&f).store.flow, 6 + 12);
    for &c in &ring {
        world.set_block_anywhere(c, WATER);
    }
    tick(&mut f, &mut world, 2);
    assert_eq!(wheel(&f).store.flow, MAX_KW, "30 blocks would be 60 kW, capped");
}

#[test]
fn it_gives_only_what_the_grid_lacks_and_follows_the_water_each_tick() {
    let (mut f, mut world) = plant();
    let ring = around(&f);
    for &c in &ring {
        world.set_block_anywhere(c, WATER);
    }
    tick(&mut f, &mut world, 5);
    assert_eq!(f.power.demand[0], 15);
    assert_eq!((f.power.supply[0], wheel(&f).store.given, f.power.capacity[0]), (15, 15, MAX_KW));
    assert!(wheel(&f).status_text().contains("giving 15 kW"), "{}", wheel(&f).status_text());
    for &c in &ring {
        world.set_block_anywhere(c, crate::block::AIR);
    }
    tick(&mut f, &mut world, 2);
    assert_eq!((f.power.supply[0], wheel(&f).store.flow), (0, 0), "the water was drained");
}

#[test]
fn a_wheel_writes_no_state_of_its_own() {
    let (f, _) = plant();
    let mut w = ByteWriter::default();
    wheel(&f).write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.spec.block, back.store.flow, back.store.charge), (WATER_WHEEL, 0, 0));
}
