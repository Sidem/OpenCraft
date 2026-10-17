//! The vault in some ruins (R5c, generator version 10, `worldgen/ruins.rs`): a door of `RINGS` dials, each showing a digit
//! 0 to 4. Read as a number in base 5 (first ring last digit), the dials open the door when their square ends in the
//! door's own number, `…44444` (that is -1 in base 5). Ring k is lit when the last k digits already fit, so the first ring
//! is found by trying (2 or 3) and then every ring has exactly one right digit. Nothing here is core state: the door is a
//! block, and opening it (`Action::OpenVault`) turns it into air and drops `LOOT`.
//! To change the puzzle: `RINGS` (a ring more makes 5 times the space for the same steps) and `LOOT`.

use super::{PlayerId, Sim};
use crate::block::{AIR, VAULT_DOOR};
use crate::inventory::Stack;
use crate::item::{ItemId, CIRCUIT, MOTOR, SCRAP_METAL, STEEL_PLATE};
use crate::math::IVec3;

/// Dials on a door.
pub const RINGS: usize = 6;
const BASE: u32 = 5;
/// Modest loot: a few of the better parts, nothing that skips an era.
const LOOT: [(ItemId, u32); 4] = [(CIRCUIT, 5), (STEEL_PLATE, 4), (MOTOR, 1), (SCRAP_METAL, 8)];

/// The dials' digits packed as a number: ring `i` (0 is the first) is digit `i` in base 5.
pub fn pack(digits: &[u8]) -> u32 {
    digits.iter().take(RINGS).rev().fold(0, |n, &d| n * BASE + (d as u32).min(BASE - 1))
}

/// The squared number's digits, ring order (the first is the last digit in base 5).
pub fn square_digits(dials: u32) -> [u8; RINGS] {
    let modulus = BASE.pow(RINGS as u32) as u64;
    let mut sq = (dials as u64 * dials as u64) % modulus;
    std::array::from_fn(|_| {
        let d = (sq % BASE as u64) as u8;
        sq /= BASE as u64;
        d
    })
}

/// How many rings are lit: the number of last digits of the square that equal the door's (all 4s).
pub fn lit_rings(dials: u32) -> usize {
    square_digits(dials).iter().take_while(|&&d| d == BASE as u8 - 1).count()
}

impl Sim {
    /// Turns the rings of the door at `pos` to `dials`: if they fit, the door opens and its loot drops.
    pub(crate) fn open_vault(&mut self, _player: PlayerId, pos: IVec3, dials: u32) {
        if lit_rings(dials) < RINGS || self.world.block_anywhere_or_generate(pos) != VAULT_DOOR {
            return;
        }
        // The door is two blocks tall: both go.
        for at in [pos + IVec3::new(0, 1, 0), pos + IVec3::new(0, -1, 0), pos] {
            if self.world.block_anywhere_or_generate(at) == VAULT_DOOR {
                self.world.set_block_anywhere(at, AIR);
            }
        }
        let loot = LOOT.iter().map(|&(item, count)| Stack { item, count }).collect();
        self.drop_stacks(pos, loot);
    }
}

#[cfg(test)]
mod tests;
