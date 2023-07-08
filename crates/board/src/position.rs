//! The full game position.

use crate::{BitBoard, Board, Capture, Color, MoveExt, PieceKind, Square};

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

impl CastlingRights {
    #[inline]
    pub fn unset_oo(self, color: Color) -> Self {
        match color {
            Color::White => self & !Self::WHITE_OO,
            Color::Black => self & !Self::BLACK_OO,
        }
    }

    #[inline]
    pub fn unset_ooo(self, color: Color) -> Self {
        match color {
            Color::White => self & !Self::WHITE_OO,
            Color::Black => self & !Self::BLACK_OO,
        }
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
    /// A ply is a single move made by a single player. This counts the number of plies
    /// since the last capture or pawn move and is used for the 50-move rule.
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

    /// Apply a move without preliminary checks (piece existence for egs).
    pub fn apply_move_unchecked(&mut self, p_move: MoveExt) {
        let from_sq = BitBoard::from_square(p_move.from);
        let to_sq = BitBoard::from_square(p_move.to);

        // apply move
        let color_bb = self.board.get_color_bb_mut(self.active_color);
        *color_bb &= !from_sq;
        *color_bb |= to_sq;

        let piece_bb = self.board.get_piece_kind_bb_mut(p_move.piece_kind);
        *piece_bb &= !from_sq;
        *piece_bb |= to_sq;

        // handle castling
        if p_move.piece_kind == PieceKind::King
            && p_move.from.raw_index().abs_diff(p_move.to.raw_index()) == 2
        {
            // queen side
            let (rook_from_sq, rook_to_sq) = if p_move.to.raw_index() < p_move.from.raw_index() {
                self.castling = self.castling.unset_ooo(self.active_color);
                let rfs = from_sq >> 4;
                let rts = to_sq >> 1;
                (rfs, rts)
            }
            // king side
            else {
                self.castling = self.castling.unset_oo(self.active_color);
                let rfs = from_sq << 3;
                let rts = to_sq << 1;
                (rfs, rts)
            };

            let rook_bb = self.board.get_piece_kind_bb_mut(PieceKind::Rook);
            *rook_bb &= !rook_from_sq;
            *rook_bb |= rook_to_sq;

            let color_bb = self.board.get_color_bb_mut(self.active_color);
            *color_bb &= !rook_from_sq;
            *color_bb |= rook_to_sq;

            return;
        }

        // handle pawn promotion
        if p_move.piece_kind == PieceKind::Pawn {
            if let Some(promotion) = p_move.promotion {
                // remove previously moved pawn
                let pawn_bb = self.board.get_piece_kind_bb_mut(PieceKind::Pawn);
                *pawn_bb &= !to_sq;

                let promo_bb = self.board.get_piece_kind_bb_mut(promotion);
                *promo_bb |= to_sq;

                return;
            }
        }

        // check for capture
        match p_move.capture {
            Some(Capture::Regular(cap)) => {
                *self.board.get_color_bb_mut(self.active_color.opposite()) &= !to_sq;
                *self.board.get_piece_kind_bb_mut(cap) &= !to_sq;
            }
            Some(Capture::EnPassant(target)) => {
                let target_sq = BitBoard::from_square(target);
                *self.board.get_color_bb_mut(self.active_color.opposite()) &= !target_sq;
                *self.board.get_piece_kind_bb_mut(PieceKind::Pawn) &= !target_sq;
            }
            _ => {}
        }

        if p_move.capture.is_none() || p_move.piece_kind != PieceKind::Pawn {
            self.halfmove_clock += 1;
        }
        if self.active_color == Color::Black {
            self.fullmove_counter += 1;
        }
        self.active_color = self.active_color.opposite();
    }
}
