//! Saves before version 18 kept smelters and constructors in lists of their own, each with its own
//! layout; these read them into Mk1 processors. Work was whole ticks then (the constructor's
//! thousandths since version 7), so it is scaled to thousandths.

use crate::block::{CONSTRUCTOR, SMELTER};
use crate::bytes::ByteReader;

use super::super::buffer::Buffer;
use super::super::power::FULL_SPEED;
use super::{spec, Processor, Status};

/// A smelter as saved before version 18.
pub fn read_smelter(r: &mut ByteReader) -> Option<Processor> {
    const STATUSES: [Status; 4] = [Status::Working, Status::NoInput, Status::NoFuel, Status::OutputFull];
    let mut p = Processor::new(r.ivec3()?, spec(SMELTER)?, 0);
    p.input = Buffer::read_state(r, 1)?;
    p.fuel = Buffer::read_state(r, 1)?;
    p.out = Buffer::read_state(r, 1)?;
    let (busy, batch) = (r.bool()?, r.u16()?);
    p.batch = busy.then_some(batch);
    p.progress = r.u32()?.saturating_mul(FULL_SPEED);
    p.burn = r.u32()?.saturating_mul(FULL_SPEED);
    p.next_out = r.u32()? as usize;
    p.status = *STATUSES.get(r.u8()? as usize)?;
    p.valid()
}

/// A constructor as saved before version 18 (its statuses were the first five of today's).
pub fn read_constructor(r: &mut ByteReader) -> Option<Processor> {
    let mut p = Processor::new(r.ivec3()?, spec(CONSTRUCTOR)?, 0);
    let (chosen, recipe) = (r.bool()?, r.u16()?);
    p.recipe = chosen.then_some(recipe);
    p.input = Buffer::read_state(r, 1)?;
    p.out = Buffer::read_state(r, 1)?;
    let busy = r.bool()?;
    p.batch = if busy { Some(p.recipe?) } else { None };
    p.progress = r.u32()?;
    if r.version < 7 {
        p.progress = p.progress.saturating_mul(FULL_SPEED);
    }
    p.next_out = r.u32()? as usize;
    p.status = *super::STATUSES[..5].get(r.u8()? as usize)?;
    p.valid()
}
