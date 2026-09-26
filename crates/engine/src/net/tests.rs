//! Lockstep through bytes: a host `Game` and a client `Game` made from the same seed, wired together
//! only by the buffers `take_outbox` → `host_stamp` and `take_frames` → `push_frames`; and what they
//! see of each other (`take_states`, `host_state`, `push_states`, item views).

use super::items::ITEM_TICKS;
use super::players::STATE_TICKS;
use super::*;
use crate::authority::MAX_PLAYERS;
use crate::block::{MINER, STONE};
use crate::math::{IVec3, Vec3};
use crate::sim::tests::{outcrop, script, SEED};
use crate::tests::run_until_ready;

const A: PlayerId = PlayerId(0);
const B: PlayerId = PlayerId(1);

/// A host (player A) and a client playing as B, who joined before the first tick.
fn pair() -> (Game, Game) {
    let mut host = Game::new(SEED, 2);
    host.start_host();
    assert_eq!(host.host_join(0), Ok(1));
    let mut client = Game::new(SEED, 2);
    assert!(client.start_client(1));
    (host, client)
}

fn frames_of(host: &mut Game, ticks: u32) -> Vec<u8> {
    host.run_ticks(ticks);
    host.take_frames()
}

#[test]
fn host_and_client_stay_in_step_through_bytes() {
    let (top, other, key) = outcrop();
    let log = script(top, other);
    let (mut host, mut client) = pair();
    // The host's hash after every tick, indexed by tick.
    let mut hashes = vec![host.sim.state_hash()];
    let mut checked = 0;
    let mut check = |g: &Game, hashes: &[u64]| {
        let t = g.sim.tick as usize;
        assert_eq!(g.sim.state_hash(), hashes[t], "client and host differ at tick {t}");
        checked += 1;
    };
    // Frames the network is holding back: two stretches arrive late, in one burst each.
    let late = |t: u64| (1000..1100).contains(&t) || (2000..2300).contains(&t);
    let mut held = Vec::new();
    for t in 0..3000u64 {
        for &(_, player, action) in log.iter().filter(|q| q.0 == t && !matches!(q.2, Action::Join { .. })) {
            if player == A {
                host.act(action);
            } else {
                client.act(action);
            }
        }
        if late(t) && t.is_multiple_of(10) {
            client.act(Action::ScrollSlot { delta: 1 });
        }
        assert!(host.host_stamp(1, &client.take_outbox()));
        held.extend(frames_of(&mut host, 1));
        hashes.push(host.sim.state_hash());
        if !late(t) {
            assert!(client.push_frames(&held));
            held.clear();
        }
        // A client frame: its own tick, then catch-up after a burst (at most 8 more ticks).
        client.update(1.0 / 60.0);
        check(&client, &hashes);
        assert!(client.sim.tick <= client.confirmed_tick() as u64);
    }
    // The last burst has been caught up; the client is level with the host.
    assert_eq!(client.sim.tick, host.sim.tick);
    assert!(checked == 3000 && hashes.len() == 3001);
    let checksums = host.take_checksums();
    assert_eq!((checksums.len(), checksums[0], checksums[1]), (100, 60, hashes[60]), "every 60th tick");
    assert_eq!(client.take_checksums(), checksums);

    // The line really runs, on both: A's miner fills A's box, and B's stone and smelter are in place.
    for g in [&mut host, &mut client] {
        assert_eq!(g.sim.world.block_anywhere_or_generate(top + IVec3::new(0, 1, 0)), MINER);
        assert!(g.sim.factory.storage_count_at(top + IVec3::new(3, 1, 0), key.ore.into()) > 10);
        assert_eq!(g.sim.player(B).unwrap().inventory.count(STONE.into()), 2, "B placed one");
    }
    assert_eq!(client.inventory().count(STONE.into()), 2, "the client shows B's inventory");
}

#[test]
fn everyone_waits_the_input_delay_even_the_host() {
    let (mut host, mut client) = pair();
    host.act(Action::Give { item: STONE.into(), count: 1 });
    client.act(Action::Give { item: STONE.into(), count: 2 });
    assert_eq!(client.inventory().count(STONE.into()), 0, "no inventory before B's join applies");
    assert!(host.host_stamp(1, &client.take_outbox()));
    let frames = frames_of(&mut host, INPUT_DELAY as u32);
    assert!(client.push_frames(&frames));
    client.run_ticks(INPUT_DELAY as u32 + 3);
    assert_eq!(client.sim.tick, INPUT_DELAY, "the client waits for frames it doesn't have");
    assert_eq!(host.sim.player(A).unwrap().inventory.count(STONE.into()), 0);

    let frames = frames_of(&mut host, 1);
    assert!(client.push_frames(&frames));
    client.run_ticks(1);
    for g in [&host, &client] {
        assert_eq!(g.sim.player(A).unwrap().inventory.count(STONE.into()), 1);
        assert_eq!(g.sim.player(B).unwrap().inventory.count(STONE.into()), 2);
    }
    assert_eq!(client.inventory().count(STONE.into()), 2);
    assert_eq!(host.sim.state_hash(), client.sim.state_hash());
}

#[test]
fn bad_bytes_are_refused_whole() {
    let (mut host, mut client) = pair();
    // Authority actions never leave a client, and a host refuses them from peers.
    client.act_as(B, Action::PickUp { item: STONE.into(), count: 1 });
    client.act_as(A, Action::Give { item: STONE.into(), count: 1 });
    assert!(client.take_outbox().is_empty());
    let mut w = ByteWriter::default();
    Action::Give { item: STONE.into(), count: 1 }.write(&mut w);
    Action::Join { key: 1 }.write(&mut w);
    assert!(!host.host_stamp(1, &w.bytes), "a Join from a peer");
    assert!(!host.host_stamp(1, &w.bytes[..3]), "a cut action");
    assert!(!host.host_stamp(256, &w.bytes[..7]), "no such player");
    assert!(!client.host_stamp(1, &w.bytes[..7]), "only a host stamps");

    let frames = frames_of(&mut host, 3);
    assert!(!client.push_frames(&frames[..frames.len() - 1]), "a cut frame");
    // Frame 0 is 14 bytes: its tick, a count of one and B's join.
    assert!(!client.push_frames(&frames[14..]), "frames must start at the confirmed tick");
    assert!(!client.push_frames(&[0xff; 40]), "garbage");
    assert!(!host.push_frames(&frames), "only a client takes frames");
    assert_eq!(client.confirmed_tick(), 0.0, "nothing half-applied");
    assert!(client.push_frames(&frames));
    assert!(!client.push_frames(&frames), "the same frames twice");
    assert_eq!(client.confirmed_tick(), 3.0);

    // Solo play has no frames or outbox, and its actions still apply at the next tick.
    let mut solo = Game::new(SEED, 2);
    solo.act(Action::Give { item: STONE.into(), count: 1 });
    solo.run_ticks(1);
    assert!(solo.take_frames().is_empty() && solo.take_outbox().is_empty());
    assert_eq!(solo.inventory().count(STONE.into()), 1);
}

/// One tick for a host and one client (`id`): the client's actions go in, the host's frame comes
/// back, and both cores agree afterwards.
fn step_pair(host: &mut Game, client: &mut Game, id: u32) {
    assert!(host.host_stamp(id, &client.take_outbox()));
    host.run_ticks(1);
    assert!(client.push_frames(&host.take_frames()));
    client.run_ticks(1);
    assert_eq!(client.sim.state_hash(), host.sim.state_hash(), "tick {}", host.sim.tick);
}

#[test]
fn a_client_joins_a_running_factory_and_stays_in_step() {
    let (top, other, key) = outcrop();
    let log = script(top, other);
    let mut host = Game::new(SEED, 2);
    host.start_host();
    // A builds the line and runs it alone for 25 s, with nobody listening to the frames.
    for t in 0..1500 {
        for q in log.iter().filter(|q| q.0 == t && q.1 == A) {
            host.act(q.2);
        }
        host.run_ticks(1);
    }
    // An action still queued for a coming tick must reach the joiner through the snapshot.
    host.act(Action::Give { item: STONE.into(), count: 1 });
    host.run_ticks(1);
    host.take_frames();
    let id = host.host_join(0xabc).unwrap();
    let mut client = Game::from_snapshot(&host.snapshot(), id, 2).expect("starts");
    assert!(client.is_client() && client.local == PlayerId(id as u8));
    assert_eq!(client.sim.state_hash(), host.sim.state_hash());

    let boxed = |g: &Game| g.sim.factory.storage_count_at(top + IVec3::new(3, 1, 0), key.ore.into());
    let before = boxed(&host);
    for t in 0..600 {
        if t % 100 == 10 {
            client.act(Action::Give { item: STONE.into(), count: 2 });
        }
        step_pair(&mut host, &mut client, id);
    }
    assert!(boxed(&client) > before, "the miner kept filling the box");
    assert_eq!(client.inventory().count(STONE.into()), 12);
    assert_eq!(host.sim.player(A).unwrap().inventory.count(STONE.into()), 1, "the queued action applied");
}

#[test]
fn a_player_who_leaves_and_rejoins_gets_their_things_back() {
    let mut host = Game::new(SEED, 2);
    host.start_host();
    let id = host.host_join(77).unwrap();
    let mut client = Game::from_snapshot(&host.snapshot(), id, 2).unwrap();
    client.act(Action::Give { item: STONE.into(), count: 5 });
    client.act(Action::SelectSlot { slot: 3 });
    for _ in 0..10 {
        step_pair(&mut host, &mut client, id);
    }
    let spot = Vec3::new(30.5, 90.0, -12.5);
    host.bodies[id as usize].as_mut().unwrap().pos = spot;
    host.host_leave(id);
    host.run_ticks(10);
    assert!(host.sim.player(PlayerId(id as u8)).is_none());
    assert_eq!((host.sim.away.len(), host.sim.away[0].pos), (1, spot));

    host.take_frames();
    let again = host.host_join(77).unwrap();
    let mut back = Game::from_snapshot(&host.snapshot(), again, 2).unwrap();
    assert_eq!(back.body().pos, spot, "back where they left");
    for _ in 0..INPUT_DELAY + 1 {
        step_pair(&mut host, &mut back, again);
    }
    assert_eq!((back.inventory().count(STONE.into()), back.inventory().selected), (5, 3));
    assert!(host.sim.away.is_empty());

    // A new key starts with nothing.
    let other = host.host_join(78).unwrap();
    host.run_ticks(INPUT_DELAY as u32 + 1);
    assert_eq!(host.sim.player(PlayerId(other as u8)).unwrap().inventory.count(STONE.into()), 0);
}

#[test]
fn a_full_world_and_a_bad_snapshot_are_refused_readably() {
    let mut host = Game::new(SEED, 2);
    host.start_host();
    for key in 1..MAX_PLAYERS as u64 {
        assert!(host.host_join(key).is_ok());
    }
    assert_eq!(host.host_join(99).err().as_deref(), Some("This world is full."));
    host.host_leave(1);
    assert!(host.host_join(99).is_ok(), "a free place can be taken again");

    let snap = host.snapshot();
    let refused = |bytes: &[u8]| Game::from_snapshot(bytes, 1, 2).err().expect("refused");
    assert_eq!(refused(&snap[..snap.len() - 1]), "The host sent a world that can't be read.");
    assert!(!refused(&[1, 2, 3]).is_empty());
    let mut longer = snap.clone();
    longer.push(0);
    assert!(!refused(&longer).is_empty());
    assert!(Game::from_snapshot(&snap, 1, 2).is_ok());
}

/// `pair`, once the ground around the host's spawn is loaded (the client caught up on those ticks).
fn ready_pair() -> (Game, Game) {
    let (mut host, mut client) = pair();
    run_until_ready(&mut host);
    assert!(client.push_frames(&host.take_frames()));
    client.run_ticks((host.sim.tick - client.sim.tick) as u32);
    (host, client)
}

/// Hands the client's latest body state to the host.
fn send_state(client: &mut Game, host: &mut Game) {
    client.run_ticks(STATE_TICKS);
    assert!(host.host_state(1, &client.take_states()));
}

#[test]
fn players_see_each_other_through_state_bytes() {
    let (mut host, mut client) = ready_pair();
    // B hovers above spawn on its own machine: the host shows it there and runs no physics for it,
    // while a body the host runs itself falls.
    let spot = host.spawn + Vec3::new(3.0, 12.0, 0.0);
    client.teleport(spot.x, spot.y, spot.z);
    client.body_mut().yaw = 1.5;
    client.run_ticks(STATE_TICKS);
    let state = client.take_states();
    assert!(!state.is_empty() && client.take_states().is_empty(), "noted once per update");
    assert!(host.host_state(1, &state));
    let c = host.add_player().unwrap() as usize;
    host.bodies[c].as_mut().unwrap().pos = spot;
    host.run_ticks(30);
    let b = host.bodies[1].as_ref().unwrap();
    assert_eq!((b.pos, b.yaw), (spot, 1.5));
    assert!(host.bodies[c].as_ref().unwrap().pos.y < spot.y - 1.0, "the host's own bodies fall");

    // The host sends every body; the client shows the others and keeps its own.
    host.run_ticks(STATE_TICKS);
    let states = host.take_states();
    assert!(client.push_states(&states));
    assert_eq!(client.bodies[0].as_ref().unwrap().pos, host.bodies[0].as_ref().unwrap().pos);
    assert!(client.bodies[c].is_some() && client.body().pos == spot);
    client.update(0.0);
    assert_eq!((client.instance_count(), client.label_count()), (6, 2), "two avatars of three boxes each");
    host.remove_player(c as u32);
    host.run_ticks(STATE_TICKS);
    assert!(client.push_states(&host.take_states()));
    assert!(client.bodies[c].is_none() && client.bodies[0].is_some(), "gone when no longer listed");

    assert!(!host.host_state(c as u32, &state), "only a peer's own body");
    assert!(!host.host_state(1, &state[..state.len() - 1]), "a cut state");
    assert!(!client.host_state(1, &state) && !host.push_states(&states), "the wrong role");
    assert!(!client.push_states(&states[1..]), "damaged states");
}

#[test]
fn items_live_on_the_host_and_clients_draw_them() {
    let (mut host, mut client) = ready_pair();
    let a = host.spawn + Vec3::new(12.0, 0.0, 0.0);
    host.teleport(a.x, a.y, a.z);
    client.act(Action::Give { item: STONE.into(), count: 1 });
    for _ in 0..INPUT_DELAY + 1 {
        step_pair(&mut host, &mut client, 1);
    }
    // B throws the stone: only the host spawns it, from B's body there.
    client.act(Action::DropSelected { count: 1 });
    for _ in 0..INPUT_DELAY + 1 {
        step_pair(&mut host, &mut client, 1);
    }
    assert_eq!((host.items.list.len(), client.items.list.len()), (1, 0));
    assert_eq!(client.inventory().count(STONE.into()), 0);

    // Ten times a second the host sends B the items near it, and the client draws them.
    for _ in 0..ITEM_TICKS {
        step_pair(&mut host, &mut client, 1);
    }
    let view = host.take_item_view(1);
    assert!(!view.is_empty() && host.take_item_view(1).is_empty() && host.take_item_view(0).is_empty());
    assert!(client.push_items(&view));
    client.update(0.0);
    assert_eq!(client.instance_count(), 1);
    assert!(!client.push_items(&view[..view.len() - 1]) && !host.push_items(&view));

    // Once it has landed, B walks onto it: the host's pickup reaches B's inventory through the frames.
    for _ in 0..60 {
        step_pair(&mut host, &mut client, 1);
    }
    let at = host.items.list[0].pos;
    client.teleport(at.x, at.y - 0.9, at.z);
    send_state(&mut client, &mut host);
    for _ in 0..200 {
        step_pair(&mut host, &mut client, 1);
    }
    assert!(host.items.list.is_empty());
    assert_eq!(client.inventory().count(STONE.into()), 1);
}

#[test]
fn the_host_loads_the_ground_around_every_player() {
    let (mut host, mut client) = ready_pair();
    let far = Vec3::new(700.5, 100.0, 300.5);
    let loaded = |host: &mut Game, p: Vec3| {
        host.update(0.0);
        host.begin_work();
        while host.work_step() {}
        host.sim.world.is_loaded(p)
    };
    assert!(!loaded(&mut host, far));
    client.teleport(far.x, far.y, far.z);
    send_state(&mut client, &mut host);
    let spawn = host.spawn;
    assert!(loaded(&mut host, far) && loaded(&mut host, spawn), "around B and still around A");
    let mut meshed_far = false;
    while host.next_event() != 0 {
        meshed_far |= (host.event_x() - 21).abs() < 4;
    }
    assert!(!meshed_far, "only the local player's chunks are meshed");
    host.host_leave(1);
    assert!(!loaded(&mut host, far), "B left");
}

/// Streams `g`'s world in and meshes everything pending.
fn settle(g: &mut Game) {
    run_until_ready(g);
    g.begin_work();
    while g.work_step() {}
    while g.next_event() != 0 {}
}

#[test]
fn a_client_that_went_apart_resyncs_from_a_fresh_snapshot() {
    let (mut host, mut client) = ready_pair();
    settle(&mut client);
    client.debug_desync();
    host.run_ticks(60);
    assert!(client.push_frames(&host.take_frames()));
    client.run_ticks(60);
    assert_eq!(client.sim.tick, host.sim.tick);
    assert_ne!(client.sim.state_hash(), host.sim.state_hash(), "the cores went apart");

    let (loaded, edge) = (client.sim.world.loaded_count(), client.sim.world.dirty_count());
    let feet = client.body().pos;
    assert!(client.resync(&host.snapshot()).is_ok());
    assert_eq!(client.sim.state_hash(), host.sim.state_hash());
    assert_eq!(client.sim.world.loaded_count(), loaded, "nothing streams in again");
    let dirty = client.sim.world.dirty_count() - edge; // edge chunks never mesh
    assert!((1..=27).contains(&dirty), "only around the flipped block: {dirty}");
    assert_eq!(client.body().pos, feet, "the body stays");
    client.begin_work();
    while client.work_step() {}
    assert!(client.next_event() == 1, "the flipped block's chunks are remeshed");

    // Frames follow on from the snapshot's tick, and both stay equal.
    host.act(Action::ScrollSlot { delta: 1 });
    client.act(Action::ScrollSlot { delta: 2 });
    assert!(host.host_stamp(1, &client.take_outbox()));
    host.run_ticks(120);
    assert!(client.push_frames(&host.take_frames()));
    client.run_ticks(120);
    assert_eq!(client.sim.state_hash(), host.sim.state_hash());
    let (mine, theirs) = (client.take_checksums(), host.take_checksums());
    assert!(!mine.is_empty() && theirs.ends_with(&mine), "the checksums since the resync match");
    assert!(host.resync(&host.snapshot()).is_err(), "only clients resync");
}
