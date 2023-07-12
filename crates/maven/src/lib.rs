//! Move generator implementation.
//!
//! Why is it called Maven? I dunno. It sounds better than "movegen" tho.

#![allow(clippy::comparison_chain)]
#![allow(clippy::field_reassign_with_default)]

use std::cmp::min;

use sealion_board::{BitBoard, CastlingRights, Color, MoveExt, PieceKind, Position, Square};
use smallvec::SmallVec;

mod o_moves;
mod tables;

use o_moves::OpponentMoves;

#[inline]
fn merge_bb(boards: [BitBoard; 4]) -> BitBoard {
    boards[0] | boards[1] | boards[2] | boards[3]
}

/// The primary structure which contains relevant piece state information, such as attacks and checks.
#[derive(Debug, Clone)]
pub enum MoveList {
    Moves(Vec<MoveExt>),
    Checkmate,
    Stalemate,
}

impl MoveList {
    pub fn generate(position: &Position) -> Self {
        let opponent_moves = OpponentMoves::generate(position);

        let move_list = Self::generate_impl(position, &opponent_moves);

        if move_list.is_empty() {
            if opponent_moves.attacks & opponent_moves.friendly_king != 0 {
                return Self::Checkmate;
            }
            return Self::Stalemate;
        }

        Self::Moves(move_list)
    }

    fn generate_impl(position: &Position, o_moves: &OpponentMoves) -> Vec<MoveExt> {
        let mut moves = Vec::with_capacity(256);

        // initial king move generation
        let king_sq = o_moves.friendly_king.to_square_unchecked();
        let king_moves = Self::pseudo_king_moves(king_sq, position) & !o_moves.attacks;

        for to_square in king_moves.set_iter() {
            let p_move = MoveExt {
                from: king_sq,
                to: to_square,
                piece_kind: PieceKind::King,
                promotion: None,
                capture: o_moves.resolve_capture(to_square),
            };

            moves.push(p_move);
        }

        // Double check
        // - Forced king move
        if o_moves.checkers.melee.len() + o_moves.checkers.sliders.len() > 1 {
            return moves;
        }

        let mut restricted = BitBoard(u64::MAX);

        // Melee check
        // - Checker can be captured
        // ~ King move to non-attacked square
        if let Some(checker_sq) = o_moves.checkers.melee.get(0) {
            restricted = BitBoard::from_square(*checker_sq);
        }

        // Sliding check
        // - Checker can be captured
        // - Checker can be blocked along attack-ray
        // ~ King move to non-attacked square
        if let Some(checker_ray) = o_moves.checkers.sliders.get(0) {
            restricted = *checker_ray;
        }

        // Generate other piece moves
        let friendly = position.board.get_color_bb(position.active_color);

        for square in friendly.set_iter() {
            let square_bb = BitBoard::from_square(square);

            // Handle pins
            let mut restricted = restricted;

            for pinned in &o_moves.pinners {
                if square_bb & *pinned != 0 {
                    restricted &= *pinned;
                    break;
                }
            }

            // Generate moves

            let mut push_moves = |p_moves: BitBoard, piece_kind| {
                let legal_moves = p_moves & restricted;

                for to_square in legal_moves.set_iter() {
                    let p_move = MoveExt {
                        from: square,
                        to: to_square,
                        piece_kind,
                        promotion: None,
                        capture: o_moves.resolve_capture(to_square),
                    };

                    moves.push(p_move);
                }
            };

            // Bishop
            if square_bb & position.board.get_piece_kind_bb(PieceKind::Bishop) != 0 {
                let p_moves = MoveList::pseudo_bishop_moves(square, position);
                (push_moves)(p_moves, PieceKind::Bishop);
            // Rook
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Rook) != 0 {
                let p_moves = MoveList::pseudo_rook_moves(square, position);
                (push_moves)(p_moves, PieceKind::Rook);
            // Queen
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Queen) != 0 {
                let p_moves = MoveList::pseudo_bishop_moves(square, position)
                    | MoveList::pseudo_rook_moves(square, position);
                (push_moves)(p_moves, PieceKind::Queen);
            // Knight
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Knight) != 0 {
                let p_moves = MoveList::pseudo_knight_moves(square, position);
                (push_moves)(p_moves, PieceKind::Knight);
            // Pawn
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Pawn) != 0 {
                let p_moves = MoveList::pseudo_pawn_moves(square, position);

                let legal_moves = p_moves & restricted;

                // handle pawn moves separately
                let promotable = match position.active_color {
                    Color::White => square.rank() == 6,
                    Color::Black => square.rank() == 1,
                };

                if promotable {
                    for to_square in legal_moves.set_iter() {
                        let p_move = MoveExt {
                            from: square,
                            to: to_square,
                            piece_kind: PieceKind::Pawn,
                            promotion: None,
                            capture: o_moves.resolve_capture(to_square),
                        };

                        for promote_to in [
                            PieceKind::Knight,
                            PieceKind::Bishop,
                            PieceKind::Rook,
                            PieceKind::Queen,
                        ] {
                            moves.push(MoveExt {
                                promotion: Some(promote_to),
                                ..p_move
                            });
                        }
                    }
                } else {
                    for to_square in legal_moves.set_iter() {
                        let p_move = MoveExt {
                            from: square,
                            to: to_square,
                            piece_kind: PieceKind::Pawn,
                            promotion: None,
                            capture: o_moves
                                .resolve_capture(to_square)
                                .or_else(|| o_moves.resolve_ep(to_square, position.ep_target)),
                        };

                        moves.push(p_move);
                    }
                }
            }
        }

        // Castling moves
        let castling = Self::castling_moves(king_sq, position, o_moves.attacks);
        moves.extend(castling);

        moves
    }
}

impl MoveList {
    #[inline]
    pub fn pseudo_bishop_moves(square: Square, position: &Position) -> BitBoard {
        Self::sliding_moves::<0>(square, position)
    }

    #[inline]
    pub fn pseudo_rook_moves(square: Square, position: &Position) -> BitBoard {
        Self::sliding_moves::<1>(square, position)
    }

    #[inline]
    fn sliding_moves<const DIR: u8>(square: Square, position: &Position) -> BitBoard {
        let friendly = position.board.get_color_bb(position.active_color);
        let blockers = position.board.get_full_bb();

        let attacks = Self::sliding_attacks::<DIR>(square, blockers);

        merge_bb(attacks) & !friendly
    }

    fn sliding_attacks<const DIR: u8>(square: Square, blockers: BitBoard) -> [BitBoard; 4] {
        let mut moves = [BitBoard(0); 4];

        let (max, shifts) = match DIR {
            0 => {
                // DIAGONAL
                let max = [
                    min(7 - square.rank(), 7 - square.file()), // NE
                    min(7 - square.rank(), square.file()),     // NW
                    min(square.rank(), 7 - square.file()),     // SE
                    min(square.rank(), square.file()),         // SW
                ];

                (max, [[9, 7, 0, 0], [0, 0, 7, 9]])
            }
            1 => {
                // ORTHOGONAL
                let max = [
                    7 - square.rank(), // N
                    square.rank(),     // S
                    7 - square.file(), // E
                    square.file(),     // W
                ];

                (max, [[8, 0, 1, 0], [0, 8, 0, 1]])
            }
            _ => panic!("disallowed value for sliding attack direction (should be 1 or 0)"),
        };

        let square = BitBoard::from_square(square);

        for direction in 0..4 {
            let mut square = square;

            for _ in 0..max[direction] {
                square <<= shifts[0][direction];
                square >>= shifts[1][direction];

                moves[direction] |= square;

                if square & blockers != 0 {
                    break;
                }
            }
        }

        moves
    }

    fn knight_attacks(square: Square) -> BitBoard {
        tables::KNIGHT_ATTACKS[square.raw_index() as usize]
    }

    #[inline]
    pub fn pseudo_knight_moves(square: Square, position: &Position) -> BitBoard {
        Self::knight_attacks(square) & !position.board.get_color_bb(position.active_color)
    }

    fn pawn_attacks(square: Square, color: Color) -> BitBoard {
        tables::PAWN_ATTACKS[(square.raw_index() + (color as u8 * 64)) as usize]
    }

    pub fn pseudo_pawn_moves(square: Square, position: &Position) -> BitBoard {
        let mut unfriendly = position
            .board
            .get_color_bb(position.active_color.opposite());
        let blockers = position.board.get_full_bb();

        // fake a piece for ep
        if let Some(ep_target) = position.ep_target {
            unfriendly |= BitBoard::from_square(ep_target);
        }

        let start = BitBoard::from_square(square);
        let mut moves = Self::pawn_attacks(square, position.active_color) & unfriendly;

        match position.active_color {
            Color::White => {
                let next = start << 8;

                if blockers & next == 0 {
                    // single push
                    if square.rank() < 7 {
                        moves |= next;

                        // double push
                        let next_2 = next << 8;

                        if square.rank() == 1 && blockers & next_2 == 0 {
                            moves |= next_2;
                        }
                    }
                }
            }
            Color::Black => {
                // single push
                let next = start >> 8;

                if blockers & next == 0 {
                    // single push
                    if square.rank() > 1 {
                        moves |= next;

                        // double push
                        let next_2 = next >> 8;

                        if square.rank() == 6 && blockers & next_2 == 0 {
                            moves |= next_2;
                        }
                    }
                }
            }
        }

        moves
    }

    fn king_attacks(square: Square) -> BitBoard {
        tables::KING_ATTACKS[square.raw_index() as usize]
    }

    #[inline]
    pub fn pseudo_king_moves(square: Square, position: &Position) -> BitBoard {
        Self::king_attacks(square) & !position.board.get_color_bb(position.active_color)
    }

    const CASTLING_CHECKS: [CastlingChecks; 4] = {
        // white
        let start = 0b1110;

        let mut checks_woo = CastlingChecks::zero();
        checks_woo.clear = BitBoard(start << 4 & !(1 << 7));
        checks_woo.safe = BitBoard(start << 3);
        checks_woo.to_sq = BitBoard(1 << 6).to_square_unchecked();

        let mut checks_wooo = CastlingChecks::zero();
        checks_wooo.clear = BitBoard(start);
        checks_wooo.safe = BitBoard(start << 1);
        checks_wooo.to_sq = BitBoard(1 << 2).to_square_unchecked();

        // black
        let start = 0b111 << 57;

        let mut checks_boo = CastlingChecks::zero();
        checks_boo.clear = BitBoard(start << 4 & !(1 << 63));
        checks_boo.safe = BitBoard(start << 3);
        checks_boo.to_sq = BitBoard(1 << 58).to_square_unchecked();

        let mut checks_booo = CastlingChecks::zero();
        checks_booo.clear = BitBoard(start);
        checks_booo.safe = BitBoard(start << 1);
        checks_booo.to_sq = BitBoard(1 << 62).to_square_unchecked();

        [checks_woo, checks_wooo, checks_boo, checks_booo]
    };

    fn castling_moves(
        king_sq: Square,
        position: &Position,
        attacks: BitBoard,
    ) -> SmallVec<[MoveExt; 2]> {
        let mut moves = SmallVec::new();

        let blockers = position.board.get_full_bb();

        let mut do_checks = |checks: CastlingChecks| {
            if checks.clear & blockers == 0 && checks.safe & attacks == 0 {
                moves.push(MoveExt {
                    piece_kind: PieceKind::King,
                    from: king_sq,
                    to: checks.to_sq,
                    promotion: None,
                    capture: None,
                });
            }
        };

        match position.active_color {
            Color::White => {
                if position.castling.contains(CastlingRights::WHITE_OO) {
                    (do_checks)(Self::CASTLING_CHECKS[0]);
                }
                if position.castling.contains(CastlingRights::WHITE_OOO) {
                    (do_checks)(Self::CASTLING_CHECKS[1]);
                }
            }
            Color::Black => {
                if position.castling.contains(CastlingRights::BLACK_OO) {
                    (do_checks)(Self::CASTLING_CHECKS[2]);
                }
                if position.castling.contains(CastlingRights::BLACK_OOO) {
                    (do_checks)(Self::CASTLING_CHECKS[3]);
                }
            }
        }

        moves
    }
}

/// Secondary checks for a valid castling move.
#[derive(Debug, Clone, Copy)]
struct CastlingChecks {
    /// Squares in between the king and rook are not occupied.
    clear: BitBoard,
    /// Castling squares are not under attack.
    safe: BitBoard,
    /// Final square.
    to_sq: Square,
}

impl CastlingChecks {
    #[inline]
    const fn zero() -> Self {
        Self {
            clear: BitBoard(0),
            safe: BitBoard(0),
            to_sq: Square::from_index_unchecked(0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct MoveTester {
        pub name: &'static str,
        pub sq: (u8, u8),
        pub fen: &'static str,
        pub result: u64,
    }

    impl MoveTester {
        pub fn do_gen(&self, f: impl Fn(Square, &Position) -> BitBoard) {
            let position = sealion_fen::de::parse(self.fen)
                .expect(&format!("`{}` failed due to bad fen", self.name))
                .1;
            let square = Square::try_from(self.sq)
                .expect(&format!("`{}` failed due to bad square", self.name));

            let result = f(square, &position);

            assert_eq!(result, self.result, "`{}` failed", self.name);
        }
    }

    #[test]
    fn bishop_moves() {
        let tests = [MoveTester {
            name: "start with bishop",
            sq: (4, 5),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            result: 0x88_50_00_50_88_00_00,
        }];

        for test in tests {
            test.do_gen(MoveList::pseudo_bishop_moves);
        }
    }

    #[test]
    fn rook_moves() {
        let tests = [MoveTester {
            name: "start with bishop",
            sq: (4, 5),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            result: 0x20_20_df_20_20_00_00,
        }];

        for test in tests {
            test.do_gen(MoveList::pseudo_rook_moves);
        }
    }

    #[test]
    fn knight_moves() {
        let tests = [MoveTester {
            name: "knight middle",
            sq: (3, 3),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            result: 0x14_22_00_22_00_00,
        }];

        for test in tests {
            test.do_gen(MoveList::pseudo_knight_moves);
        }
    }

    #[test]
    fn pawn_moves() {
        let tests = [
            MoveTester {
                name: "ep white",
                sq: (4, 3),
                fen: "rnbqkbnr/pppp1ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1",
                result: 0x18_00_00_00_00_00,
            },
            MoveTester {
                name: "ep black",
                sq: (3, 4),
                fen: "rnbqkbnr/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1",
                result: 0x18_00_00,
            },
        ];

        for test in tests {
            test.do_gen(MoveList::pseudo_pawn_moves);
        }
    }

    #[test]
    fn king_moves() {
        let tests = [
            MoveTester {
                name: "king middle",
                sq: (3, 3),
                fen: "rnbqkbnr/pppppppp/8/8/3K4/8/PPPPPPPP/RNBQ1BNR w kq - 0 1",
                result: 0x1c_14_1c_00_00,
            },
            MoveTester {
                name: "king side",
                sq: (4, 0),
                fen: "rnbqkbnr/pppp1ppp/8/K3p3/1P6/8/PP1PPPPP/RNBQ1BNR w kq - 0 1",
                result: 0x03_82_01_80_00_00,
            },
        ];

        for test in tests {
            test.do_gen(MoveList::pseudo_king_moves);
        }
    }

    #[test]
    fn full_move_gen() {
        let position = Position::starting();
        let moves = MoveList::generate(&position);

        match moves {
            MoveList::Moves(moves) => {
                assert_eq!(moves.len(), 20)
            }
            _ => panic!("starting position is not mate"),
        }
    }
}
