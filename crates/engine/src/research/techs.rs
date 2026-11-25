//! The tech tree as data ([`TECHS`]): 76 techs organised by ring (`red_green.rs`, `blue.rs`, `violet.rs`,
//! `gold_sky.rs`), then the techs added since at the open end whatever their ring (`later.rs`), joined with `crate::join`. Named indices live in `ids.rs`; pre-86 save migration lives in
//! `migrate.rs`.

use crate::join::join;

use super::blue::BLUE;
use super::gold_sky::GOLD_SKY;
use super::later::LATER;
use super::red_green::RED_GREEN;
use super::violet::VIOLET;
use super::Tech;

pub const TECHS: &[Tech] =
    &join::<Tech, { RED_GREEN.len() + BLUE.len() + VIOLET.len() + GOLD_SKY.len() + LATER.len() }>(&[
        &RED_GREEN, &BLUE, &VIOLET, &GOLD_SKY, &LATER,
    ]);
