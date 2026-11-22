use super::*;
use crate::action::Action;
use crate::block::{COAL_ORE, IRON_ORE, URANIUM_ORE};
use crate::item::{ACID_BARREL, FUEL_CELL, HYDROGEN_BARREL, IRON_INGOT, OXYGEN_BARREL, SILICON};
use crate::sim::{PlayerId, Sim, SimEvent};

#[test]
fn every_form_references_a_valid_item_and_documented_element() {
    for f in FORMS {
        assert!(f.item.is_valid(), "invalid item {:?} for Z={}", f.item, f.z);
        assert!(element_info(f.z).is_some(), "missing ElementInfo for Z={}", f.z);
    }
    for e in ELEMENTS {
        assert!(FORMS.iter().any(|f| f.z == e.z), "element Z={} has no forms", e.z);
    }
    assert!(ACHIEVEMENTS.len() <= 32, "unlocked_mask is u32");
    for a in ACHIEVEMENTS {
        assert!(a.icon.is_valid(), "invalid icon for achievement {}", a.title);
    }
}

#[test]
fn events_record_items_unlock_elements_and_achievements_and_round_trip() {
    let mut c = Codex::default();
    assert_eq!((c.discovered_elements(), c.unlocked_count()), (0, 0));
    c.note(&[
        SimEvent::Gained { player: PlayerId(0), item: IRON_ORE.into(), count: 1 },
        SimEvent::Produced { item: IRON_INGOT, count: 1 },
        SimEvent::Crafted { player: PlayerId(0), item: SILICON, count: 1 },
    ]);
    assert!(c.has(IRON_ORE.into()));
    assert!(c.has(IRON_INGOT));
    assert!(c.has(SILICON));
    assert!(c.element_discovered(26) && c.element_discovered(14));
    assert!(!c.element_discovered(92));
    assert_eq!(c.discovered_elements(), 2);
    // First Strike (0), Iron Age (1), Silicon Valley (4) are unlocked.
    let mask = c.unlocked_mask();
    assert!(mask & (1 << 0) != 0, "First Strike");
    assert!(mask & (1 << 1) != 0, "Iron Age");
    assert!(mask & (1 << 4) != 0, "Silicon Valley");
    assert!(mask & (1 << 12) == 0, "Atomic Era still locked");

    // Round-trip through canonical bytes.
    let mut w = ByteWriter::default();
    c.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    assert_eq!(Codex::read_state(&mut r), Some(c));
}

#[test]
fn full_element_and_split_water_rules_work() {
    let mut c = Codex::default();
    c.record(HYDROGEN_BARREL);
    assert_eq!(c.achievement_progress(10), (1, 2), "Split Water needs both H2 and O2");
    assert!(!c.achievement_done(10));
    c.record(OXYGEN_BARREL);
    assert!(c.achievement_done(10));

    // Uranium has two forms: ore and fuel cell.
    assert_eq!(c.element_forms_done(92), (0, 2));
    c.record(URANIUM_ORE.into());
    assert_eq!(c.element_forms_done(92), (1, 2));
    c.record(FUEL_CELL);
    assert_eq!(c.element_forms_done(92), (2, 2));
    assert!(c.achievement_done(12), "Atomic Era");
}

#[test]
fn sim_step_folds_events_deterministically_and_give_records_items() {
    let (mut a, mut b) = (Sim::new(42, 1), Sim::new(42, 1));
    for s in [&mut a, &mut b] {
        s.queue(0, PlayerId(0), Action::PickUp { item: COAL_ORE.into(), count: 2 });
        s.queue(1, PlayerId(0), Action::PickUp { item: ACID_BARREL, count: 1 });
        s.step();
        s.step();
    }
    assert_eq!(a.state_hash(), b.state_hash());
    assert!(a.codex.has(COAL_ORE.into()));
    assert!(a.codex.has(ACID_BARREL));
    assert!(a.codex.element_discovered(6) && a.codex.element_discovered(16));
}
