//! The guide must answer for every item a player can need, and its answers must lead somewhere.

use super::*;
use crate::block::{BLOCK_COUNT, CONCRETE, LIMESTONE};
use crate::item::{self, QUICKLIME, STEEL_INGOT};
use crate::recipes::FUELS;
use crate::settlements::{upkeep, ORDERS, STAGES};

/// Every item something asks for (a hand or machine recipe, a fuel slot, a town's order or upkeep) and every item a
/// breakable block drops.
fn needed() -> Vec<ItemId> {
    let breakable = |b: &usize| block::def(*b as BlockId).break_time >= 0.0;
    let mut out: Vec<ItemId> = Vec::new();
    out.extend(RECIPES.iter().flat_map(|r| r.inputs.iter().map(|x| x.0)));
    out.extend(MACHINE_RECIPES.iter().flat_map(|r| r.inputs.iter().map(|x| x.0)));
    out.extend(FUELS.iter().map(|f| f.0));
    out.extend(ORDERS.iter().flat_map(|o| o.iter().map(|l| l.0)));
    for kind in 0..4 {
        out.extend((1..=STAGES as u8).flat_map(|built| upkeep(kind, built)).map(|l| l.0));
    }
    out.extend(
        (0..BLOCK_COUNT)
            .filter(breakable)
            .map(|b| block::def(b as BlockId).drop)
            .filter(|&d| d != block::AIR)
            .map(ItemId::block),
    );
    out.retain(|i| i.is_valid());
    out.sort();
    out.dedup();
    out
}

#[test]
fn every_needed_item_has_a_way_in_every_generator_version() {
    for version in [1, 2, 5, 6, WorldGen::new(1).version()] {
        let gen = WorldGen::with_version(7, version);
        let missing: Vec<&str> = needed().into_iter().filter(|&i| ways(&gen, i).is_empty()).map(item::name).collect();
        assert!(missing.is_empty(), "v{version}: no way to get {missing:?}; add a FOUND row or a recipe");
    }
}

#[test]
fn every_needed_item_can_be_made_from_what_the_world_gives() {
    // Start from what is found, then add whatever a recipe makes once its inputs (and its machine) are reachable.
    let gen = WorldGen::new(7);
    let found = |i: usize| matches!(ways(&gen, ItemId(i as u16)).first(), Some(Way::Found(_)));
    let mut have: Vec<bool> = (0..item::ITEM_COUNT).map(found).collect();
    loop {
        let mut grew = false;
        let all_in = |have: &[bool], inputs: &[(ItemId, u32)]| inputs.iter().all(|x| have[x.0 .0 as usize]);
        for r in RECIPES {
            if !have[r.output.0 as usize] && all_in(&have, r.inputs) {
                have[r.output.0 as usize] = true;
                grew = true;
            }
        }
        for r in MACHINE_RECIPES.iter().filter(|r| r.category != Category::Launch) {
            let machine = machine_for(r.category).is_some_and(|m| have[m.0 as usize]);
            if machine && all_in(&have, r.inputs) {
                for o in r.outputs {
                    grew |= !have[o.0 .0 as usize];
                    have[o.0 .0 as usize] = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    let stuck: Vec<&str> = needed().into_iter().filter(|i| !have[i.0 as usize]).map(item::name).collect();
    assert!(stuck.is_empty(), "nothing the world gives leads to {stuck:?}");
}

#[test]
fn concrete_leads_back_to_limestone_in_the_ground() {
    // The play-tester's question: concrete is mixed in an assembler from quicklime, quicklime is smelted from
    // limestone, and limestone is dug at a depth this world's generator gives.
    let gen = WorldGen::new(7);
    let machine_inputs = |item: ItemId| -> Vec<(ItemId, Vec<ItemId>)> {
        let ways = ways(&gen, item);
        let made = ways.iter().filter_map(|w| match w {
            Way::Machine { recipe, machine } => {
                Some((*machine, MACHINE_RECIPES[*recipe].inputs.iter().map(|x| x.0).collect()))
            }
            _ => None,
        });
        made.collect()
    };
    let concrete = machine_inputs(ItemId::block(CONCRETE));
    assert!(concrete.iter().any(|(m, ins)| item::name(*m) == "Assembler" && ins.contains(&QUICKLIME)), "{concrete:?}");
    let lime = machine_inputs(QUICKLIME);
    assert!(lime.iter().any(|(m, ins)| item::name(*m) == "Smelter" && ins == &[ItemId::block(LIMESTONE)]), "{lime:?}");
    let (lo, hi) = gen.ore_band(LIMESTONE);
    let Some(Way::Found(text)) = ways(&gen, ItemId::block(LIMESTONE)).into_iter().next() else { panic!("not found") };
    assert!(text.contains(&format!("{lo} to {hi} blocks below")), "{text}");
    assert!(uses(QUICKLIME).iter().any(|&(m, i)| m && MACHINE_RECIPES[i].outputs[0].0 == STEEL_INGOT));
}

#[test]
fn the_plate_ratio_is_what_the_two_machines_panels_say() {
    use crate::block::{CONSTRUCTOR, IRON_ORE, SMELTER};
    use crate::factory::Factory;
    use crate::item::{IRON_INGOT, IRON_PLATE};
    use crate::math::IVec3;
    let plates = MACHINE_RECIPES.iter().position(|r| r.main().0 == IRON_PLATE).unwrap();
    let (mut f, mut world) = (Factory::default(), crate::world::World::new(1, 2));
    let (smelter, constructor) = (IVec3::ZERO, IVec3::new(4, 0, 0));
    f.place(&mut world, SMELTER, smelter, 0, smelter - IVec3::new(0, 1, 0), 0);
    f.place(&mut world, CONSTRUCTOR, constructor, 0, constructor - IVec3::new(0, 1, 0), 0);
    f.insert(smelter, IRON_ORE.into(), 1);
    f.set_recipe(constructor, Some(plates as u16)).unwrap();
    let give = f.smelter_at(smelter).rates().unwrap().1[0];
    let need = f.constructor_at(constructor).rates().unwrap().0[0];
    assert_eq!((give.0, need.0), (IRON_INGOT, IRON_INGOT));
    let line = ratios(&WorldGen::new(7), plates);
    let (a, b) = line.split_once(" Smelter to ").expect(&line);
    let b: u32 = b.split(' ').next().unwrap().parse().unwrap();
    let a: f64 = a.parse().unwrap();
    // a smelters make what b constructors use, within the 3% the line rounds to.
    let (made, used) = (a * f64::from(give.1), f64::from(b * need.1));
    assert!((made - used).abs() <= used * 0.03, "{line}: {a} × {} against {b} × {}", give.1, need.1);
    assert!(line.ends_with("Constructor (Iron Ingot)"), "{line}");
}

#[test]
fn every_machine_named_can_be_built_by_hand() {
    let gen = WorldGen::new(7);
    for r in MACHINE_RECIPES.iter().filter(|r| r.category != Category::Launch) {
        let machine = machine_for(r.category).expect("a machine makes every category");
        let built = ways(&gen, machine).iter().any(|w| matches!(w, Way::Hand(_)));
        assert!(built, "{} has no hand recipe", item::name(machine));
    }
}

#[test]
fn an_old_world_says_it_lacks_the_newer_ores() {
    let gen = WorldGen::with_version(7, 1);
    let Some(Way::Found(text)) = ways(&gen, ItemId::block(block::BAUXITE_ORE)).into_iter().next() else { panic!() };
    assert!(text.contains("new world"), "{text}");
}
