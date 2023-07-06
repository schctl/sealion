use std::fmt::Display;

use crate::{PieceKind, Square};

/// Minimal information required to represent a move in [LAN].
///
/// [LAN]: https://www.chessprogramming.org/Algebraic_Chess_Notation#Long_Algebraic_Notation_.28LAN.29
#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub piece_kind: PieceKind,
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.piece_kind != PieceKind::Pawn {
            write!(f, "{}", self.piece_kind.as_char())?;
        }

        write!(f, "{}", self.from)?;
        write!(f, "{}", self.to)?;

        if let Some(promotion) = self.promotion {
            write!(f, "{}", promotion.as_char().to_ascii_lowercase())?;
        }

        Ok(())
    }
}
