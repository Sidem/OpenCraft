//! Byte encoding of the core state: `ByteWriter` builds the canonical form that `Sim::state_hash`
//! fingerprints (and that the save format stores, from step 1.6).
//!
//! Invariants: little-endian fixed-width integers, floats by their exact bits, collection lengths as
//! `u32` before the items. The same state must always give the same bytes, so writers iterate `Vec`s
//! or sorted keys, never hash-map order. Each type writes itself in a `write_state` method next to
//! its fields; derived data (links, caches, anything rebuilt on load) is left out.

use crate::math::IVec3;

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
}

/// 64-bit FNV-1a: tiny, and stable across platforms and versions.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
    }
    h
}
