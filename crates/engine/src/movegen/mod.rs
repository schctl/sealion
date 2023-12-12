//! Move generator implementation.

use std::cmp::min;
use std::ops::BitOr;

use sealion_board::{BitBoard, CastlingRights, Color, MoveExt, PieceKind, Square};
use smallvec::SmallVec;

use crate::state::PositionState;
use PieceKind::*;

mod tables;

#[inline]
pub fn merge_bb<const U: usize>(boards: [BitBoard; U]) -> BitBoard {
    boards.into_iter().fold(BitBoard::ZERO, BitOr::bitor)
}

/// The primary structure which contains relevant piece state information, such as attacks and checks.
#[derive(Debug, Clone)]
pub enum MoveList {
    Moves(Vec<MoveExt>),
    Checkmate,
    Stalemate,
}

impl MoveList {
    /// Generate the full [`MoveList`] without going through an intermediate generator.
    #[inline]
    pub fn generate(position: &PositionState) -> Self {
        let generator = Generator::new(position);
        generator.generate()
    }
}

/// Move generator re-usable data.
#[derive(Debug, Clone)]
pub struct Generator<'a> {
    /// The position we are generating moves for.
    state: &'a PositionState<'a>,
}

impl<'a> Generator<'a> {
    #[inline]
    pub fn new(state: &'a PositionState) -> Self {
        Self { state }
    }

    pub fn generate(&self) -> MoveList {
        let move_list = self.generate_impl();

        if move_list.is_empty() {
            if self.state.attacks.bb & self.state.board_ext.king_bb != 0 {
                return MoveList::Checkmate;
            }
            return MoveList::Stalemate;
        }

        MoveList::Moves(move_list)
    }

    fn generate_impl(&self) -> Vec<MoveExt> {
        let mut moves = Vec::with_capacity(256);

        // initial king move generation
        let king_sq = self.state.board_ext.king_bb.to_square_unchecked();
        let king_moves = self.pseudo_king_moves(king_sq) & !self.state.attacks.bb;

        for to_square in king_moves.set_iter() {
            let p_move = MoveExt {
                from: king_sq,
                to: to_square,
                piece_kind: King,
                promotion: None,
                capture: self.state.resolve_capture_only(to_square),
            };

            moves.push(p_move);
        }

        // Double check
        // - Forced king move
        if self.state.attacks.checkers.melee.len() + self.state.attacks.checkers.sliders.len() > 1 {
            return moves;
        }

        let mut restricted = BitBoard(u64::MAX);

        // Melee check
        // - Checker can be captured
        // ~ King move to non-attacked square
        if let Some(checker_sq) = self.state.attacks.checkers.melee.get(0) {
            restricted = BitBoard::from_square(*checker_sq);
        }

        // Sliding check
        // - Checker can be captured
        // - Checker can be blocked along attack-ray
        // ~ King move to non-attacked square
        if let Some(checker_ray) = self.state.attacks.checkers.sliders.get(0) {
            restricted = *checker_ray;
        }

        // Generate other piece moves
        let friendly = self
            .state
            .position
            .board
            .get_color_bb(self.state.position.active_color);

        for square in friendly.set_iter() {
            let square_bb = BitBoard::from_square(square);

            // Handle pins
            let mut restricted = restricted;

            for pinned in &self.state.attacks.pinners {
                if square_bb & *pinned != 0 {
                    restricted &= *pinned;
                    break;
                }
            }

            // Generate moves
            let p_kind = self.state.board_ext.get(square).unwrap().kind;
            let p_moves = self.pseudo_moves(square, p_kind);

            if p_kind == Pawn {
                // insert pawn moves separately
                let legal_moves = p_moves & restricted;

                // handle inserting pawn moves
                let promotable = match self.state.position.active_color {
                    Color::White => square.rank() == 6,
                    Color::Black => square.rank() == 1,
                };

                if promotable {
                    for to_square in legal_moves.set_iter() {
                        let p_move = MoveExt {
                            from: square,
                            to: to_square,
                            piece_kind: Pawn,
                            promotion: None,
                            capture: self.state.resolve_capture_only(to_square),
                        };

                        for promote_to in PieceKind::PROMOTABLE {
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
                            piece_kind: Pawn,
                            promotion: None,
                            capture: self.state.resolve_capture(to_square),
                        };

                        moves.push(p_move);
                    }
                }
            } else if p_kind != King {
                // insert other piece moves
                let legal_moves = p_moves & restricted;

                for to_square in legal_moves.set_iter() {
                    let p_move = MoveExt {
                        from: square,
                        to: to_square,
                        piece_kind: p_kind,
                        promotion: None,
                        capture: self.state.resolve_capture_only(to_square),
                    };

                    moves.push(p_move);
                }
            }
        }

        // Castling moves
        let castling = self.castling_moves();
        moves.extend(castling);

        moves
    }
}

impl<'a> Generator<'a> {
    #[rustfmt::skip]
    pub fn pseudo_moves(&self, square: Square, kind: PieceKind) -> BitBoard {
        match kind {
            Pawn   => self.pseudo_pawn_moves(square),
            Knight => self.pseudo_knight_moves(square),
            Bishop => self.pseudo_bishop_moves(square),
            Rook   => self.pseudo_rook_moves(square),
            Queen  => self.pseudo_bishop_moves(square) | self.pseudo_rook_moves(square),
            King   => self.pseudo_king_moves(square),
        }
    }

    #[inline]
    pub fn pseudo_bishop_moves(&self, square: Square) -> BitBoard {
        self.sliding_moves::<0>(square)
    }

    #[inline]
    pub fn pseudo_rook_moves(&self, square: Square) -> BitBoard {
        self.sliding_moves::<1>(square)
    }

    #[inline]
    fn sliding_moves<const DIR: u8>(&self, square: Square) -> BitBoard {
        let friendly = self
            .state
            .position
            .board
            .get_color_bb(self.state.position.active_color);
        let blockers = self.state.position.board.get_full_bb();

        let attacks = Self::sliding_attacks::<DIR>(square, blockers);

        merge_bb(attacks) & !friendly
    }

    pub fn sliding_attacks<const DIR: u8>(square: Square, blockers: BitBoard) -> [BitBoard; 4] {
        let mut moves = [BitBoard::ZERO; 4];

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

    pub fn knight_attacks(square: Square) -> BitBoard {
        tables::KNIGHT_ATTACKS[square.raw_index() as usize]
    }

    #[inline]
    pub fn pseudo_knight_moves(&self, square: Square) -> BitBoard {
        Self::knight_attacks(square)
            & !self
                .state
                .position
                .board
                .get_color_bb(self.state.position.active_color)
    }

    pub fn pawn_attacks(square: Square, color: Color) -> BitBoard {
        tables::PAWN_ATTACKS[(square.raw_index() + (color as u8 * 64)) as usize]
    }

    pub fn pseudo_pawn_moves(&self, square: Square) -> BitBoard {
        let mut unfriendly = self
            .state
            .position
            .board
            .get_color_bb(self.state.position.active_color.opposite());
        let blockers = self.state.position.board.get_full_bb();

        // fake a piece for ep
        if let Some(ep_target) = self.state.position.ep_target {
            unfriendly |= BitBoard::from_square(ep_target);
        }

        let start = BitBoard::from_square(square);
        let mut moves = Self::pawn_attacks(square, self.state.position.active_color) & unfriendly;

        match self.state.position.active_color {
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

    pub fn king_attacks(square: Square) -> BitBoard {
        tables::KING_ATTACKS[square.raw_index() as usize]
    }

    #[inline]
    pub fn pseudo_king_moves(&self, square: Square) -> BitBoard {
        Self::king_attacks(square)
            & !self
                .state
                .position
                .board
                .get_color_bb(self.state.position.active_color)
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

    fn castling_moves(&self) -> SmallVec<[MoveExt; 2]> {
        let mut moves = SmallVec::new();

        let blockers = self.state.position.board.get_full_bb();

        let mut do_checks = |checks: CastlingChecks| {
            if checks.clear & blockers == 0 && checks.safe & self.state.attacks.bb == 0 {
                moves.push(MoveExt {
                    piece_kind: King,
                    from: self.state.board_ext.king_bb.to_square_unchecked(),
                    to: checks.to_sq,
                    promotion: None,
                    capture: None,
                });
            }
        };

        match self.state.position.active_color {
            Color::White => {
                if self
                    .state
                    .position
                    .castling
                    .contains(CastlingRights::WHITE_OO)
                {
                    (do_checks)(Self::CASTLING_CHECKS[0]);
                }
                if self
                    .state
                    .position
                    .castling
                    .contains(CastlingRights::WHITE_OOO)
                {
                    (do_checks)(Self::CASTLING_CHECKS[1]);
                }
            }
            Color::Black => {
                if self
                    .state
                    .position
                    .castling
                    .contains(CastlingRights::BLACK_OO)
                {
                    (do_checks)(Self::CASTLING_CHECKS[2]);
                }
                if self
                    .state
                    .position
                    .castling
                    .contains(CastlingRights::BLACK_OOO)
                {
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
            clear: BitBoard::ZERO,
            safe: BitBoard::ZERO,
            to_sq: Square::from_index_unchecked(0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sealion_board::Position;

    struct MoveTester {
        pub name: &'static str,
        pub sq: (u8, u8),
        pub fen: &'static str,
        pub result: u64,
    }

    impl MoveTester {
        pub fn do_gen<F>(&self, f: F)
        where
            F: Fn(Generator<'_>, Square) -> BitBoard,
        {
            let position = sealion_fen::from_str(self.fen)
                .expect(&format!("`{}` failed due to bad fen", self.name));
            let state = PositionState::generate(&position);
            let square = Square::try_from(self.sq)
                .expect(&format!("`{}` failed due to bad square", self.name));
            let generator = Generator::new(&state);

            let result = f(generator, square);
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
            test.do_gen(|gen, sq| gen.pseudo_bishop_moves(sq));
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
            test.do_gen(|gen, sq| gen.pseudo_rook_moves(sq));
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
            test.do_gen(|gen, sq| gen.pseudo_knight_moves(sq));
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
            test.do_gen(|gen, sq| gen.pseudo_pawn_moves(sq));
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
            test.do_gen(|gen, sq| gen.pseudo_king_moves(sq));
        }
    }

    #[test]
    fn full_move_gen() {
        let position = Position::starting();
        let state = PositionState::generate(&position);
        let moves = MoveList::generate(&state);

        match moves {
            MoveList::Moves(moves) => {
                assert_eq!(moves.len(), 20)
            }
            _ => panic!("starting position is not mate"),
        }
    }
}
