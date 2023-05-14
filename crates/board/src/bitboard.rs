//! BitBoard utilities.

use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, ShlAssign, Shr, ShrAssign};

use crate::Square;

/// Represents the 8x8 grid as a bitboard.
#[repr(transparent)]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    // num traits
    Not,
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
)]
pub struct BitBoard(u64);

impl BitBoard {
    /// Get the value at this position.
    #[inline]
    pub const fn get(&self, square: Square) -> bool {
        self.0 & (1 << square.0) != 0
    }

    /// Set the value at this position.
    #[inline]
    pub fn set(&mut self, square: Square, value: bool) {
        let reset = !(1 << square.0);
        let mask = u64::from(value) << square.0;
        self.0 = (self.0 & reset) | mask;
    }
}
