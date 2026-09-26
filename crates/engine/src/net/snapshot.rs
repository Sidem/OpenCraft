//! Join snapshots: what a client needs to start playing in the host's world. It is the save bytes
//! (`save.rs`) plus the actions already queued for coming ticks, which a save leaves out.
//!
//! Invariant: the host takes a snapshot between frames, and the joiner then gets every frame from
//! the snapshot's tick on. Actions the host collected but hasn't framed yet (the joiner's own `Join`
//! among them) arrive in that first frame, so nothing is missed or applied twice.
//! Bytes: the save's length (`u32`), the save, then `Sim::write_pending`. A client that went apart
//! from the host takes a fresh snapshot the same way (`resync_from`).

use super::Role;
use crate::bytes::{ByteReader, ByteWriter};
use crate::sim::PlayerId;
use crate::Game;

const DAMAGED: &str = "The host sent a world that can't be read.";

impl Game {
    /// Host: the world as a joiner needs it, taken now (between frames).
    pub(crate) fn snapshot_bytes(&self) -> Vec<u8> {
        let save = self.save_bytes();
        let mut w = ByteWriter::default();
        w.count(save.len());
        w.bytes.extend_from_slice(&save);
        self.sim.write_pending(&mut w);
        w.bytes
    }

    /// A client playing as `local` from a host's `snapshot`, or why it can't start, in words for the
    /// player.
    pub(crate) fn read_snapshot(bytes: &[u8], local: PlayerId, view_radius: u32) -> Result<Game, String> {
        let mut r = ByteReader::new(bytes);
        let save = r.count().and_then(|n| r.bytes(n)).ok_or(DAMAGED)?;
        let mut g = Game::from_save(save, view_radius)?;
        if g.sim.read_pending(&mut r).is_none() || !r.is_done() {
            return Err(DAMAGED.into());
        }
        g.become_client(local);
        Ok(g)
    }

    /// Client: starts over from a fresh host snapshot, after the cores went apart or this game fell
    /// far behind. Only the core is replaced: the local body, the view, the loaded chunks (remeshed
    /// where they differ) and the items drawn stay. Frames then follow from the snapshot's tick.
    pub(crate) fn resync_from(&mut self, bytes: &[u8]) -> Result<(), String> {
        if !matches!(self.role, Role::Client(_)) {
            return Err("Only a co-op client can resync.".into());
        }
        let fresh = Game::read_snapshot(bytes, self.local, 2)?;
        if fresh.sim.world.generator().seed() != self.sim.world.generator().seed() {
            return Err(DAMAGED.into());
        }
        let old = std::mem::replace(&mut self.sim, fresh.sim);
        self.sim.world.adopt_loaded(old.world);
        if let Role::Client(c) = &mut self.role {
            c.confirmed = self.sim.tick;
            c.checksums.clear();
        }
        self.surveyed = None;
        Ok(())
    }
}
