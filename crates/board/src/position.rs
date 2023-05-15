//! The full game position.

use crate::{Board, Color, Square};

bitflags::bitflags! {
    /// Player castling availability.
    pub struct CastlingRights: u8 {
        /// White kingside.
        const WHITE_OO = 0x01;
        /// White queenside.
        const WHITE_OOO = 0x02;
        /// Black kingside.
        const BLACK_OO = 0x04;
        /// Black queenside.
        const BLACK_OOO = 0x08;
    }
}

/// Full chessboard state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    /// Piece positions.
    pub board: Board,
    /// Whose turn it is to move.
    pub active_color: Color,
    /// Castling rights flags.
    pub castling: CastlingRights,
    /// En passant target square.
    pub ep_target: Option<Square>,
    /// Half-move (ply) clock.
    ///
    /// A half-move is a single move made by a single player. This counts the number of half-moves
    /// since the last capture, pawn move, or check; and is used for the 50-move rule.
    pub halfmove_clock: u8,
    /// Full-move counter.
    ///
    /// A full-move consists of two half-moves, one by white and one by black. This counts the total
    /// number of moves since the game began. It starts at 1 and increments after black's move.
    pub fullmove_counter: u8,
}

impl Position {
    pub fn starting() -> Self {
        Position {
            board: Board::starting_position(),
            active_color: Color::White,
            castling: CastlingRights::all(),
            ep_target: None,
            halfmove_clock: 0,
            fullmove_counter: 1,
        }
    }
}
