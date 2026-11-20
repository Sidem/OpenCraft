use super::*;
use crate::block::{BELT, SPLITTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::recipes::{MACHINE_RECIPES, RECIPES};

fn finish(r: &mut Research, tech: u8) {
    (0..TECHS[tech as usize].units).for_each(|_| r.add_unit(tech));
}

/// Content lint for a tech table: prerequisites come earlier (so every tech is reachable and there
/// are no cycles), packs are in `PACKS`, every unlock exists and is listed once, and there is work to do.
fn lint_techs(techs: &[Tech]) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, t) in techs.iter().enumerate() {
        if !t.needs.iter().all(|&n| (n as usize) < i) {
            errors.push(format!("{} needs a later tech", t.name));
        }
        if !t.packs.iter().all(|&p| pack_slot(p).is_some()) {
            errors.push(format!("{} uses a pack labs don't hold", t.name));
        }
        for &u in t.unlocks {
            let exists = match u {
                Unlock::Recipe(item) => RECIPES.iter().any(|r| r.output == item),
                Unlock::MachineRecipe(r) => (r as usize) < MACHINE_RECIPES.len(),
                Unlock::Upgrade(block, tier) => tier > 0 && crate::factory::tiers::item_of(block, tier).is_some(),
                Unlock::Bonus(_) | Unlock::Feature(_) | Unlock::Perk(..) => true,
            };
            if !exists {
                errors.push(format!("{} unlocks {u:?}, which doesn't exist", t.name));
            }
            let earlier = techs[..i].iter().any(|o| o.unlocks.contains(&u));
            if earlier || t.unlocks.iter().filter(|&&x| x == u).count() > 1 {
                errors.push(format!("{} unlocks {u:?} again", t.name));
            }
        }
        if t.units == 0 || t.seconds <= 0.0 {
            errors.push(format!("{} has no work", t.name));
        }
    }
    if techs.len() >= u8::MAX as usize {
        errors.push("too many techs for a u8 index".to_string());
    }
    errors
}

#[test]
fn the_tech_table_passes_the_lint() {
    assert_eq!(lint_techs(TECHS), Vec::<String>::new());
}

#[test]
fn the_tech_lint_catches_a_planted_mistake_of_each_kind() {
    const fn tech(name: &'static str, needs: &'static [u8], unlocks: &'static [Unlock]) -> Tech {
        Tech { name, blurb: "", needs, packs: &[RED_PACK], units: 1, seconds: 1.0, unlocks }
    }
    const SPLITTERS: Unlock = Unlock::Recipe(ItemId::block(SPLITTER));
    const BAD: [Tech; 4] = [
        tech("Loop", &[1], &[]),
        tech("Twice", &[], &[SPLITTERS, SPLITTERS]),
        tech(
            "Ghost",
            &[],
            &[Unlock::Recipe(crate::item::IRON_INGOT), Unlock::MachineRecipe(999), Unlock::Upgrade(BELT, 7)],
        ),
        Tech { packs: &[crate::item::IRON_PLATE], units: 0, ..tech("Idle", &[], &[]) },
    ];
    let errors = lint_techs(&BAD);
    for e in [
        "Loop needs a later tech",
        "Twice unlocks Recipe(ItemId(17)) again",
        "Ghost unlocks Recipe(ItemId(256)), which doesn't exist",
        "Ghost unlocks MachineRecipe(999), which doesn't exist",
        "Ghost unlocks Upgrade(12, 7), which doesn't exist",
        "Idle uses a pack labs don't hold",
        "Idle has no work",
    ] {
        assert!(errors.iter().any(|x| x == e), "{e} not in {errors:#?}");
    }
}

#[test]
fn prerequisites_gate_what_can_be_chosen() {
    let mut r = Research::default();
    assert_eq!((r.state(0), r.state(1), r.state(3)), (TechState::Available, TechState::Locked, TechState::Locked));
    r.set_current(Some(1));
    assert_eq!(r.current, None, "a locked tech can't be chosen");
    r.set_current(Some(0));
    assert_eq!(r.current, Some(0));
    finish(&mut r, 0);
    assert_eq!((r.state(0), r.current), (TechState::Done, None), "labs stop when it's done");
    assert_eq!((r.state(1), r.state(2), r.state(3)), (TechState::Available, TechState::Available, TechState::Locked));
    r.set_current(Some(0));
    assert_eq!(r.current, None, "a done tech can't be chosen again");
    r.add_unit(0);
    assert_eq!(r.progress(0), TECHS[0].units, "progress never passes the units");
}

#[test]
fn techs_lock_the_recipes_they_unlock() {
    let mut r = Research::default();
    assert_eq!(r.locked_by(Unlock::Recipe(SPLITTER.into())), Some(0));
    assert_eq!(r.locked_by(Unlock::Recipe(GREEN_PACK)), Some(2));
    assert_eq!(r.locked_by(Unlock::Recipe(BELT.into())), None, "not in the tree: available from the start");
    finish(&mut r, 0);
    assert_eq!(r.locked_by(Unlock::Recipe(SPLITTER.into())), None);
}

/// Every tech that has been done by the time `tech` is: it and its prerequisites, all the way back.
fn closure(tech: usize) -> Vec<bool> {
    let mut done = vec![false; TECHS.len()];
    let mut todo = vec![tech];
    while let Some(t) = todo.pop() {
        if !std::mem::replace(&mut done[t], true) {
            todo.extend(TECHS[t].needs.iter().map(|&n| n as usize));
        }
    }
    done
}

/// What an unlock lets the player make: the items it gives and, per product, the ingredients.
fn unlock_recipes(u: Unlock) -> Vec<(ItemId, Vec<ItemId>)> {
    let hand = |item: ItemId| {
        RECIPES.iter().filter(|r| r.output == item).map(|r| (item, r.inputs.iter().map(|i| i.0).collect())).collect()
    };
    match u {
        Unlock::Recipe(item) => hand(item),
        Unlock::Upgrade(block, tier) => crate::factory::tiers::item_of(block, tier).map_or(vec![], hand),
        Unlock::MachineRecipe(i) => {
            let m = &MACHINE_RECIPES[usize::from(i)];
            m.outputs.iter().map(|o| (o.0, m.inputs.iter().map(|x| x.0).collect())).collect()
        }
        _ => vec![],
    }
}

/// Ingredients a tech may list before the tech that makes them, because what it unlocks is an upgrade or a recipe for
/// something not yet usable: (tech, ingredient). Mechanics lists every family's Mk2, whether or not the player has the
/// base machine yet; the Mk5 port is made from the Mk4 one (Swarm Logistics); washing bauxite needs crushed bauxite;
/// concrete takes sand, which lies about in the world and is only also a crusher product; the tints and bricks of
/// Architecture take whatever the player has found (slag, sulfur) whenever they get it; blue pigment takes a copper ingot,
/// smelted from ore from the start (Ore Crushing only adds a second way, from crushed ore).
const EARLY_INGREDIENTS: &[(&str, &str)] = &[
    ("Pigments", "Copper Ingot"),
    ("Architecture", "Iron Ingot"),
    ("Architecture", "Copper Ingot"),
    ("Mechanics", "Pump"),
    ("Mechanics", "Forester"),
    ("Mk5 Machines", "Drone Port Mk4"),
    ("Ore Washing", "Crushed Bauxite"),
    ("Assembly", "Sand"),
    ("Architecture", "Slag"),
    ("Architecture", "Sulfur"),
];

/// Whether `item` has no tech that unlocks a way to make it: raw, found, or crafted by hand from the start.
fn made_from_the_start(item: ItemId) -> bool {
    let unlocked = TECHS.iter().any(|t| t.unlocks.iter().any(|&u| unlock_recipes(u).iter().any(|p| p.0 == item)));
    !unlocked
        || RECIPES.iter().any(|r| r.output == item && !TECHS.iter().any(|t| t.unlocks.contains(&Unlock::Recipe(item))))
}

#[test]
fn every_ingredient_comes_from_a_tech_before_the_tech_that_needs_it() {
    let mut errors = Vec::new();
    for (i, tech) in TECHS.iter().enumerate() {
        let before = closure(i);
        for &u in tech.unlocks {
            for (product, ingredients) in unlock_recipes(u) {
                for x in ingredients.into_iter().filter(|&x| !made_from_the_start(x)) {
                    let makers = |t: &Tech| t.unlocks.iter().any(|&v| unlock_recipes(v).iter().any(|p| p.0 == x));
                    let ready = TECHS.iter().enumerate().any(|(j, t)| before[j] && makers(t));
                    let name = crate::item::name(x);
                    if !ready && !EARLY_INGREDIENTS.contains(&(tech.name, name)) {
                        errors.push(format!(
                            "{}: {} needs {name}, which no earlier tech unlocks",
                            tech.name,
                            crate::item::name(product)
                        ));
                    }
                }
            }
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn research_reads_back_what_it_wrote() {
    let mut r = Research::default();
    finish(&mut r, 0);
    (0..7).for_each(|_| r.add_unit(2));
    r.set_current(Some(2));
    let mut w = ByteWriter::default();
    r.write_state(&mut w);
    assert_eq!(Research::read_state(&mut ByteReader::new(&w.bytes)), Some(r));
}

#[test]
fn forestry_is_a_violet_tech_that_comes_after_the_violet_pack_is_made() {
    let by_name = |name: &str| TECHS.iter().position(|t| t.name == name).unwrap() as u8;
    let forestry = &TECHS[by_name("Forestry") as usize];
    assert_eq!(forestry.needs, &[by_name("Violet Science")]);
    assert!(forestry.packs.contains(&VIOLET_PACK) && !forestry.packs.contains(&GOLD_PACK));
}

#[test]
fn the_drone_is_the_end_of_a_long_violet_ladder() {
    let by_name = |name: &str| TECHS.iter().position(|t| t.name == name).unwrap() as u8;
    let (processors, robotics, power, nav, drones) = (
        by_name("Processors"),
        by_name("Robotics"),
        by_name("Drone Power"),
        by_name("Navigation"),
        by_name("Construction Drones"),
    );
    let mut r = Research::default();
    let mut steps = 0;
    // Finish prerequisites one at a time until the drones tech is available: every step is a tech to do.
    while r.state(drones) != TechState::Available {
        let next = (0..TECHS.len() as u8).find(|&t| r.state(t) == TechState::Available && t != drones).unwrap();
        finish(&mut r, next);
        steps += 1;
    }
    assert!(steps >= 12, "only {steps} techs stand in front of the drone");
    for t in [processors, robotics, power, nav, drones] {
        assert_eq!(r.state(t), if t == drones { TechState::Available } else { TechState::Done });
        assert!(TECHS[t as usize].packs.contains(&VIOLET_PACK), "{} needs violet packs", TECHS[t as usize].name);
    }
    assert!(TECHS[drones as usize].units >= 200, "the last rung is the dearest");
    assert!(
        r.locked_by(Unlock::MachineRecipe(crate::recipes::DRONE_RECIPES[5])).is_some()
            == (r.state(drones) != TechState::Done)
    );
}
