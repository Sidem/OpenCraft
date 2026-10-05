//! What a processor is doing, as its lamp and readout show it. Saves store `status as u8`, so statuses are appended
//! to both the enum and [`STATUSES`], never reordered.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    NoRecipe,
    Working,
    NoInput,
    OutputFull,
    NoPower,
    NoFuel,
    /// A boiler with fuel but no water.
    NoWater,
    /// A pumpjack with no oil below it, or one whose reservoir ran dry.
    NoDeposit,
    Exhausted,
    /// A reactor that has shut down until its heat has fallen.
    Overheated,
}

/// Every status, in declaration order.
pub(super) const STATUSES: [Status; 10] = [
    Status::NoRecipe,
    Status::Working,
    Status::NoInput,
    Status::OutputFull,
    Status::NoPower,
    Status::NoFuel,
    Status::NoWater,
    Status::NoDeposit,
    Status::Exhausted,
    Status::Overheated,
];
