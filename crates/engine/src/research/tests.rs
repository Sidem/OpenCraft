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
        tech("Ghost", &[], &[Unlock::Recipe(crate::item::IRON_INGOT), Unlock::MachineRecipe(999)]),
        Tech { packs: &[crate::item::IRON_PLATE], units: 0, ..tech("Idle", &[], &[]) },
    ];
    let errors = lint_techs(&BAD);
    for e in [
        "Loop needs a later tech",
        "Twice unlocks Recipe(ItemId(17)) again",
        "Ghost unlocks Recipe(ItemId(256)), which doesn't exist",
        "Ghost unlocks MachineRecipe(999), which doesn't exist",
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
