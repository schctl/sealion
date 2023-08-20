//! Piece specific definitions.

use std::ops::Not;

use strum::{EnumCount, EnumIter, FromRepr};

use Color::*;
use PieceKind::*;

/// Represents a player or a piece's color.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, FromRepr)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Get the opposite color for this player.
    #[inline]
    pub const fn opposite(&self) -> Self {
        match self {
            White => Black,
            Black => White,
        }
    }
}

impl Not for Color {
    type Output = Self;

    /// Does [`Color::opposite`].
    #[inline]
    fn not(self) -> Self::Output {
        self.opposite()
    }
}

/// All possible piece types.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, FromRepr)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub const PROMOTABLE: [Self; 4] = [Knight, Bishop, Rook, Queen];

    /// Standard notation for this piece kind.
    #[inline]
    #[rustfmt::skip]
    pub const fn as_char(&self) -> char {
        match self {
            Pawn   => 'P',
            Knight => 'N',
            Bishop => 'B',
            Rook   => 'R',
            Queen  => 'Q',
            King   => 'K',
        }
    }

    /// Piece valuation on some arbitrary scale.
    #[inline]
    #[rustfmt::skip]
    pub const fn score(&self) -> i16 {
        match self {
            Pawn   => 100,
            Knight => 300,
            Bishop => 325,
            Rook   => 500,
            Queen  => 900,
            King   => 10_000, // real
        }
    }

    /// Check if this piece performs "ray" attacks.
    #[inline]
    #[rustfmt::skip]
    pub const fn is_slider(&self) -> bool {
        matches!(self, Bishop | Rook | Queen)
    }
}

/// Piece belonging to a side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    /// Standard representation for this piece.
    #[inline]
    #[rustfmt::skip]
    pub const fn as_char(&self) -> char {
        match self.color {
            Color::White => match self.kind {
                Pawn   => 'P',
                Knight => 'N',
                Bishop => 'B',
                Rook   => 'R',
                Queen  => 'Q',
                King   => 'K',
            },
            Color::Black => match self.kind {
                Pawn   => 'p',
                Knight => 'n',
                Bishop => 'b',
                Rook   => 'r',
                Queen  => 'q',
                King   => 'k',
            },
        }
    }

    #[inline]
    pub const fn from_char(c: char) -> Option<Self> {
        let (color, kind) = match c {
            'P' => (White, Pawn),
            'N' => (White, Knight),
            'B' => (White, Bishop),
            'R' => (White, Rook),
            'Q' => (White, Queen),
            'K' => (White, King),
            // -
            'p' => (Black, Pawn),
            'n' => (Black, Knight),
            'b' => (Black, Bishop),
            'r' => (Black, Rook),
            'q' => (Black, Queen),
            'k' => (Black, King),
            _ => return None,
        };

        Some(Self { color, kind })
    }
}
