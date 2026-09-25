//! Byte encoding of game state: `ByteWriter` builds the canonical form that `Sim::state_hash`
//! fingerprints and saves store (`save.rs`); `ByteReader` reads it back.
//!
//! Invariants: little-endian fixed-width integers, floats by their exact bits, collection lengths as
//! `u32` before the items. The same state must always give the same bytes, so writers iterate `Vec`s
//! or sorted keys, never hash-map order. Each type writes itself in a `write_state` method next to
//! its fields, and reads itself back in a `read_state` beside it; derived data (links, caches,
//! anything rebuilt on load) is left out. Reads return `None` on anything malformed (past the end, an
//! unknown block id, an impossible value), so a damaged save fails cleanly instead of panicking later.
//! A reader knows the save version it reads (`version`), so a read can follow an older layout.

use crate::block::{BlockId, BLOCK_COUNT};
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::save::SAVE_VERSION;

#[derive(Default)]
pub struct ByteWriter {
    pub bytes: Vec<u8>,
}

// The primitives are not inlined: dozens of call sites would each carry a copy (1.2 KB of wasm).
impl ByteWriter {
    #[inline(never)]
    pub fn u8(&mut self, v: u8) {
        self.bytes.push(v);
    }

    pub fn bool(&mut self, v: bool) {
        self.u8(v as u8);
    }

    #[inline(never)]
    pub fn u16(&mut self, v: u16) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }

    #[inline(never)]
    pub fn u32(&mut self, v: u32) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }

    #[inline(never)]
    pub fn u64(&mut self, v: u64) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i32(&mut self, v: i32) {
        self.u32(v as u32);
    }

    pub fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }

    pub fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }

    /// A collection length, written before its items.
    pub fn count(&mut self, n: usize) {
        self.u32(n as u32);
    }

    pub fn ivec3(&mut self, v: IVec3) {
        self.i32(v.x);
        self.i32(v.y);
        self.i32(v.z);
    }

    pub fn vec3(&mut self, v: Vec3) {
        self.f64(v.x);
        self.f64(v.y);
        self.f64(v.z);
    }

    pub fn item(&mut self, v: ItemId) {
        self.u16(v.0);
    }
}

/// Reads what a `ByteWriter` wrote, in the same order. Its primitives are not inlined either (0.4 KB).
pub struct ByteReader<'a> {
    bytes: &'a [u8],
    pos: usize,
    /// The save format being read (`SAVE_VERSION` unless a save header says otherwise).
    pub version: u32,
}

impl<'a> ByteReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        ByteReader { bytes, pos: 0, version: SAVE_VERSION }
    }

    /// The next `n` raw bytes.
    pub fn bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let out = self.bytes.get(self.pos..end)?;
        self.pos = end;
        Some(out)
    }

    /// True once everything has been read (a save with bytes left over is damaged).
    pub fn is_done(&self) -> bool {
        self.pos == self.bytes.len()
    }

    #[inline(never)]
    pub fn u8(&mut self) -> Option<u8> {
        Some(self.bytes(1)?[0])
    }

    pub fn bool(&mut self) -> Option<bool> {
        match self.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    #[inline(never)]
    pub fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.bytes(2)?.try_into().ok()?))
    }

    #[inline(never)]
    pub fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.bytes(4)?.try_into().ok()?))
    }

    #[inline(never)]
    pub fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.bytes(8)?.try_into().ok()?))
    }

    pub fn i32(&mut self) -> Option<i32> {
        Some(self.u32()? as i32)
    }

    pub fn f32(&mut self) -> Option<f32> {
        Some(f32::from_bits(self.u32()?))
    }

    #[inline(never)]
    pub fn f64(&mut self) -> Option<f64> {
        Some(f64::from_bits(self.u64()?))
    }

    /// A collection length. Every item takes at least a byte, so a length the rest of the data can't
    /// hold is damage (and would otherwise reserve absurd amounts of memory).
    pub fn count(&mut self) -> Option<usize> {
        let n = self.u32()? as usize;
        (n <= self.bytes.len() - self.pos).then_some(n)
    }

    pub fn ivec3(&mut self) -> Option<IVec3> {
        Some(IVec3::new(self.i32()?, self.i32()?, self.i32()?))
    }

    pub fn vec3(&mut self) -> Option<Vec3> {
        Some(Vec3::new(self.f64()?, self.f64()?, self.f64()?))
    }

    /// A block id that exists (unknown ids would index past the block tables).
    #[inline(never)]
    pub fn block(&mut self) -> Option<BlockId> {
        self.u8().filter(|&b| (b as usize) < BLOCK_COUNT)
    }

    /// An item id in the table, or `ItemId::NONE`. Version 1 saves stored items as block ids (`u8`).
    #[inline(never)]
    pub fn item(&mut self) -> Option<ItemId> {
        let id = if self.version < 2 { ItemId::block(self.block()?) } else { ItemId(self.u16()?) };
        (id == ItemId::NONE || id.is_valid()).then_some(id)
    }
}

/// 64-bit FNV-1a: tiny, and stable across platforms and versions.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
    }
    h
}
