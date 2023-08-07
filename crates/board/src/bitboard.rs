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
    pub const ZERO: Self = Self(0);

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

    /// A bitboard with one square set to 1.
    #[inline]
    pub const fn from_square(square: Square) -> Self {
        Self(1 << square.raw_index())
    }

    /// Try to resolve this bitboard into a singular square.
    #[inline]
    pub const fn to_square_unchecked(&self) -> Square {
        Square::from_index_unchecked(self.0.trailing_zeros() as u8)
    }

    /// An iterator over all `set` squares on the board.
    #[inline]
    pub const fn set_iter(&self) -> SetIter {
        SetIter { inner: *self }
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

/// An iterator over all `set` squares on the board.
#[must_use]
#[derive(Debug, Clone)]
pub struct SetIter {
    inner: BitBoard,
}

impl Iterator for SetIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let trailing = self.inner.0.trailing_zeros() as u8;

        if trailing < 64 {
            let sq = Square::from_index_unchecked(trailing);
            let sq_bb = BitBoard::from_square(sq);
            self.inner &= !sq_bb;
            return Some(sq);
        }

        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.len();
        (count, Some(count))
    }
}

impl ExactSizeIterator for SetIter {
    #[inline]
    fn len(&self) -> usize {
        self.inner.0.count_ones() as usize
    }
}

pub mod constants {
    use super::*;

    pub const A_FILE: BitBoard = BitBoard(0x01_01_01_01_01_01_01_01);
    pub const H_FILE: BitBoard = BitBoard(0x80_80_80_80_80_80_80_80);
}
