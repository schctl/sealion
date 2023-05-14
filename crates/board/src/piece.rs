//! Piece specific definitions.

use std::ops::Not;

use strum::{EnumCount, FromRepr};

/// Represents a player or a piece's color.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, FromRepr)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Get the opposite color for this player.
    #[inline]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, FromRepr)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
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
                PieceKind::Pawn   => 'P',
                PieceKind::Knight => 'N',
                PieceKind::Bishop => 'B',
                PieceKind::Rook   => 'R',
                PieceKind::Queen  => 'Q',
                PieceKind::King   => 'K',
            },
            Color::Black => match self.kind {
                PieceKind::Pawn   => 'p',
                PieceKind::Knight => 'n',
                PieceKind::Bishop => 'b',
                PieceKind::Rook   => 'r',
                PieceKind::Queen  => 'q',
                PieceKind::King   => 'k',
            },
        }
    }

    #[inline]
    pub const fn from_char(c: char) -> Option<Self> {
        let (color, kind) = match c {
            'P' => (Color::White, PieceKind::Pawn),
            'N' => (Color::White, PieceKind::Knight),
            'B' => (Color::White, PieceKind::Bishop),
            'R' => (Color::White, PieceKind::Rook),
            'Q' => (Color::White, PieceKind::Queen),
            'K' => (Color::White, PieceKind::King),
            // -
            'p' => (Color::Black, PieceKind::Pawn),
            'n' => (Color::Black, PieceKind::Knight),
            'b' => (Color::Black, PieceKind::Bishop),
            'r' => (Color::Black, PieceKind::Rook),
            'q' => (Color::Black, PieceKind::Queen),
            'k' => (Color::Black, PieceKind::King),
            _ => return None,
        };

        Some(Self { color, kind })
    }
}
