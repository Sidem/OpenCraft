use super::*;
use crate::action::Action;
use crate::block::{DIRT, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::storage::Storage;
use crate::factory::Machine;
use crate::sim::{PlayerId, Sim};
use crate::Game;

fn stack(item: impl Into<ItemId>, count: u32) -> Stack {
    Stack { item: item.into(), count }
}

#[test]
fn a_box_shows_what_it_holds_most_of_or_its_category() {
    let empty = vec![Stack::default(); 24];
    assert_eq!(shown(&empty, AUTO), None, "an empty box shows nothing");
    assert_eq!(shown(&empty, 5), Some(item::IRON_PICKAXE), "a category shows its icon, full or not");

    // Stone is in more stacks, but dirt adds up to more.
    let mut slots = empty.clone();
    slots[0] = stack(STONE, 30);
    slots[3] = stack(DIRT, 64);
    slots[7] = stack(STONE, 20);
    slots[9] = stack(DIRT, 10);
    assert_eq!(shown(&slots, AUTO), Some(DIRT.into()));
    // A tie goes to the item met first.
    slots[9] = stack(STONE, 14);
    assert_eq!(shown(&slots, AUTO), Some(STONE.into()));
    assert_eq!(shown(&slots, 3), Some(item::GEAR));
}

#[test]
fn every_label_names_an_item_that_draws() {
    for (i, &(name, icon)) in LABELS.iter().enumerate() {
        assert!(!name.is_empty());
        if i as u8 == AUTO {
            assert_eq!(icon, ItemId::NONE);
            continue;
        }
        assert!(item::def(icon).is_some(), "{name}");
        let mut out = Vec::new();
        draw(&mut out, icon, Vec3::new(0.0, 0.0, -3.0));
        assert!(out.len() > 2 * crate::factory::INSTANCE_FLOATS, "{name}: a plate, a ledge and the item");
    }
}

#[test]
fn only_the_sides_the_camera_sees_are_drawn_and_only_near() {
    let count = |rel: Vec3| {
        let mut out = Vec::new();
        draw(&mut out, item::GEAR, rel);
        out.len()
    };
    let one = count(Vec3::new(0.0, 0.0, -3.0)); // the camera due south: one side
    assert!(one > 0);
    assert_eq!(count(Vec3::new(3.0, 0.0, -3.0)), 2 * one, "seen from a corner: two sides");
    assert_eq!(count(Vec3::new(0.0, -4.0, 0.0)), 0, "seen from straight above: none");
    assert_eq!(count(Vec3::new(0.0, 0.0, -(NEAR + 1.0))), 0, "too far");
}

#[test]
fn a_label_is_an_action_and_saved_with_the_box() {
    let mut sim = Sim::new(7, 2);
    let pos = IVec3::new(0, 200, 0);
    sim.factory.add_storage(pos);
    assert_eq!(sim.factory.box_label(pos), Some(AUTO));
    let before = sim.state_hash();
    sim.apply(PlayerId(0), Action::LabelBox { pos, label: 7 });
    assert_eq!(sim.factory.box_label(pos), Some(7));
    assert_ne!(sim.state_hash(), before, "the label is core state");
    // No such label, no box: nothing changes.
    sim.apply(PlayerId(0), Action::LabelBox { pos, label: LABELS.len() as u8 });
    sim.apply(PlayerId(0), Action::LabelBox { pos: pos + IVec3::new(1, 0, 0), label: 2 });
    assert_eq!(sim.factory.box_label(pos), Some(7));
    assert_eq!(sim.factory.box_label(pos + IVec3::new(1, 0, 0)), None);

    // The box writes it and reads it back; a label past the table is a damaged file.
    let mut s = Storage::new(pos);
    s.label = 4;
    let mut w = ByteWriter::default();
    s.write_state(&mut w);
    assert_eq!(Storage::read_state(&mut ByteReader::new(&w.bytes)).map(|s| s.label), Some(4));
    let last = w.bytes.len() - 1;
    w.bytes[last] = LABELS.len() as u8;
    assert!(Storage::read_state(&mut ByteReader::new(&w.bytes)).is_none());
}

#[test]
fn a_box_saved_before_labels_shows_what_it_holds() {
    let mut s = Storage::new(IVec3::new(1, 2, 3));
    s.buf.slots[0] = stack(STONE, 5);
    let mut w = ByteWriter::default();
    s.write_state(&mut w);
    w.bytes.pop(); // version 68 ends a box at its round-robin position
    let mut r = ByteReader::new(&w.bytes);
    r.version = 68;
    let old = Storage::read_state(&mut r).expect("loads");
    assert_eq!((old.label, old.buf.slots[0]), (AUTO, stack(STONE, 5)));
}

#[test]
fn both_players_see_the_same_label_in_co_op() {
    let mut host = Game::new(2024, 2);
    host.start_host();
    assert_eq!(host.host_join(0), Ok(1));
    let mut client = Game::new(2024, 2);
    assert!(client.start_client(1));
    let pos = IVec3::new(4, 120, 4);
    for g in [&mut host, &mut client] {
        g.sim.factory.add_storage(pos);
    }
    let mut hashes = vec![host.sim.state_hash()];
    let mut step = |host: &mut Game, client: &mut Game| {
        assert!(host.host_stamp(1, &client.take_outbox()));
        host.run_ticks(1);
        hashes.push(host.sim.state_hash());
        assert!(client.push_frames(&host.take_frames()));
        client.update(1.0 / 60.0);
        assert_eq!(client.sim.state_hash(), hashes[client.sim.tick as usize], "tick {}", client.sim.tick);
    };
    for _ in 0..3 {
        step(&mut host, &mut client);
    }
    client.label_box(pos.x, pos.y, pos.z, 1);
    for _ in 0..6 {
        step(&mut host, &mut client);
    }
    assert_eq!(host.sim.factory.box_label(pos), Some(1), "the client's pick reached the host");
    assert_eq!(client.box_label(pos.x, pos.y, pos.z), 1);
}
