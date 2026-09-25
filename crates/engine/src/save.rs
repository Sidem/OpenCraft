//! The world save file: a header, the seed, the core (`Sim::write_state`, the same bytes the state
//! hash covers), then what the authority keeps (bodies and loose items). Everything else, such as
//! meshes, links, deposit members and the camera, is derived and rebuilt on load.
//!
//! Layout: magic `OCW1`, `SAVE_VERSION`, `WORLDGEN_VERSION`, seed, core, local player id, bodies,
//! loose items. Invariants: loading refuses, with a message a player can read, any file from another
//! format or generation version and any damaged file; it never panics on bad bytes. Until 1.0, a
//! format change just bumps `SAVE_VERSION` (old saves are refused). To save something new: write and
//! read it in its type's `write_state` / `read_state`, then bump `SAVE_VERSION`.

use crate::bytes::{ByteReader, ByteWriter};
use crate::entities::Items;
use crate::player::Player;
use crate::sim::PlayerId;
use crate::worldgen::WORLDGEN_VERSION;
use crate::Game;

/// The format of everything after the header. Bump on any change to what is written.
pub const SAVE_VERSION: u32 = 1;
const MAGIC: &[u8] = b"OCW1";
const DAMAGED: &str = "This world file is damaged and can't be loaded.";

impl Game {
    /// The whole world as save bytes.
    pub(crate) fn save_bytes(&self) -> Vec<u8> {
        let mut w = ByteWriter::default();
        w.bytes.extend_from_slice(MAGIC);
        w.u32(SAVE_VERSION);
        w.u32(WORLDGEN_VERSION);
        w.u32(self.sim.world.generator().seed());
        self.sim.write_state(&mut w);
        w.u8(self.local.0);
        w.count(self.bodies.len());
        for body in &self.bodies {
            w.bool(body.is_some());
            if let Some(body) = body {
                body.write_state(&mut w);
            }
        }
        self.items.write_state(&mut w);
        w.bytes
    }

    /// A game from save bytes, or why it can't be loaded, in words for the player.
    pub(crate) fn from_save(bytes: &[u8], view_radius: u32) -> Result<Game, String> {
        let mut r = ByteReader::new(bytes);
        if r.bytes(MAGIC.len()) != Some(MAGIC) {
            return Err("This is not an OpenCraft world file.".into());
        }
        let (Some(version), Some(worldgen)) = (r.u32(), r.u32()) else { return Err(DAMAGED.into()) };
        if version > SAVE_VERSION {
            return Err("This world was saved by a newer version of OpenCraft. Reload the page to update.".into());
        }
        if version < SAVE_VERSION {
            return Err("This world was saved by an older version of OpenCraft that can't be loaded any more.".into());
        }
        if worldgen != WORLDGEN_VERSION {
            return Err("World generation has changed since this world was saved, so it can't be loaded.".into());
        }
        read_game(&mut r, view_radius).ok_or_else(|| DAMAGED.into())
    }
}

fn read_game(r: &mut ByteReader, view_radius: u32) -> Option<Game> {
    let mut g = Game::new(r.u32()?, view_radius);
    g.sim.read_state(r)?;
    g.local = PlayerId(r.u8()?);
    let bodies = r.count()?;
    if bodies > 256 {
        return None;
    }
    g.bodies.clear();
    for _ in 0..bodies {
        let present = r.bool()?;
        g.bodies.push(if present { Some(Player::read_state(r)?) } else { None });
    }
    g.items = Items::read_state(r)?;
    // The local player must exist in the core and as a body, and nothing may follow.
    let local_body = g.bodies.get(g.local.0 as usize).is_some_and(Option::is_some);
    if !r.is_done() || !local_body || g.sim.player(g.local).is_none() {
        return None;
    }
    g.prev_eye = g.body().eye();
    g.render_eye = g.prev_eye;
    Some(g)
}

#[cfg(test)]
mod tests;
