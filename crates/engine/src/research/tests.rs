use super::*;
use crate::block::{BELT, SPLITTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::recipes::RECIPES;

fn finish(r: &mut Research, tech: u8) {
    (0..TECHS[tech as usize].units).for_each(|_| r.add_unit(tech));
}

#[test]
fn the_tech_table_is_consistent() {
    for (i, t) in TECHS.iter().enumerate() {
        assert!(t.needs.iter().all(|&n| (n as usize) < i), "{}: prerequisites come earlier (no cycles)", t.name);
        assert!(t.packs.iter().all(|&p| pack_slot(p).is_some()), "{}: packs are in PACKS", t.name);
        assert!(t.unlocks.iter().all(|&u| RECIPES.iter().any(|r| r.output == u)), "{}: unlocks a recipe", t.name);
        assert!(t.units > 0 && t.seconds > 0.0);
    }
    assert!(TECHS.len() < u8::MAX as usize);
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
    assert_eq!(r.locked_by(SPLITTER.into()), Some(0));
    assert_eq!(r.locked_by(GREEN_PACK), Some(2));
    assert_eq!(r.locked_by(BELT.into()), None, "not in the tree: available from the start");
    finish(&mut r, 0);
    assert_eq!(r.locked_by(SPLITTER.into()), None);
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
