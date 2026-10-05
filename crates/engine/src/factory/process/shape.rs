//! Where a processor is: the cells of its footprint, where items go in and the faces it gives them out of (the
//! footprint is `spec.footprint`, turned by `dir`: `footprint/`).

use crate::math::IVec3;

use super::super::footprint::Role;
use super::Processor;

impl Processor {
    /// Every cell it occupies, its anchor `pos` first.
    pub fn cells(&self) -> Vec<IVec3> {
        self.spec.footprint.cells(self.pos, self.dir)
    }

    /// Whether items arriving into `cell` from the cell `from` beside it go in (through a port).
    pub fn takes_from(&self, cell: IVec3, from: IVec3) -> bool {
        self.spec.footprint.takes(self.pos, self.dir, cell, from)
    }

    /// The faces it gives items out of: a cell and the `DIRS` index it faces.
    pub fn out_faces(&self) -> Vec<(IVec3, u8)> {
        self.spec.footprint.faces(self.pos, self.dir, Role::Out)
    }

    /// The faces it gives byproducts out of.
    pub fn side_faces(&self) -> Vec<(IVec3, u8)> {
        self.spec.footprint.faces(self.pos, self.dir, Role::Side)
    }
}
