use super::*;
use crate::block::{BELT, SPLITTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::recipes::{MACHINE_RECIPES, RECIPES};

fn finish(r: &mut Research, tech: u8) {
    (0..TECHS[tech as usize].units).for_each(|_| r.add_unit(tech));
}

/// Returns whether following `needs` from `start` reaches a cycle.
fn has_cycle(techs: &[Tech], start: usize) -> bool {
    let mut state = vec![0u8; techs.len()]; // 0 unvisited, 1 on stack, 2 done
    fn dfs(techs: &[Tech], u: usize, state: &mut [u8]) -> bool {
        state[u] = 1;
        for &n in techs[u].needs {
            let v = n as usize;
            if v >= techs.len() {
                continue;
            }
            if state[v] == 1 || (state[v] == 0 && dfs(techs, v, state)) {
                return true;
            }
        }
        state[u] = 2;
        false
    }
    dfs(techs, start, &mut state)
}

/// Content lint for a tech table: prerequisites exist and form no cycles (so every tech is reachable), packs are in
/// `PACKS`, every unlock exists and is listed once, and there is work to do.
fn lint_techs(techs: &[Tech]) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, t) in techs.iter().enumerate() {
        if !t.needs.iter().all(|&n| (n as usize) < techs.len()) {
            errors.push(format!("{} needs an unknown tech", t.name));
        } else if has_cycle(techs, i) {
            errors.push(format!("{} has a prerequisite cycle", t.name));
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
        Tech {
            name,
            blurb: "",
            ring: Ring::Red,
            wedge: Wedge::Logistics,
            deed: None,
            needs,
            packs: &[RED_PACK],
            units: 1,
            seconds: 1.0,
            unlocks,
        }
    }
    const SPLITTERS: Unlock = Unlock::Recipe(ItemId::block(SPLITTER));
    const BAD: [Tech; 4] = [
        tech("Loop", &[0], &[]),
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
        "Loop has a prerequisite cycle",
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
    assert_eq!(
        (r.state(ids::BELT_ROUTING), r.state(ids::MECHANICS), r.state(ids::ASSEMBLY)),
        (TechState::Available, TechState::Locked, TechState::Locked)
    );
    // Deeds cannot be chosen for labs.
    r.set_current(Some(ids::BELT_ROUTING));
    assert_eq!(r.current, None, "a deed can't be chosen for labs");
    // Open the Green ring: Mechanics becomes available, Assembly still waits for Mechanics + Masonry.
    r.open_ring(Ring::Green);
    assert_eq!((r.state(ids::MECHANICS), r.state(ids::ASSEMBLY)), (TechState::Available, TechState::Locked));
    r.set_current(Some(ids::ASSEMBLY));
    assert_eq!(r.current, None, "a locked tech can't be chosen");
    r.set_current(Some(ids::MECHANICS));
    assert_eq!(r.current, Some(ids::MECHANICS));
    finish(&mut r, ids::MECHANICS);
    assert_eq!((r.state(ids::MECHANICS), r.current), (TechState::Done, None), "labs stop when it's done");
    finish(&mut r, ids::MASONRY);
    assert_eq!(r.state(ids::ASSEMBLY), TechState::Available);
    r.set_current(Some(ids::MECHANICS));
    assert_eq!(r.current, None, "a done tech can't be chosen again");
    r.add_unit(ids::MECHANICS);
    assert_eq!(r.progress(ids::MECHANICS), TECHS[ids::MECHANICS as usize].units, "progress never passes the units");
}

#[test]
fn techs_lock_the_recipes_they_unlock() {
    let mut r = Research::default();
    assert_eq!(r.locked_by(Unlock::Recipe(SPLITTER.into())), Some(ids::BELT_ROUTING));
    assert!(!r.has(Unlock::Recipe(GREEN_PACK)), "green pack waits for half of the red ring");
    assert_eq!(r.locked_by(Unlock::Recipe(BELT.into())), None, "not in the tree: available from the start");
    r.complete_deed(ids::BELT_ROUTING);
    assert_eq!(r.locked_by(Unlock::Recipe(SPLITTER.into())), None);
    finish(&mut r, ids::UNDERPASSES);
    finish(&mut r, ids::MASONRY);
    assert!(r.has(Unlock::Recipe(GREEN_PACK)), "half of the red ring (3 of 6) unlocks the green pack recipe");
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
    ("Glass", "Iron Ingot"),
    ("Glass", "Copper Ingot"),
    ("Glass", "Sand"),
    ("Chip Fab", "Sand"),
    ("Mechanics", "Pump"),
    ("Mechanics", "Forester"),
    ("Mk5 Kits", "Drone Port Mk4"),
    ("Mk5 Kits", "Pumpjack"),
    ("Mk4 Kits", "Seismic Thumper"),
    ("Mk5 Kits", "Seismic Thumper Mk2"),
    ("Ore Washing", "Crushed Bauxite"),
    ("Assembly", "Sand"),
    ("Pigments", "Sand"),
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
fn every_declared_need_is_real() {
    // Physical infrastructure dependencies that aren't item ingredients (e.g. piped water, compute grid, ghost mode).
    const INFRA: &[(&str, &str)] = &[
        ("Steam Power", "Fluid Handling"),
        ("Ore Washing", "Fluid Handling"),
        ("Oil Refining", "Fluid Handling"),
        ("Electrolysis", "Fluid Handling"),
        ("Nuclear Power", "Fluid Handling"),
        ("Datacenters", "Fluid Handling"),
        ("Datacenters", "Data Network"),
        ("AI Labs", "Datacenters"),
        ("Optimizer", "Datacenters"),
        ("Drone Swarms", "Datacenters"),
        ("Photonics", "Data Network"),
        ("Orbital Comms", "Data Network"),
        ("Orbital Science", "Data Network"),
        ("Drop Pods", "Orbital Comms"),
        ("Earthworks", "Construction Drones"),
        ("Auto-Routing", "Construction Drones"),
        ("AI Survey", "Advanced Scanning"),
        ("AI Survey", "Processors"),
        ("Diesel Power", "Oil Refining"),
        ("Flamethrower", "Oil Refining"),
        ("Natural Gas", "Acids and Lubricants"),
    ];
    let mut errors = Vec::new();
    for (i, tech) in TECHS.iter().enumerate() {
        for &need in tech.needs {
            let b = &TECHS[need as usize];
            if is_bonus(i as u8) && need == ids::AI_LABS {
                continue;
            }
            if INFRA.contains(&(tech.name, b.name)) {
                continue;
            }
            // 1. Does `tech` unlock a recipe/upgrade whose ingredient (or base block / kit) is unlocked by `b`?
            let uses_item = tech.unlocks.iter().any(|&u| {
                if let Unlock::Upgrade(block, tier) = u {
                    if b.unlocks.contains(&Unlock::Recipe(ItemId::block(block)))
                        || (tier > 1 && b.unlocks.contains(&Unlock::Upgrade(block, tier - 1)))
                    {
                        return true;
                    }
                    if let Some(kit) = crate::factory::upgrades::kit(tier) {
                        if b.unlocks.iter().any(|&v| unlock_recipes(v).iter().any(|p| p.0 == kit)) {
                            return true;
                        }
                    }
                }
                unlock_recipes(u).iter().any(|(_, ings)| {
                    ings.iter().any(|&x| {
                        !made_from_the_start(x) && b.unlocks.iter().any(|&v| unlock_recipes(v).iter().any(|p| p.0 == x))
                    })
                })
            });
            // 2. Does `tech` unlock a machine recipe whose category runs in a machine unlocked by `b`?
            let uses_machine = tech.unlocks.iter().any(|&u| match u {
                Unlock::MachineRecipe(r) => {
                    let cat = MACHINE_RECIPES[usize::from(r)].category;
                    b.unlocks.iter().any(|&v| match v {
                        Unlock::Recipe(item) => {
                            let blk = crate::item::def(item).map_or(crate::block::AIR, |d| d.places);
                            blk != crate::block::AIR && crate::factory::makes(blk, cat)
                        }
                        _ => false,
                    })
                }
                _ => false,
            });
            if !uses_item && !uses_machine {
                errors.push(format!("{} declares need {} without using its items or machines", tech.name, b.name));
            }
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn research_reads_back_what_it_wrote() {
    let mut r = Research::default();
    r.complete_deed(ids::BELT_ROUTING);
    r.open_ring(Ring::Green);
    (0..7).for_each(|_| r.add_unit(ids::MECHANICS));
    r.set_current(Some(ids::MECHANICS));
    let mut w = ByteWriter::default();
    r.write_state(&mut w);
    let mut reader = ByteReader::new(&w.bytes);
    reader.version = crate::save::SAVE_VERSION;
    assert_eq!(Research::read_state(&mut reader), Some(r));
}

#[test]
fn forestry_is_a_green_tech() {
    let forestry = &TECHS[ids::FORESTRY as usize];
    assert_eq!(forestry.ring, Ring::Green);
    assert_eq!(forestry.packs, &[RED_PACK, GREEN_PACK]);
}

#[test]
fn the_drone_is_the_end_of_a_violet_ladder() {
    let (processors, robotics, drones) = (ids::PROCESSORS, ids::ROBOTICS, ids::CONSTRUCTION_DRONES);
    let mut r = Research::default();
    r.open_ring(Ring::Violet);
    let mut steps = 0;
    while r.state(drones) != TechState::Available {
        let next =
            (0..TECHS.len() as u8).find(|&t| r.state(t) == TechState::Available && t != drones && !is_deed(t)).unwrap();
        finish(&mut r, next);
        steps += 1;
    }
    assert!(steps >= 6, "only {steps} techs stand in front of the drone");
    for t in [processors, robotics, drones] {
        assert_eq!(r.state(t), if t == drones { TechState::Available } else { TechState::Done });
        assert!(TECHS[t as usize].packs.contains(&VIOLET_PACK), "{} needs violet packs", TECHS[t as usize].name);
    }
    assert!(TECHS[drones as usize].units >= 200, "the last rung is the dearest");
    assert_eq!(
        r.locked_by(Unlock::MachineRecipe(crate::recipes::DRONE_RECIPES[5])).is_some(),
        r.state(drones) != TechState::Done
    );
}

#[test]
fn scanning_progression_spans_red_blue_violet_and_gold_tiers() {
    let t_scan = &TECHS[ids::ORE_SCANNING as usize];
    let t_adv = &TECHS[ids::ADVANCED_SCANNING as usize];
    let t_seis = &TECHS[ids::SEISMIC_PROSPECTING as usize];
    assert_eq!((t_scan.ring, t_scan.packs), (Ring::Red, &[RED_PACK][..]));
    assert!(t_scan.unlocks.contains(&Unlock::Recipe(crate::item::SCANNER)));
    assert_eq!(t_adv.packs, &[RED_PACK, GREEN_PACK, BLUE_PACK]);
    assert!(t_adv.needs.contains(&ids::ORE_SCANNING));
    assert!(t_adv.unlocks.contains(&Unlock::Recipe(crate::item::SCANNER_MK2)));
    assert_eq!(t_seis.packs, &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK]);
    assert!(TECHS[ids::MK4_KITS as usize].unlocks.contains(&Unlock::Upgrade(crate::block::THUMPER, 1)));
    assert!(TECHS[ids::MK5_KITS as usize].unlocks.contains(&Unlock::Upgrade(crate::block::THUMPER, 2)));
}

#[test]
fn ring_pack_unlocks_at_half_the_previous_ring_and_opens_on_first_pack() {
    let mut r = Research::default();
    assert!(!r.ring_open(Ring::Green));
    assert!(!r.ring_pack_unlocked(Ring::Green));
    // Red has 6 entries (2 deeds + 4 Red-pack lab techs): finishing any 3 unlocks the Green Pack recipe.
    r.complete_deed(ids::BELT_ROUTING);
    finish(&mut r, ids::UNDERPASSES);
    assert!(!r.ring_pack_unlocked(Ring::Green), "2 of 6 is below half");
    finish(&mut r, ids::MASONRY);
    assert!(r.ring_pack_unlocked(Ring::Green), "3 of 6 unlocks the Green Pack recipe");
    assert!(!r.ring_open(Ring::Green));
    // Crafting the first Green Pack opens the Green ring!
    r.note(&[crate::sim::SimEvent::Crafted { player: crate::sim::PlayerId(0), item: GREEN_PACK, count: 1 }], &[]);
    assert!(r.ring_open(Ring::Green));
    assert_eq!(r.state(ids::MECHANICS), TechState::Available);
}

#[test]
fn red_deeds_complete_by_laying_belts_and_climbing_steps() {
    use crate::math::IVec3;
    use crate::sim::{PlayerId, SimEvent};
    let mut r = Research::default();
    // Lay 40 belts -> completes Belt Routing.
    for i in 0..BELT_DEED_TARGET {
        r.note(&[SimEvent::BlockPlaced { player: PlayerId(0), pos: IVec3::new(i as i32, 0, 0), block: BELT }], &[]);
    }
    assert_eq!(r.state(ids::BELT_ROUTING), TechState::Done);
    assert_eq!(r.state(ids::BELT_LIFTS), TechState::Available);
    // Run a connected belt line up 3 vertical steps -> completes Belt Lifts.
    let chain = [IVec3::new(0, 0, 0), IVec3::new(1, 1, 0), IVec3::new(2, 2, 0), IVec3::new(3, 3, 0)];
    r.note(&[SimEvent::BlockPlaced { player: PlayerId(0), pos: chain[3], block: BELT }], &chain);
    assert_eq!(r.state(ids::BELT_LIFTS), TechState::Done);
}

#[test]
fn pre_86_research_migrates_old_progress_science_gates_merges_and_splits() {
    let mut w = ByteWriter::default();
    w.u8(85); // old current = Architecture
    w.count(migrate::OLD_TECH_COUNT);
    let mut old = vec![0u32; migrate::OLD_TECH_COUNT];
    old[2] = 30; // Green Science done -> opens Green ring
    old[10] = 50; // Blue Science done -> opens Blue ring
    old[85] = 40; // Architecture done -> Stone Styles, Glass, Metalwork, Paint
    old[74] = 40; // Handcrafting I done -> Work Gloves
    old[63] = 7; // Mining Productivity level 7 -> index 65 level 7
    for p in old {
        w.u32(p);
    }
    w.count(0); // empty queue
    let mut reader = ByteReader::new(&w.bytes);
    reader.version = 85;
    let r = Research::read_state(&mut reader).expect("migrates cleanly");
    assert!(r.ring_open(Ring::Blue));
    for t in [ids::STONE_STYLES, ids::GLASS, ids::METALWORK, ids::PAINT, ids::WORK_GLOVES] {
        assert_eq!(r.state(t), TechState::Done, "{}", TECHS[t as usize].name);
    }
    assert_eq!(r.level(Bonus::Mining), 7);
}
