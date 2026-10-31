use super::*;
use crate::action::Action;
use crate::block::{BLOCK_COUNT, SIGN_COUNT, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::save::SAVE_VERSION;
use crate::sim::SimEvent;
use crate::tests::{belt_test_slab, run_until_ready};
use crate::Game;

const NORTH: u8 = 0;
const AT: IVec3 = IVec3::new(0, 201, 0);
const RIGHT: IVec3 = IVec3::new(1, 201, 0);

/// A game over the test slab with signs in the selected slot.
fn holding_signs() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.slots[0] = Stack { item: SIGN.into(), count: 3 };
    inv.selected = 0;
    g
}

fn place(g: &mut Game, pos: IVec3, facing: u8) {
    g.act(Action::PlaceBlock { pos, slot: 0, facing, against: pos - IVec3::new(0, 1, 0) });
    g.run_ticks(1);
}

fn signs_held(g: &Game) -> u32 {
    g.sim.player(crate::sim::PlayerId(0)).unwrap().inventory.count(SIGN.into())
}

#[test]
fn the_sign_is_the_last_block() {
    assert_eq!(usize::from(SIGN) + SIGN_COUNT, BLOCK_COUNT);
}

#[test]
fn a_sign_takes_its_cell_and_the_one_to_its_placers_right() {
    let mut g = holding_signs();
    place(&mut g, AT, NORTH);
    for c in [AT, RIGHT] {
        assert_eq!(g.sim.world.block_anywhere_or_generate(c), SIGN);
    }
    let s = g.sim.signs.at(RIGHT).expect("found from either cell");
    assert_eq!((s.pos, s.dir, s.back, s.ink), (AT, NORTH, DEFAULT_BACK, DEFAULT_INK));
    assert!(s.lines.iter().all(String::is_empty));
    assert_eq!(signs_held(&g), 2, "one sign used");
    // Facing east the right is south: blocked there by the first sign's... nothing, but by stone here.
    let east = IVec3::new(3, 201, 0);
    g.sim.world.set_block_anywhere(east + IVec3::new(0, 0, 1), STONE);
    place(&mut g, east, 1);
    assert_eq!(g.sim.world.block_anywhere_or_generate(east), crate::block::AIR, "no room for its right half");
    assert_eq!((g.sim.signs.list.len(), signs_held(&g)), (1, 2), "nothing placed, nothing paid");
}

#[test]
fn editing_writes_a_line_and_the_colours_and_refuses_bad_text() {
    let mut g = holding_signs();
    place(&mut g, AT, NORTH);
    g.edit_sign(
        RIGHT.x,
        RIGHT.y,
        RIGHT.z,
        "Iron line\nsmelters → north\n\nfourth\nfifth is dropped",
        0x123456,
        0xffeedd,
    );
    g.run_ticks(1);
    assert_eq!(g.sign_text(AT.x, AT.y, AT.z), "Iron line\nsmelters → north\n\nfourth");
    assert_eq!(g.sign_colours(AT.x, AT.y, AT.z), vec![0x123456, 0xffeedd]);
    let mut bad = line_bytes("ok");
    bad[2] = 0xff;
    for (line, text) in
        [(0, bad), (1, *b"tab\there\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"), (4, line_bytes("no fifth line"))]
    {
        g.sim.signs.edit(AT, line, &text, 0x0100_0007, 0);
    }
    assert_eq!(g.sign_text(AT.x, AT.y, AT.z), "Iron line\nsmelters → north\n\nfourth", "bad lines change nothing");
    let long = "ä".repeat(20);
    assert_eq!(line_text(&line_bytes(&long)), Some("ä".repeat(LINE_BYTES / 2)), "cut at a whole character");
    g.sim.signs.edit(AT, 3, &line_bytes("x"), 0x0100_0007, 0x0200_0000);
    assert_eq!(g.sign_colours(AT.x, AT.y, AT.z), vec![7, 0], "colours keep 24 bits");
}

#[test]
fn breaking_either_cell_takes_the_whole_sign_and_drops_one() {
    for broken in [AT, RIGHT] {
        let mut g = holding_signs();
        place(&mut g, AT, NORTH);
        g.sim.events.clear();
        g.sim.queue(g.sim.tick, crate::sim::PlayerId(0), Action::BreakBlock { pos: broken });
        g.sim.step();
        for c in [AT, RIGHT] {
            assert_eq!(g.sim.world.block_anywhere_or_generate(c), crate::block::AIR);
        }
        assert!(g.sim.signs.list.is_empty());
        let dropped: u32 = g
            .sim
            .events
            .iter()
            .filter_map(|e| match *e {
                SimEvent::Dropped { item, count, .. } if item == ItemId::block(SIGN) => Some(count),
                _ => None,
            })
            .sum();
        assert_eq!(dropped, 1, "breaking {broken:?}");
    }
}

#[test]
fn signs_are_saved_and_older_saves_have_none() {
    let mut g = holding_signs();
    place(&mut g, AT, NORTH);
    g.edit_sign(AT.x, AT.y, AT.z, "Kiln\n\n\nüber", 0xabcdef, 0x010203);
    g.run_ticks(1);
    let mut w = ByteWriter::default();
    g.sim.signs.write_state(&mut w);
    let bytes = w.bytes;
    let read = Signs::read_state(&mut ByteReader::new(&bytes)).expect("reads back");
    assert_eq!(read.list, g.sim.signs.list);
    // A whole save: the sign comes back, at the end of the core (the test slab is not saved, so compare only that).
    let save = g.save();
    let loaded = Game::from_save(&save, 3).expect("loads");
    assert_eq!(loaded.sim.signs.list, g.sim.signs.list);
    for sim in [&g.sim, &loaded.sim] {
        let mut core = ByteWriter::default();
        sim.write_state(&mut core);
        assert!(core.bytes.ends_with(&bytes), "the core ends with the signs");
    }
    assert_eq!(SAVE_VERSION, 63, "signs came with save 63");
    // A core from save 62 ends before the signs' count: it reads whole, with none.
    let mut w = ByteWriter::default();
    crate::sim::Sim::new(2024, 2).write_state(&mut w);
    let old = &w.bytes[..w.bytes.len() - 4];
    let (mut r, mut sim) = (ByteReader::new(old), crate::sim::Sim::new(2024, 2));
    r.version = 62;
    assert!(sim.read_state(&mut r).is_some() && r.is_done(), "a version 62 core");
    assert!(sim.signs.list.is_empty());
    // Damaged bytes are refused, never a panic.
    for (at, b) in [(12, 4), (13, 0xff), (25, 40)] {
        let mut bad = bytes.clone();
        bad[at] = b;
        let _ = Signs::read_state(&mut ByteReader::new(&bad));
    }
    let mut bad = bytes.clone();
    bad[16] = 9;
    assert!(Signs::read_state(&mut ByteReader::new(&bad)).is_none(), "a heading that does not exist");
}

#[test]
fn both_players_see_the_same_sign_in_co_op() {
    let mut host = Game::new(2024, 2);
    host.start_host();
    assert_eq!(host.host_join(0), Ok(1));
    let mut client = Game::new(2024, 2);
    assert!(client.start_client(1));
    let b = crate::sim::PlayerId(1);
    host.act(Action::Give { item: SIGN.into(), count: 1 });
    let mut hashes = vec![host.sim.state_hash()];
    let mut step = |host: &mut Game, client: &mut Game| {
        assert!(host.host_stamp(1, &client.take_outbox()));
        host.run_ticks(1);
        hashes.push(host.sim.state_hash());
        assert!(client.push_frames(&host.take_frames()));
        client.update(1.0 / 60.0);
        assert_eq!(client.sim.state_hash(), hashes[client.sim.tick as usize], "tick {}", client.sim.tick);
    };
    let pos = IVec3::new(4, 120, 4);
    for g in [&mut host, &mut client] {
        g.sim.world.set_block_anywhere(pos - IVec3::new(0, 1, 0), STONE);
    }
    for _ in 0..5 {
        step(&mut host, &mut client);
    }
    let slot =
        host.sim.player(crate::sim::PlayerId(0)).unwrap().inventory.slots.iter().position(|s| s.item == SIGN.into());
    host.act(Action::PlaceBlock { pos, slot: slot.unwrap() as u8, facing: NORTH, against: pos - IVec3::new(0, 1, 0) });
    for _ in 0..10 {
        step(&mut host, &mut client);
    }
    // The client writes it; the host applies the same edit.
    client.edit_sign(pos.x, pos.y, pos.z, "Hello\nfrom B", 0x334455, 0xffffff);
    for _ in 0..10 {
        step(&mut host, &mut client);
    }
    assert!(host.sim.player(b).is_some());
    for g in [&host, &client] {
        assert_eq!(g.sign_text(pos.x, pos.y, pos.z), "Hello\nfrom B\n\n");
        assert_eq!(g.sign_colours(pos.x, pos.y, pos.z), vec![0x334455, 0xffffff]);
    }
}

#[test]
fn the_nearest_signs_get_text_slots_and_keep_them() {
    let mut g = holding_signs();
    g.sim.players[0].as_mut().unwrap().inventory.slots[0].count = 20;
    // Fourteen signs along the slab, two cells apart from x = -2 (step-up cells at y 202 from x = 6).
    let spots: Vec<IVec3> =
        (0..7).flat_map(|i| [IVec3::new(-2 + 2 * i, 201, -1), IVec3::new(-2 + 2 * i, 201, 1)]).collect();
    for &p in &spots {
        let p = if p.x >= 6 { p + IVec3::new(0, 1, 0) } else { p };
        place(&mut g, p, NORTH);
    }
    assert_eq!(g.sim.signs.list.len(), 14);
    g.update(1.0 / 60.0);
    let faces = g.sign_faces();
    let used = faces.chunks(4).filter(|f| f[3] != 0).count();
    assert_eq!(used, view::SLOTS, "every slot in use, two signs bare");
    let first = faces.clone();
    // Editing a sign changes its stamp, not its slot.
    let s = g.sim.signs.list[0].pos;
    g.edit_sign(s.x, s.y, s.z, "changed", DEFAULT_BACK, DEFAULT_INK);
    g.update(1.0 / 60.0);
    g.update(1.0 / 60.0);
    let now = g.sign_faces();
    let slot = first.chunks(4).position(|f| f[..3] == [s.x, s.y, s.z]).expect("the nearest sign has a slot");
    assert_eq!(now[slot * 4..slot * 4 + 3], first[slot * 4..slot * 4 + 3], "same slot");
    assert_ne!(now[slot * 4 + 3], first[slot * 4 + 3], "new stamp");
    // Gone far away, the slots empty.
    g.teleport(500.5, 202.0, 0.5);
    g.update(1.0 / 60.0);
    assert!(g.sign_faces().iter().all(|&v| v == 0));
}

#[test]
fn a_board_is_drawn_against_a_wall_or_on_posts() {
    let mut g = holding_signs();
    place(&mut g, AT, NORTH);
    let boxes = |g: &mut Game| {
        g.update(1.0 / 60.0);
        let n = crate::factory::INSTANCE_FLOATS;
        let layers: Vec<u16> = g.instances.chunks(n).map(|b| b[9] as u16).collect();
        let faces =
            layers.iter().filter(|&&l| (crate::block::tex::SIGN_FACE..=crate::block::tex::SIGN_FACE_LAST).contains(&l));
        (faces.count(), layers.iter().filter(|&&l| l == crate::block::tex::PLANKS).count())
    };
    let free = boxes(&mut g);
    assert_eq!(free, (2, 3), "two face halves; a board on two posts");
    g.sim.world.set_block(AT - IVec3::new(0, 0, 1), STONE);
    assert_eq!(boxes(&mut g), (2, 1), "against a wall: the board alone");
}

#[test]
fn a_blueprint_copies_a_sign_once_turned_and_builds_it_blank() {
    use crate::blueprint::Blueprint;
    use crate::sim::{PlayerId, Sim};
    let mut sim = Sim::new(7, 2);
    let at = IVec3::new(500, 200, -500);
    sim.apply(PlayerId(0), Action::Give { item: SIGN.into(), count: 2 });
    let slot =
        sim.player(PlayerId(0)).unwrap().inventory.slots.iter().position(|s| s.item == SIGN.into()).unwrap() as u8;
    sim.apply(PlayerId(0), Action::PlaceBlock { pos: at, slot, facing: 1, against: at - IVec3::new(0, 1, 0) });
    sim.signs.edit(at, 0, &line_bytes("words stay"), 0x112233, 0);
    let bp = Blueprint::copy(&sim, at - IVec3::new(2, 0, 2), at + IVec3::new(2, 1, 2), "Sign".into()).unwrap();
    assert_eq!(bp.entries.len(), 1, "one entry for both cells");
    assert_eq!((bp.entries[0].block, bp.entries[0].facing), (SIGN, 1));
    let origin = IVec3::new(520, 200, -520);
    let ghost = bp.ghosts_at(origin, 1).into_iter().next().unwrap();
    assert_eq!(ghost.facing, 2, "turned with the stamp");
    assert!(sim.put_block(PlayerId(0), ghost.pos, SIGN.into(), ghost.facing, ghost.pos));
    let built = sim.signs.at(ghost.pos).unwrap();
    assert_eq!((built.dir, built.back, built.lines[0].as_str()), (2, DEFAULT_BACK, ""), "a blank sign");
    assert_eq!(built.cells()[1], ghost.pos + IVec3::new(-1, 0, 0), "facing south its right is west");
}
