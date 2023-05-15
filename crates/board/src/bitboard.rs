//! BitBoard utilities.

use std::fmt::Display;

use derive_more::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

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
    BitXor,
    BitXorAssign,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
)]
pub struct BitBoard(pub u64);

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

    /// Check if all bits are set to 0.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl PartialEq<u64> for BitBoard {
    #[inline]
    fn eq(&self, other: &u64) -> bool {
        self.0.eq(other)
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut square = Square::at(7, 0).unwrap();

        for _ in 0..8 {
            for _ in 0..8 {
                write!(f, "{} ", self.get(square) as u8)?;
                square.0 += 1;
            }

            writeln!(f)?;
            square.0 = square.0.saturating_sub(16);
        }

        Ok(())
    }
}
