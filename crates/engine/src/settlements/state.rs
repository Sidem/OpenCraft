//! The towns' save format (Sim::write_state ends with it): per town its depot, kind, stage, order progress, stall,
//! dormancy, plots, cursor, drones and a harbour's product and payment, then what the machines have made. Each field
//! arrived with a save version (48 towns, 49 stages and orders, 50 plots and drones, 51 dormancy, 53 the harbour); a new field goes on the end of a town's record behind the version that adds it.

use crate::bytes::{ByteReader, ByteWriter};

use super::build::TownDrone;
use super::perks::harbour::{price, Made};
use super::plots::Plot;
use super::{Settlements, Town, HARBOUR, KINDS, STAGES};

impl Settlements {
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.towns.len());
        for t in &self.towns {
            w.ivec3(t.pos);
            w.u8(t.kind);
            w.u8(t.built);
            w.count(t.delivered.len());
            t.delivered.iter().for_each(|&d| w.u32(d));
            w.bool(t.stalled);
            w.u64(t.due);
            w.bool(t.dormant);
            w.u8(t.planned);
            w.count(t.plots.len());
            t.plots.iter().for_each(|p| p.write(w));
            w.u16(t.cursor);
            w.count(t.drones.len());
            t.drones.iter().for_each(|d| d.write(w));
            w.item(t.product);
            w.count(t.paid.len());
            t.paid.iter().for_each(|&p| w.u32(p));
        }
        self.made.write_state(w);
    }

    /// Reads what `write_state` wrote (a town from before save version 49 has just its depot); `None` on a kind or
    /// stage that doesn't exist, or deliveries that don't match the open order.
    pub fn read_state(r: &mut ByteReader) -> Option<Settlements> {
        let mut towns = Vec::new();
        for _ in 0..r.count()? {
            let (pos, kind) = (r.ivec3()?, r.u8()?);
            if kind as usize >= KINDS.len() {
                return None;
            }
            let mut town = Town::new(pos, kind);
            if r.version >= 49 {
                town.built = r.u8()?;
                town.delivered = (0..r.count()?).map(|_| r.u32()).collect::<Option<_>>()?;
                (town.stalled, town.due) = (r.bool()?, r.u64()?);
                if town.built as usize > STAGES || town.delivered.len() != town.order().len() {
                    return None;
                }
            }
            if r.version >= 50 {
                town.dormant = r.version >= 51 && r.bool()?;
                town.planned = r.u8()?.min(town.built);
                town.plots = (0..r.count()?).map(|_| Plot::read(r)).collect::<Option<_>>()?;
                town.cursor = r.u16()?;
                town.drones = (0..r.count()?).map(|_| TownDrone::read(r)).collect::<Option<_>>()?;
            }
            if r.version >= 53 {
                let product = r.item()?;
                let paid: Vec<u32> = (0..r.count()?).map(|_| r.u32()).collect::<Option<_>>()?;
                // A product no longer offered (the recipes changed) is dropped; a changed price starts over.
                if let Some(price) = price(product).filter(|_| town.kind == HARBOUR) {
                    town.product = product;
                    town.paid = if paid.len() == price.len() { paid } else { vec![0; price.len()] };
                }
            }
            towns.push(town);
        }
        let made = if r.version >= 53 { Made::read_state(r)? } else { Made::default() };
        Some(Settlements { towns, made })
    }
}
