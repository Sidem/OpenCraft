//! Co-op lockstep inside the engine (DEV_PLAN section 2, "Co-op"): the game's `Role` and the bytes that
//! travel. Only actions travel; the core (`Sim`) never learns about the network.
//!
//! - `Solo`: an action queues for the next tick, as always.
//! - `Host`: every action (the local player's, peers' through `host_stamp`, the authority's pickups
//!   and joins) is collected; when a tick has run, the collected ones are queued for
//!   `tick + INPUT_DELAY` and written to that tick's frame (`take_frames`). The host's own actions take
//!   the same path, so it has no advantage.
//! - `Client`: the local player's inputs go to the outbox (`take_outbox`); authority actions are
//!   dropped, since the host is the authority. `push_frames` queues each frame's actions just as the
//!   host did and moves `confirmed` on; the core steps only through the confirmed tick.
//!
//! Bytes (`ByteWriter`): an outbox is actions back to back (`action/codec.rs`); frames are, per tick
//! run, the `u64` tick, a count, then (`u8` player, action) each. A damaged buffer is refused whole.
//! Invariant: host and client queue a frame's actions in frame order at `tick + INPUT_DELAY`, so
//! every core applies the same actions in the same order at the same tick. A joiner starts from a
//! snapshot (`snapshot.rs`) and gets every frame from the snapshot's tick on.
//!
//! Beside the lockstep, what players see of each other: body states (`players.rs`) and the host's
//! loose items (`items.rs`). Neither touches the core.

use crate::action::Action;
use crate::bytes::{ByteReader, ByteWriter};
use crate::entities::Items;
use crate::sim::{PlayerId, Sim};
use crate::{Game, MAX_TICKS_PER_FRAME};
use items::ItemView;
use players::Relay;

/// Ticks between the tick an action is stamped in and the tick it applies: time for its frame to
/// reach every client before they need it. Tuned in play within 3–6.
pub const INPUT_DELAY: u64 = 4;
/// Host and clients fingerprint the core after every this many ticks (`take_checksums`); a mismatch
/// means the cores went apart.
pub const CHECKSUM_TICKS: u64 = 60;

pub(crate) enum Role {
    Solo,
    Host(Host),
    Client(Client),
}

#[derive(Default)]
pub(crate) struct Host {
    /// Actions collected since the last tick ran, in arrival order.
    stamped: Vec<(PlayerId, Action)>,
    /// Frames of the ticks run since the last `take_frames`.
    frames: ByteWriter,
    checksums: Vec<u64>,
    /// Players on other machines, whose bodies they move (`players.rs`).
    peers: Vec<PlayerId>,
    relay: Relay,
    /// Each peer's latest view of the items near it (`items.rs`).
    views: Vec<(PlayerId, Vec<u8>)>,
}

pub(crate) struct Client {
    /// The local player's actions not yet taken for the host.
    outbox: ByteWriter,
    /// Ticks the host has sent frames for: the core may run while `sim.tick < confirmed`.
    confirmed: u64,
    checksums: Vec<u64>,
    relay: Relay,
    /// The host's items near this player, as drawn here.
    items: ItemView,
}

impl Role {
    /// Sends an action where this role wants it: queued (solo), collected (host), outbox (client).
    pub(crate) fn route(&mut self, sim: &mut Sim, local: PlayerId, player: PlayerId, action: Action) {
        match self {
            Role::Solo => sim.queue(sim.tick, player, action),
            Role::Host(h) => h.stamped.push((player, action)),
            Role::Client(c) if player == local && action.is_peer_input() => action.write(&mut c.outbox),
            Role::Client(_) => {}
        }
    }

    /// Whether the core may run tick `tick` now (a client waits for the host's frame).
    pub(crate) fn may_step(&self, tick: u64) -> bool {
        match self {
            Role::Client(c) => tick < c.confirmed,
            Role::Solo | Role::Host(_) => true,
        }
    }

    /// After the core ran a tick: every 60th, host and client note (tick, hash); the host queues what
    /// it collected and writes the tick's frame.
    pub(crate) fn end_tick(&mut self, sim: &mut Sim) {
        let checksums = match self {
            Role::Solo => return,
            Role::Host(h) => &mut h.checksums,
            Role::Client(c) => &mut c.checksums,
        };
        if sim.tick.is_multiple_of(CHECKSUM_TICKS) {
            checksums.extend([sim.tick, sim.state_hash()]);
        }
        let Role::Host(h) = self else { return };
        let tick = sim.tick - 1;
        h.frames.u64(tick);
        h.frames.count(h.stamped.len());
        for (player, action) in h.stamped.drain(..) {
            h.frames.u8(player.0);
            action.write(&mut h.frames);
            sim.queue(tick + INPUT_DELAY, player, action);
        }
    }
}

impl Role {
    pub(crate) fn host() -> Role {
        Role::Host(Host::default())
    }

    /// Host: collects actions a peer sent for its player `player`. Refuses the whole buffer (and
    /// returns false) if it is damaged or holds anything but that player's own input.
    pub(crate) fn host_stamp(&mut self, player: PlayerId, bytes: &[u8]) -> bool {
        let Role::Host(h) = self else { return false };
        let mut r = ByteReader::new(bytes);
        let mut actions = Vec::new();
        while !r.is_done() {
            match Action::read(&mut r) {
                Some(a) if a.is_peer_input() => actions.push((player, a)),
                _ => return false,
            }
        }
        h.stamped.extend(actions);
        true
    }

    /// Host: the frames of every tick run since the last call.
    pub(crate) fn take_frames(&mut self) -> Vec<u8> {
        match self {
            Role::Host(h) => std::mem::take(&mut h.frames.bytes),
            _ => Vec::new(),
        }
    }

    /// Host and client: (tick, state hash) pairs, flat, noted since the last call.
    pub(crate) fn take_checksums(&mut self) -> Vec<u64> {
        match self {
            Role::Host(h) => std::mem::take(&mut h.checksums),
            Role::Client(c) => std::mem::take(&mut c.checksums),
            Role::Solo => Vec::new(),
        }
    }

    /// Client: the local player's actions since the last call, for the host.
    pub(crate) fn take_outbox(&mut self) -> Vec<u8> {
        match self {
            Role::Client(c) => std::mem::take(&mut c.outbox.bytes),
            _ => Vec::new(),
        }
    }

    /// Client: queues the host's frames into `sim`. They must follow on from the last one, in order.
    /// Refuses the whole buffer (and returns false) if it is damaged or out of order.
    pub(crate) fn push_frames(&mut self, sim: &mut Sim, bytes: &[u8]) -> bool {
        let Role::Client(c) = self else { return false };
        let Some((frames, confirmed)) = read_frames(bytes, c.confirmed) else { return false };
        c.confirmed = confirmed;
        for (tick, player, action) in frames {
            sim.queue(tick + INPUT_DELAY, player, action);
        }
        true
    }

    /// The tick the host has confirmed up to (`core_tick` outside a client).
    pub(crate) fn confirmed(&self, core_tick: u64) -> u64 {
        match self {
            Role::Client(c) => c.confirmed,
            _ => core_tick,
        }
    }
}

impl Game {
    /// Plays as `local` in the host's world, from this core's tick on. Only the local body stays: the
    /// host moves everyone else. That is `local`'s body if the world has one (a snapshot taken after
    /// the host's `join`), else the body this game showed so far.
    pub(crate) fn become_client(&mut self, local: PlayerId) {
        let own = self.bodies.get_mut(local.0 as usize).and_then(Option::take);
        let body = own.or_else(|| self.bodies[self.local.0 as usize].take());
        self.bodies.clear();
        self.bodies.resize_with(local.0 as usize + 1, || None);
        self.bodies[local.0 as usize] = body;
        self.local = local;
        self.prev_eye = self.body().eye();
        self.render_eye = self.prev_eye;
        // The host owns loose items; this game only draws what it sends.
        self.items = Items::default();
        self.role = Role::Client(Client {
            outbox: ByteWriter::default(),
            confirmed: self.sim.tick,
            checksums: Vec::new(),
            relay: Relay::default(),
            items: ItemView::default(),
        });
    }

    /// Client: runs core ticks the host has confirmed but this game hasn't run yet (frames that came
    /// in a burst), at most `MAX_TICKS_PER_FRAME` per call. The bodies and hands don't run again.
    pub(crate) fn catch_up(&mut self) {
        let Role::Client(_) = self.role else { return };
        for _ in 0..MAX_TICKS_PER_FRAME {
            if !self.role.may_step(self.sim.tick) {
                return;
            }
            self.step_core();
        }
    }
}

/// An action from a frame: the frame's tick, the player, the action.
type FrameAction = (u64, PlayerId, Action);

/// Parses frames starting at tick `next`: every (tick, player, action), and the tick after the last
/// frame. `None` if anything is damaged or a frame is out of order.
fn read_frames(bytes: &[u8], mut next: u64) -> Option<(Vec<FrameAction>, u64)> {
    let mut r = ByteReader::new(bytes);
    let mut out = Vec::new();
    while !r.is_done() {
        let tick = r.u64()?;
        if tick != next {
            return None;
        }
        for _ in 0..r.count()? {
            out.push((tick, PlayerId(r.u8()?), Action::read(&mut r)?));
        }
        next += 1;
    }
    Some((out, next))
}

mod items;
mod players;
mod snapshot;
#[cfg(test)]
mod tests;
