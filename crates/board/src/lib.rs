use std::fmt::Display;
use std::str::FromStr;

use strum::EnumCount;

pub mod bitboard;
pub mod fen;
pub mod piece;
pub mod position;

pub use bitboard::*;
pub use piece::*;
pub use position::*;

/// A position on the board.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub(crate) u8);

impl Square {
    /// The square at a particular rank and file.
    #[inline]
    pub const fn at(rank: u8, file: u8) -> Option<Self> {
        if rank > 7 || file > 7 {
            return None;
        }

        Some(Self(rank * 8 + file))
    }

    /// Rank of this square.
    #[inline]
    pub const fn rank(&self) -> u8 {
        self.0 / 8
    }

    /// File of this square.
    #[inline]
    pub const fn file(&self) -> u8 {
        self.0 % 8
    }
}

impl TryFrom<(u8, u8)> for Square {
    type Error = ();

    /// Determine a square from a (rank, file) pair.
    #[inline]
    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        Self::at(value.0, value.1).ok_or(())
    }
}

impl FromStr for Square {
    type Err = ();

    /// Determine a square's position from algebraic notation.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(());
        }

        let rank = s.as_bytes()[0];
        let rank = rank
            .overflowing_sub(if rank > b'H' { b'a' } else { b'A' })
            .0;

        let file = s.as_bytes()[1];
        let file = file.overflowing_sub(b'1').0;

        Self::try_from((rank, file))
    }
}

impl Display for Square {
    /// Format the square into algebraic notation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            (self.rank() + b'a') as char,
            (self.file() + b'1') as char
        )
    }
}

/// Represents the board and all the pieces on it.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    /// Color masks.
    color_bb: [BitBoard; Color::COUNT],
    /// Piece masks.
    piece_bb: [BitBoard; PieceKind::COUNT],
}

impl Board {
    /// Get the color at a certain square.
    pub fn get_color(&self, square: Square) -> Option<Color> {
        self.color_bb.iter().enumerate().find_map(|(n, bb)| {
            if bb.get(square) {
                Color::from_repr(n as u8)
            } else {
                None
            }
        })
    }

    /// Get the piece type at a certain square.
    pub fn get_piece_kind(&self, square: Square) -> Option<PieceKind> {
        self.piece_bb.iter().enumerate().find_map(|(n, bb)| {
            if bb.get(square) {
                PieceKind::from_repr(n as u8)
            } else {
                None
            }
        })
    }

    /// Get the piece at a certain square.
    pub fn get(&self, square: Square) -> Option<Piece> {
        let (color, kind) = self.get_color(square).zip(self.get_piece_kind(square))?;
        Some(Piece { color, kind })
    }

    /// Get the bitboard associated with a certain piece.
    #[inline]
    pub fn get_bb(&self, piece: Piece) -> BitBoard {
        self.piece_bb[piece.kind as u8 as usize] & self.color_bb[piece.color as u8 as usize]
    }

    /// Get the bitboard associated with a certain piece kind.
    #[inline]
    pub const fn get_piece_bb(&self, piece: PieceKind) -> BitBoard {
        self.piece_bb[piece as u8 as usize]
    }

    /// Get an exclusive reference to the bitboard associated with a certain piece kind.
    #[inline]
    pub fn get_piece_bb_mut(&mut self, piece: PieceKind) -> &mut BitBoard {
        &mut self.piece_bb[piece as u8 as usize]
    }

    /// Get the bitboard associated with a certain color.
    #[inline]
    pub const fn get_color_bb(&self, color: Color) -> BitBoard {
        self.color_bb[color as u8 as usize]
    }

    /// Get an exclusive reference to the bitboard associated with a certain color.
    #[inline]
    pub fn get_color_bb_mut(&mut self, color: Color) -> &mut BitBoard {
        &mut self.color_bb[color as u8 as usize]
    }

    /// Set a piece on the board.
    pub fn set(&mut self, square: Square, piece: Option<Piece>) {
        match piece {
            Some(piece) => {
                self.get_color_bb_mut(piece.color).set(square, true);
                self.get_piece_bb_mut(piece.kind).set(square, true);
            }
            None => {
                // should be vectorized hopefully
                for bb in &mut self.color_bb {
                    bb.set(square, false);
                }
                for bb in &mut self.piece_bb {
                    bb.set(square, false);
                }
            }
        }
    }
}

impl Display for Board {
    // not pretty but works
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " a  b  c  d  e  f  g  h")?;
        let mut square = Square::at(7, 0).unwrap();

        for _ in 0..8 {
            for _ in 0..8 {
                if let Some(piece) = self.get(square) {
                    write!(f, " {} ", piece.as_char())?;
                } else {
                    write!(f, "   ")?;
                }

                square.0 += 1;
            }

            writeln!(f)?;
            square.0 = square.0.saturating_sub(16);
        }

        Ok(())
    }
}

#[cfg(test)]
mod square_tests {
    use super::*;

    #[test]
    fn square_to_str() {
        assert_eq!(&Square::at(0, 0).unwrap().to_string(), "a1");
        assert_eq!(&Square::at(5, 7).unwrap().to_string(), "f8");
        assert_eq!(&Square::at(4, 3).unwrap().to_string(), "e4");
        assert_eq!(&Square::at(2, 6).unwrap().to_string(), "c7");
        assert_eq!(&Square::at(8, 8), &None);
    }

    #[test]
    fn square_from_str() {
        assert_eq!(Square::from_str("a2"), Square::at(0, 1).ok_or(()));
        assert_eq!(Square::from_str("h8"), Square::at(7, 7).ok_or(()));
        assert_eq!(Square::from_str("C5"), Square::at(2, 4).ok_or(()));
        assert!(Square::from_str("5c").is_err());
        assert!(Square::from_str("b-").is_err());
        assert!(Square::from_str("^8").is_err());
        assert!(Square::from_str("b891").is_err());
        assert!(Square::from_str("b0").is_err());
    }
}
