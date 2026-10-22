//! A launcher's or pad's channel: one of the power-grid palette's colours (`grid_colour::PALETTE`, `pods::CHANNELS` of
//! them), chosen in its panel (`Action::SetChannel`) and kept in `Extra::Channel`. A pod goes only to a pad of its
//! launcher's colour. Before save version [`CHANNEL_SAVED`] the way the machine faced was its channel: an old save keeps
//! those four routes as the palette's first four colours (`state.rs`).
//!
//! To add a channel: a row in `PALETTE` and a layer in `tex::GRID_COLOURS`; nothing here changes.

use crate::factory::grid_colour::PALETTE;
use crate::factory::links::Slot;
use crate::factory::Factory;
use crate::math::IVec3;
use crate::pods::CHANNELS;

use super::super::extra::Extra;
use super::super::Processor;

/// The first save version that stores a channel.
pub(in crate::factory::process) const CHANNEL_SAVED: u32 = 54;

/// What a channel is called ("blue"); an unknown one is the first's.
pub fn channel_name(channel: u8) -> &'static str {
    PALETTE[usize::from(channel).min(PALETTE.len() - 1)].0
}

/// A channel's colour as 0xRRGGBB (the maps draw launchers and pads in it).
pub fn channel_colour(channel: u8) -> u32 {
    let [r, g, b] = PALETTE[usize::from(channel).min(PALETTE.len() - 1)].1;
    u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)
}

impl Processor {
    /// The channel of a launcher or pad (0 for another processor).
    pub fn channel(&self) -> u8 {
        match self.extra {
            Extra::Channel(channel) => channel,
            _ => 0,
        }
    }

    /// Tunes a launcher or pad to `channel` (nothing for another processor or a channel that does not exist).
    pub(in crate::factory::process) fn set_channel(&mut self, channel: u8) {
        if let (Extra::Channel(now), true) = (&mut self.extra, channel < CHANNELS) {
            *now = channel;
        }
    }
}

impl Factory {
    /// Tunes the launcher or pad at `pos` to `channel` (nothing changes for another machine).
    pub fn set_channel(&mut self, pos: IVec3, channel: u8) {
        if let Some(&Slot::Process(i)) = self.at.get(&pos) {
            self.processors[i as usize].set_channel(channel);
        }
    }

    /// The channel of the launcher or pad at `pos` (`None` for another machine). Presentation (the panel).
    pub fn channel_at(&self, pos: IVec3) -> Option<u8> {
        match self.at.get(&pos) {
            Some(&Slot::Process(i)) => {
                let p = &self.processors[i as usize];
                (p.is_launcher() || p.is_pad()).then(|| p.channel())
            }
            _ => None,
        }
    }
}
