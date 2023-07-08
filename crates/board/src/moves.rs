//! Piece move information.

use std::fmt::Display;

use crate::{PieceKind, Square};

/// Minimal information required to represent a move in [LAN].
///
/// [LAN]: https://www.chessprogramming.org/Algebraic_Chess_Notation#Long_Algebraic_Notation_.28LAN.29
#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.from)?;
        write!(f, "{}", self.to)?;

        if let Some(promotion) = self.promotion {
            write!(f, "{}", promotion.as_char().to_ascii_lowercase())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Capture {
    Regular(PieceKind),
    EnPassant(Square),
}

/// Some additional info about a move to help with move ordering, application, etc.
#[derive(Debug, Clone, Copy)]
pub struct MoveExt {
    pub piece_kind: PieceKind,
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
    pub capture: Option<Capture>,
}

impl MoveExt {
    #[inline]
    pub const fn from_move(p_move: Move, piece_kind: PieceKind) -> Self {
        Self {
            piece_kind,
            from: p_move.from,
            to: p_move.to,
            promotion: None,
            capture: None,
        }
    }

    #[inline]
    pub const fn to_move(&self) -> Move {
        Move {
            from: self.from,
            to: self.to,
            promotion: self.promotion,
        }
    }
}

impl Display for MoveExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.piece_kind != PieceKind::Pawn {
            write!(f, "{}", self.piece_kind.as_char())?;
        }

        write!(f, "{}", self.from)?;

        if self.capture.is_some() {
            write!(f, "x")?;
        }

        write!(f, "{}", self.to)?;

        if let Some(promotion) = self.promotion {
            write!(f, "{}", promotion.as_char().to_ascii_lowercase())?;
        }

        Ok(())
    }
}
