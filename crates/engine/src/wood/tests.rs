use super::*;

#[test]
fn every_wood_item_knows_its_wood_and_form() {
    let mut seen = std::collections::HashSet::new();
    for w in 0..WOODS {
        for f in FORMS {
            let id = item(w, f);
            assert_eq!(of(id), Some((w, f)), "{}", crate::item::name(id));
            assert!(seen.insert(id), "{} twice", crate::item::name(id));
            assert!(!crate::item::name(id).is_empty(), "wood {w} {f:?} has a name");
        }
    }
    assert_eq!(of(ItemId::block(block::STONE)), None);
}

#[test]
fn any_wood_stands_in_for_the_oaks_and_makes_its_own() {
    let pine = |f| item(2, f);
    assert!(like(pine(Form::Planks), ItemId::block(PLANKS)));
    assert!(like(pine(Form::Log), ItemId::block(LOG)));
    assert!(!like(pine(Form::Log), ItemId::block(PLANKS)), "a log is no planks");
    assert!(!like(ItemId::block(PLANKS), pine(Form::Planks)), "a recipe naming pine wants pine");
    assert_eq!(in_wood_of(ItemId::block(PLANKS), pine(Form::Log)), pine(Form::Planks));
    assert_eq!(in_wood_of(crate::item::STICK, pine(Form::Planks)), crate::item::STICK);
}

#[test]
fn every_tree_log_has_a_wood() {
    for b in 0..block::BLOCK_COUNT as BlockId {
        assert_eq!(block::is_log(b), of_tree(b).is_some(), "{}", block::def(b).name);
    }
}

/// An inventory holding what `inputs` ask for, with every wood item in `w`'s wood.
fn stocked_in(w: usize, inputs: &[(ItemId, u32)]) -> crate::inventory::Inventory {
    let mut inv = crate::inventory::Inventory::default();
    for &(it, n) in inputs {
        let have = of(it).map_or(it, |(_, f)| item(w, f));
        assert_eq!(inv.add(have, n), 0, "room for {n} {}", crate::item::name(have));
    }
    inv
}

#[test]
fn every_hand_recipe_with_wood_in_it_crafts_from_pine_and_makes_pine() {
    use crate::crafting::{max_times, plan, CraftQueue};
    use crate::recipes::RECIPES;
    use crate::research::Research;
    let mut research = Research::default();
    research.complete_all();
    let mut checked = 0;
    for (i, r) in RECIPES.iter().enumerate() {
        if !r.inputs.iter().any(|&(it, _)| standard(it) == it && of(it).is_some()) {
            continue;
        }
        let name = crate::item::name(r.output);
        let mut inv = stocked_in(2, r.inputs);
        let p = plan(&inv, &research, false, i as u16, 1).unwrap_or_else(|| panic!("{name} plans from pine"));
        assert!(p.leaves.iter().all(|&(it, _)| of(it).is_none_or(|(w, _)| w == 2)), "{name} takes only pine");
        assert!(max_times(&inv, &research, false, i as u16) > 0, "{name}");
        let (mut q, mut events) = (CraftQueue::default(), Vec::new());
        assert!(q.enqueue(&mut inv, &research, false, i as u16, 1), "{name} queues");
        for _ in 0..100_000 {
            if q.orders.is_empty() {
                break;
            }
            q.tick(&mut inv, crate::sim::PlayerId(0), 1000, &mut events);
        }
        assert!(q.orders.is_empty(), "{name} finishes");
        let made = of(r.output).map_or(r.output, |(_, f)| item(2, f));
        assert_eq!(inv.count(made), r.count, "{name} comes out as {}", crate::item::name(made));
        assert_eq!(inv.count_like(ItemId::block(PLANKS)), if r.output == ItemId::block(PLANKS) { r.count } else { 0 });
        checked += 1;
    }
    assert!(checked >= 5, "planks, door, floor and more use wood: {checked}");
}

#[test]
fn every_machine_recipe_with_wood_in_it_takes_any_wood_and_makes_its_own() {
    use crate::factory::Buffer;
    use crate::recipes::MACHINE_RECIPES;
    let mut checked = 0;
    for r in MACHINE_RECIPES {
        if !r.inputs.iter().any(|&(it, _)| standard(it) == it && of(it).is_some()) {
            continue;
        }
        let mut buf = Buffer::new(8);
        for &(it, n) in r.inputs {
            let have = of(it).map_or(it, |(_, f)| item(2, f));
            assert_eq!(buf.add(have, n), 0);
        }
        assert!(r.inputs.iter().all(|&(it, n)| buf.count_like(it) >= n));
        let taken: Vec<ItemId> = r.inputs.iter().flat_map(|&(it, n)| buf.remove_like(it, n)).map(|s| s.item).collect();
        assert_eq!(buf.total(), 0, "all of it taken");
        for &(out, _) in r.outputs {
            let want = of(out).map_or(out, |(_, f)| item(2, f));
            assert_eq!(made_from(out, taken.iter().copied()), want);
        }
        checked += 1;
    }
    assert!(checked >= 2, "{checked}");
}

#[test]
fn stock_is_used_oak_first_then_by_wood_number_and_oak_is_not_taken_for_pine() {
    let mut inv = crate::inventory::Inventory::default();
    let planks = |w| item(w, Form::Planks);
    inv.add(planks(2), 5);
    inv.add(planks(1), 3);
    inv.add(planks(0), 4);
    assert_eq!(inv.count_like(planks(0)), 12);
    assert_eq!(inv.count_like(planks(2)), 5, "a pine ask wants pine");
    let taken = inv.remove_like(planks(0), 9);
    let items: Vec<(ItemId, u32)> = taken.iter().map(|s| (s.item, s.count)).collect();
    assert_eq!(items, [(planks(0), 4), (planks(1), 3), (planks(2), 2)]);
    assert_eq!(made_from(ItemId::block(WOOD_FLOOR), taken.iter().map(|s| s.item)), item(0, Form::Floor));
    assert!(inv.remove_like(planks(0), 4).is_empty(), "3 pine planks left: all or nothing");
    assert_eq!(inv.count(planks(2)), 3);
}

#[test]
fn pine_floor_taken_apart_gives_pine_planks() {
    use crate::recipes::RECIPES;
    let door = RECIPES.iter().position(|r| r.output == item(0, Form::Floor)).expect("a floor recipe");
    let mut inv = crate::inventory::Inventory::default();
    inv.add(item(2, Form::Floor), 4);
    let mut events = Vec::new();
    crate::take_apart::take_apart(&mut inv, door, 1, crate::sim::PlayerId(0), &mut events);
    assert_eq!(inv.count(item(2, Form::Floor)), 0);
    assert!(inv.count(item(2, Form::Planks)) > 0, "pine planks back");
    assert_eq!(inv.count(item(0, Form::Planks)), 0);
}
