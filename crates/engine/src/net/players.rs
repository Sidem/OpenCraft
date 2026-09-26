//! What players see of each other in co-op: body states, and which machine moves
//! which body. Each machine runs physics for its own player's body only (`Role::drives`).
//!
//! 20 times a second (`STATE_TICKS`) a client notes its body's state for the host (`take_states`).
//! The host puts it on that peer's body (`host_state`) and notes every body's state for everyone
//! (`take_states`). A client puts those on the other bodies (`push_states`), adding the new ones and
//! removing the ones no longer listed. `avatars.rs` glides and draws them; `items.rs` rides on the
//! same clock.
//!
//! Bytes: a state is the position (`vec3`), yaw and pitch (`f32`) and flying (`bool`). A client sends
//! one; the host sends a count, then (`u8` player, state) per body. A damaged buffer is refused whole.
//! Invariant: none of this touches the core; bodies are the authority's (authority.rs).
//! To send more about a body: add it to `write_state`, `read_state` and `BodyState::apply`.

use super::items::ITEM_TICKS;
use super::Role;
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::Vec3;
use crate::player::Player;
use crate::sim::PlayerId;
use crate::Game;

/// Body ticks between state updates: 20 a second.
pub const STATE_TICKS: u32 = 3;

/// A role's clock and the latest states not yet taken.
#[derive(Default)]
pub(crate) struct Relay {
    /// Body ticks since the session began.
    clock: u32,
    states: Vec<u8>,
}

impl Role {
    /// Whether another machine moves body `id`, so this one only takes its states.
    pub(crate) fn drives(&self, id: PlayerId, local: PlayerId) -> bool {
        match self {
            Role::Solo => false,
            Role::Host(h) => h.peers.contains(&id),
            Role::Client(_) => id != local,
        }
    }

    /// Host: player `id` plays on another machine from now on (`add`) or no more.
    pub(crate) fn set_peer(&mut self, id: PlayerId, add: bool) {
        let Role::Host(h) = self else { return };
        h.peers.retain(|&p| p != id);
        if add {
            h.peers.push(id);
        }
    }

    /// Host: every body's latest state; client: its own. Empty when nothing new was noted.
    pub(crate) fn take_states(&mut self) -> Vec<u8> {
        match self {
            Role::Host(h) => std::mem::take(&mut h.relay.states),
            Role::Client(c) => std::mem::take(&mut c.relay.states),
            Role::Solo => Vec::new(),
        }
    }
}

impl Game {
    /// Once per body tick, after everything moved: notes states when due, and the host's item views.
    pub(crate) fn net_tick(&mut self) {
        let (relay, host) = match &mut self.role {
            Role::Solo => return,
            Role::Host(h) => (&mut h.relay, true),
            Role::Client(c) => (&mut c.relay, false),
        };
        relay.clock = relay.clock.wrapping_add(1);
        let clock = relay.clock;
        if clock.is_multiple_of(STATE_TICKS) {
            let mut w = ByteWriter::default();
            if host {
                w.count(self.bodies.iter().flatten().count());
                for (slot, body) in self.bodies.iter().enumerate() {
                    if let Some(body) = body {
                        w.u8(slot as u8);
                        write_state(body, &mut w);
                    }
                }
            } else if let Some(Some(body)) = self.bodies.get(self.local.0 as usize) {
                write_state(body, &mut w);
            }
            relay.states = w.bytes;
        }
        if host && clock.is_multiple_of(ITEM_TICKS) {
            self.write_item_views();
        }
    }

    /// Host: a peer's body state (bytes from its `take_states`). False if refused.
    pub(crate) fn host_state_of(&mut self, id: PlayerId, bytes: &[u8]) -> bool {
        if !matches!(self.role, Role::Host(_)) || !self.role.drives(id, self.local) {
            return false;
        }
        let mut r = ByteReader::new(bytes);
        let Some(state) = read_state(&mut r).filter(|_| r.is_done()) else { return false };
        let Some(Some(body)) = self.bodies.get_mut(id.0 as usize) else { return false };
        state.apply(body);
        true
    }

    /// Client: every body's state from the host. Bodies appear and go to match; the local one is
    /// left alone. False (and nothing changed) if the bytes are damaged.
    pub(crate) fn push_body_states(&mut self, bytes: &[u8]) -> bool {
        if !matches!(self.role, Role::Client(_)) {
            return false;
        }
        let mut r = ByteReader::new(bytes);
        let Some(list) = read_list(&mut r).filter(|_| r.is_done()) else { return false };
        let local = self.local.0 as usize;
        for (slot, body) in self.bodies.iter_mut().enumerate() {
            if slot != local && !list.iter().any(|(id, _)| id.0 as usize == slot) {
                *body = None;
            }
        }
        for (id, state) in list {
            let slot = id.0 as usize;
            if slot == local {
                continue;
            }
            if slot >= self.bodies.len() {
                self.bodies.resize_with(slot + 1, || None);
            }
            state.apply(self.bodies[slot].get_or_insert_with(|| Player::new(state.pos)));
        }
        true
    }
}

#[derive(Clone, Copy)]
struct BodyState {
    pos: Vec3,
    yaw: f32,
    pitch: f32,
    flying: bool,
}

impl BodyState {
    fn apply(self, body: &mut Player) {
        body.pos = self.pos;
        body.vel = Vec3::ZERO;
        body.yaw = self.yaw as f64;
        body.pitch = self.pitch as f64;
        body.flying = self.flying;
    }
}

fn write_state(body: &Player, w: &mut ByteWriter) {
    w.vec3(body.pos);
    w.f32(body.yaw as f32);
    w.f32(body.pitch as f32);
    w.bool(body.flying);
}

/// A state, or `None` if damaged (including numbers that aren't finite).
fn read_state(r: &mut ByteReader) -> Option<BodyState> {
    let s = BodyState { pos: r.vec3()?, yaw: r.f32()?, pitch: r.f32()?, flying: r.bool()? };
    let finite = [s.pos.x, s.pos.y, s.pos.z, s.yaw as f64, s.pitch as f64].iter().all(|v| v.is_finite());
    finite.then_some(s)
}

fn read_list(r: &mut ByteReader) -> Option<Vec<(PlayerId, BodyState)>> {
    let mut list = Vec::new();
    for _ in 0..r.count()? {
        list.push((PlayerId(r.u8()?), read_state(r)?));
    }
    Some(list)
}
