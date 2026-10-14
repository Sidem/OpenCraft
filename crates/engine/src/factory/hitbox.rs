//! `Factory::hit_boxes`: what the crosshair can hit in a cell holding a slender machine (a pole, belt, splitter,
//! pipe, sensor, data node or rail node). The boxes are the machine's own model (`Machine::model`), so what you
//! see is what you aim at; a machine whose model is empty (a cable's is drawn elsewhere) gets a small cube.
//! Big machines (smelters, labs, ...) are not here: their whole footprint is the hit box.
//! To extend: add the machine's `Slot` arm.

use super::links::Slot;
use super::{Factory, Machine};
use crate::hitbox::Boxes;
use crate::math::{IVec3, Vec3};

/// The side of the cube a cable is hit by.
const CABLE_SIZE: f64 = 0.4;

impl Factory {
    /// The hit boxes of the machine in `cell`, or None when the whole cell counts.
    pub fn hit_boxes(&self, cell: IVec3) -> Option<Boxes> {
        let mut out = Vec::new();
        let here = Vec3::ZERO;
        match *self.at.get(&cell)? {
            Slot::Belt(i) => self.belts[i as usize].frame(&mut out, here, 0.0),
            Slot::Router(i) => self.routers[i as usize].model(&mut out, here, 0.0),
            Slot::Pole(i) => {
                let pole = &self.poles[i as usize];
                if pole.is_cable() {
                    return Some(Boxes::centred(CABLE_SIZE));
                }
                if pole.is_long() {
                    return None; // a tower: its whole footprint counts
                }
                pole.model(&mut out, here, 0.0);
            }
            Slot::Pipe(i) => self.pipework[i as usize].model(&mut out, here, 0.0),
            Slot::Sensor(i) => self.sensors[i as usize].model(&mut out, here, 0.0),
            Slot::Node(i) => self.nodes[i as usize].model(&mut out, here, 0.0),
            Slot::Rail(i) => self.rails[i as usize].model(&mut out, here, 0.0),
            _ => return None,
        }
        Boxes::from_instances(&out)
    }
}

#[cfg(test)]
mod tests;
