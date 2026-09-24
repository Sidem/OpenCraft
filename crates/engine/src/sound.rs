//! Gameplay sound events. The engine decides *what* is heard and *where*; the host synthesises and
//! spatialises the audio. Events are read zero-copy from wasm memory once per frame, then cleared.

use crate::math::Vec3;

/// Event kinds. Must match `Sfx` in `web/src/audio.ts`.
pub const DIG: u8 = 0;
pub const BREAK: u8 = 1;
pub const PLACE: u8 = 2;
pub const STEP: u8 = 3;
pub const LAND: u8 = 4;
pub const PICKUP: u8 = 5;
pub const DROP: u8 = 6;

/// Floats per event: kind, sound material, camera-relative x, y, z, volume (0..1).
pub const EVENT_FLOATS: usize = 6;
/// Safety cap in case the host stops draining (e.g. a background tab).
const MAX_EVENTS: usize = 64;

#[derive(Default)]
pub struct Sounds {
    buf: Vec<f32>,
}

impl Sounds {
    pub fn push(&mut self, kind: u8, material: u8, rel: Vec3, volume: f64) {
        if self.buf.len() >= MAX_EVENTS * EVENT_FLOATS {
            return;
        }
        self.buf.extend_from_slice(&[
            f32::from(kind),
            f32::from(material),
            rel.x as f32,
            rel.y as f32,
            rel.z as f32,
            volume.clamp(0.0, 1.0) as f32,
        ]);
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.buf.as_ptr()
    }

    pub fn count(&self) -> usize {
        self.buf.len() / EVENT_FLOATS
    }

    #[cfg(test)]
    pub fn kinds(&self) -> Vec<u8> {
        self.buf.chunks(EVENT_FLOATS).map(|e| e[0] as u8).collect()
    }
}
