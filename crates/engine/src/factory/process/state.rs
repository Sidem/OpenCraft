//! A processor's bytes in a save (`Machine::write_state` and `read_state` forward here): what every processor writes,
//! then what only some specs have (tanks, store, then the kind's own: `extra.rs`). Older layouts are read by version: no facing before 19, a launch site's satellite
//! slot from 45 (`launch.rs`), a town depot's perk buffers from 52 (`depot.rs`), a pod machine's channel from 54 (`pods/channels.rs`).
//! To save something new: write it last, behind its spec's condition, and read it behind the same and a version check.

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;

use super::super::buffer::Buffer;
use super::status::STATUSES;
use super::{depot, launch, pods, spec, Processor};

impl Processor {
    /// Core state: block, tier and facing, recipe, batch, buffers, work, fire, round-robin position and status,
    /// then the byproduct buffer and its round-robin position if its spec has one (`outs` come from `relink`).
    pub(super) fn save(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.spec.block);
        w.u8(self.tier);
        w.u8(self.dir);
        for i in [self.recipe, self.batch] {
            w.bool(i.is_some());
            w.u16(i.unwrap_or(0));
        }
        self.input.write_state(w);
        self.fuel.write_state(w);
        self.out.write_state(w);
        w.u32(self.progress);
        w.u32(self.burn);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
        if self.spec.side > 0 {
            self.side.write_state(w);
            w.u32(self.next_side as u32);
        }
        self.write_tanks(w);
        self.write_store(w);
        self.extra.write_state(w);
    }

    pub(super) fn load(r: &mut ByteReader) -> Option<Processor> {
        let pos = r.ivec3()?;
        let spec = spec(r.u8()? as BlockId)?;
        let tier = r.u8()?;
        if tier as usize >= spec.tiers.len() {
            return None;
        }
        let mut p = Processor::new(pos, spec, tier);
        // Saves before version 19 had no facing: every processor then was one cell facing north.
        p.dir = if r.version >= 19 { r.u8()? % 4 } else { 0 };
        p.recipe = read_opt(r)?;
        p.batch = read_opt(r)?;
        let [input, fuel, out] = spec.buffers;
        let old_depot = depot::before_perks(spec, r.version);
        p.input = Buffer::read_state(r, if old_depot { 0 } else { launch::saved_input_slots(spec, r.version) })?;
        p.input.slots.resize(input, Stack::default());
        p.fuel = Buffer::read_state(r, fuel)?;
        p.out = Buffer::read_state(r, out)?;
        p.progress = r.u32()?;
        p.burn = r.u32()?;
        p.next_out = r.u32()? as usize;
        p.status = *STATUSES.get(r.u8()? as usize)?;
        if spec.side > 0 && !old_depot {
            p.side = Buffer::read_state(r, spec.side)?;
            p.next_side = r.u32()? as usize;
        }
        p.read_tanks(r)?;
        p.read_store(r)?;
        // Before channels were chosen, the way a launcher or pad faced was its channel.
        if r.version < pods::CHANNEL_SAVED {
            p.set_channel(p.dir);
        }
        p.extra.read_state(r)?;
        p.valid()
    }
}

/// An optional recipe index as written: a flag, then the index (0 for none).
fn read_opt(r: &mut ByteReader) -> Option<Option<u16>> {
    let (some, i) = (r.bool()?, r.u16()?);
    Some(some.then_some(i))
}
