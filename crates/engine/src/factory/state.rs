//! The factory's core state bytes: every machine list in `Vec` order, kind by kind, then the deposits
//! and the research.
//! Derived data (`at` is rebuilt while reading; links, order and power at the first `update`) is not
//! saved. To add a kind: append its list to both functions, read behind a `r.version` check.

use rustc_hash::FxHashMap;

use super::{add_to, Factory, Machine, Slot};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::research::Research;
use crate::world::World;

impl Factory {
    /// Core state: every machine in `Vec` order, kind by kind, then the deposits. `at`, `order` and
    /// the links are derived from these.
    pub fn write_state(&self, w: &mut ByteWriter) {
        write_list(w, &self.belts);
        write_list(w, &self.miners);
        write_list(w, &self.storages);
        write_list(w, &self.smelters);
        write_list(w, &self.constructors);
        write_list(w, &self.routers);
        write_list(w, &self.generators);
        write_list(w, &self.poles);
        write_list(w, &self.labs);
        self.deposits.write_state(w);
        self.research.write_state(w);
    }

    /// Reads what `write_state` wrote; `world` must already hold the saved edits (deposits survey it).
    /// Links are rebuilt at the first `update`. Two machines in one place is damage. Saves before
    /// version 3 have no smelters, before 4 no constructors, before 5 no routers, before 7 no power, before 8 no labs or research.
    pub fn read_state(world: &mut World, r: &mut ByteReader) -> Option<Factory> {
        let mut f = Factory { dirty: true, ..Factory::default() };
        read_list(r, &mut f.belts, &mut f.at, Slot::Belt)?;
        read_list(r, &mut f.miners, &mut f.at, Slot::Miner)?;
        read_list(r, &mut f.storages, &mut f.at, Slot::Storage)?;
        if r.version >= 3 {
            read_list(r, &mut f.smelters, &mut f.at, Slot::Smelter)?;
        }
        if r.version >= 4 {
            read_list(r, &mut f.constructors, &mut f.at, Slot::Constructor)?;
        }
        if r.version >= 5 {
            read_list(r, &mut f.routers, &mut f.at, Slot::Router)?;
        }
        if r.version >= 7 {
            read_list(r, &mut f.generators, &mut f.at, Slot::Generator)?;
            read_list(r, &mut f.poles, &mut f.at, Slot::Pole)?;
        }
        if r.version >= 8 {
            read_list(r, &mut f.labs, &mut f.at, Slot::Lab)?;
        }
        f.deposits.read_state(world, r)?;
        if r.version >= 8 {
            f.research = Research::read_state(r)?;
        }
        Some(f)
    }
}

fn write_list<T: Machine>(w: &mut ByteWriter, list: &[T]) {
    w.count(list.len());
    list.iter().for_each(|m| m.write_state(w));
}

/// Reads one kind's list; a position already taken is damage.
fn read_list<T: Machine>(
    r: &mut ByteReader,
    list: &mut Vec<T>,
    at: &mut FxHashMap<IVec3, Slot>,
    slot: fn(u32) -> Slot,
) -> Option<()> {
    for _ in 0..r.count()? {
        let m = T::read_state(r)?;
        if at.contains_key(&m.pos()) {
            return None;
        }
        add_to(list, m, at, slot);
    }
    Some(())
}
