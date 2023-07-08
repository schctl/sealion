//! Move generator implementation.
//!
//! Why is it called Maven? I dunno. It sounds better than "movegen" tho.

#![allow(clippy::comparison_chain)]
#![allow(clippy::field_reassign_with_default)]

use std::cmp::min;

use sealion_board::{
    BitBoard, Capture, CastlingRights, Color, MoveExt, Piece, PieceKind, Position, Square,
};
use smallvec::SmallVec;

#[inline]
fn merge_bb(boards: [BitBoard; 4]) -> BitBoard {
    boards[0] | boards[1] | boards[2] | boards[3]
}

/// The primary structure which contains relevant piece state information, such as attacks and checks.
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
        // capture handler
        let resolve_capture = |square: Square, mover: PieceKind| -> Option<Capture> {
            if let Some(piece) = o_moves.pieces[square.raw_index() as usize] {
                return Some(Capture::Regular(piece));
            }

            if let Some(ep_target) = position.ep_target {
                if mover == PieceKind::Pawn
                    && square.raw_index().abs_diff(ep_target.raw_index()) == 8
                {
                    return Some(Capture::EnPassant(ep_target));
                }
            }

            None
        };

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
                capture: (resolve_capture)(to_square, PieceKind::King),
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

            let mut p_moves = BitBoard(0);
            let mut piece_kind = PieceKind::Pawn;

            // Bishop
            if square_bb & position.board.get_piece_kind_bb(PieceKind::Bishop) != 0 {
                p_moves = MoveList::pseudo_bishop_moves(square, position);
                piece_kind = PieceKind::Bishop;
                // Rook
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Rook) != 0 {
                p_moves = MoveList::pseudo_rook_moves(square, position);
                piece_kind = PieceKind::Rook;
                // Queen
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Queen) != 0 {
                // bishop moves first
                p_moves = MoveList::pseudo_bishop_moves(square, position)
                    | MoveList::pseudo_rook_moves(square, position);
                piece_kind = PieceKind::Queen;
                // Knight
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Knight) != 0 {
                p_moves = MoveList::pseudo_knight_moves(square, position);
                piece_kind = PieceKind::Knight;
                // Pawn
            } else if square_bb & position.board.get_piece_kind_bb(PieceKind::Pawn) != 0 {
                let pawn_moves = MoveList::pseudo_pawn_moves(square, position);
                p_moves = pawn_moves.pushes | pawn_moves.attacks;

                // handle promotions
                let promotable = pawn_moves.promotions & restricted;

                for to_square in promotable.set_iter() {
                    for promote_to in [
                        PieceKind::Knight,
                        PieceKind::Bishop,
                        PieceKind::Rook,
                        PieceKind::Queen,
                    ] {
                        let p_move = MoveExt {
                            from: square,
                            to: to_square,
                            piece_kind,
                            promotion: Some(promote_to),
                            capture: (resolve_capture)(to_square, piece_kind),
                        };

                        moves.push(p_move);
                    }
                }
            }

            // Resolve legal move bitboards into individual moves
            let legal_moves = p_moves & restricted;

            for to_square in legal_moves.set_iter() {
                let p_move = MoveExt {
                    from: square,
                    to: to_square,
                    piece_kind,
                    promotion: None,
                    capture: (resolve_capture)(to_square, piece_kind),
                };

                moves.push(p_move);
            }
        }

        // Castling moves
        let castling = Self::castling_moves(king_sq, position, o_moves.attacks);
        moves.extend(castling);

        moves
    }
}

#[derive(Debug, Clone, Default)]
struct Checkers {
    /// Nearby attackers.
    ///
    /// Have to be captured or evaded by king.
    melee: SmallVec<[Square; 2]>,
    /// Faraway attacker ray.
    ///
    /// Have to be captured, evaded or blocked.
    sliders: SmallVec<[BitBoard; 2]>,
}

/// Pseudo moves for the opponent. Used to calculate checks and pins.
#[derive(Debug, Clone)]
pub struct OpponentMoves {
    /// Pre-calculated piece kinds for each square.
    pieces: [Option<PieceKind>; 64],
    /// All attacked squares.
    attacks: BitBoard,
    /// Attackers on our king.
    checkers: Checkers,
    /// Sliders pinning pieces to the king.
    pinners: SmallVec<[(Square, BitBoard); 4]>,
    /// Friendly king square.
    friendly_king: BitBoard,
}

impl Default for OpponentMoves {
    fn default() -> Self {
        Self {
            pieces: [None; 64],
            attacks: Default::default(),
            checkers: Default::default(),
            pinners: Default::default(),
            friendly_king: Default::default(),
        }
    }
}

impl OpponentMoves {
    pub fn generate(position: &Position) -> Self {
        let mut this = Self::default();

        this.friendly_king = position.board.get_piece_bb(Piece {
            color: position.active_color,
            kind: PieceKind::King,
        });

        let mut pos_opp = position.clone();
        pos_opp.active_color = pos_opp.active_color.opposite();
        pos_opp.ep_target = None;

        let friendly = pos_opp.board.get_color_bb(pos_opp.active_color);
        let unfriendly = pos_opp.board.get_color_bb(pos_opp.active_color.opposite());

        for square in friendly.set_iter() {
            let square_bb = BitBoard::from_square(square);

            // Check handlers
            let mut handle_sliding_checker = |pinner: [BitBoard; 4]| {
                for ray in pinner {
                    if ray & this.friendly_king != 0 {
                        let intersect = (ray & unfriendly).0.count_ones();

                        if intersect == 1 {
                            // only king intersects - check
                            this.checkers.sliders.push(square_bb | ray);
                        } else if intersect == 2 {
                            // king and one more piece intersect - pin
                            this.pinners.push((square, ray));
                        }
                    }
                }
            };

            // Generate moves
            let mut p_moves = BitBoard(0);
            let mut piece_kind = PieceKind::Pawn;

            // Bishop
            if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Bishop) != 0 {
                let slider = MoveList::sliding_attacks::<0>(square, friendly | unfriendly);
                let pinner = MoveList::sliding_attacks::<0>(square, friendly | this.friendly_king);

                (handle_sliding_checker)(pinner);

                p_moves = merge_bb(slider);
                piece_kind = PieceKind::Bishop;
            // Rook
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Rook) != 0 {
                let slider = MoveList::sliding_attacks::<1>(square, friendly | unfriendly);
                let pinner = MoveList::sliding_attacks::<1>(square, friendly | this.friendly_king);

                (handle_sliding_checker)(pinner);

                p_moves = merge_bb(slider);
                piece_kind = PieceKind::Rook;
            // Queen
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Queen) != 0 {
                // bishop moves first
                let slider = MoveList::sliding_attacks::<0>(square, friendly | unfriendly);
                let pinner = MoveList::sliding_attacks::<0>(square, friendly | this.friendly_king);

                (handle_sliding_checker)(pinner);

                p_moves = merge_bb(slider);

                // rook moves
                let slider = MoveList::sliding_attacks::<1>(square, friendly | unfriendly);
                let pinner = MoveList::sliding_attacks::<1>(square, friendly | this.friendly_king);

                (handle_sliding_checker)(pinner);

                p_moves |= merge_bb(slider);
                piece_kind = PieceKind::Queen;
            // Knight
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Knight) != 0 {
                let melee = MoveList::knight_attacks(square);

                if melee & this.friendly_king != 0 {
                    this.checkers.melee.push(square);
                }

                p_moves = melee;
                piece_kind = PieceKind::Knight;
            // Pawn
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Pawn) != 0 {
                let melee = MoveList::pawn_attacks(square, pos_opp.active_color);

                if melee & this.friendly_king != 0 {
                    this.checkers.melee.push(square);
                }

                p_moves = melee;
            // King
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::King) != 0 {
                let melee = MoveList::king_attacks(square);
                // king can't check another king
                p_moves = melee;
                piece_kind = PieceKind::King;
            }

            this.attacks = p_moves;
            this.pieces[square.raw_index() as usize] = Some(piece_kind);
        }

        this
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
                    min(7 - square.rank(), 7 - square.file()) as i8, // NE
                    min(7 - square.rank(), square.file()) as i8,     // NW
                    min(square.rank(), 7 - square.file()) as i8,     // SE
                    min(square.rank(), square.file()) as i8,         // SW
                ];

                (max, [[9, 7, 0, 0], [0, 0, 7, 9]])
            }
            1 => {
                // ORTHOGONAL
                let max = [
                    7 - square.rank() as i8, // N
                    square.rank() as i8,     // S
                    7 - square.file() as i8, // E
                    square.file() as i8,     // W
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

    /// Pre-calculated knight move table for each position on the board.
    const KNIGHT_ATTACKS: [BitBoard; 64] = {
        let mut all_moves = [BitBoard(0); 64];

        let mut i = 0;
        while i < 64 {
            let mut moves = 0;
            let square = Square::from_index_unchecked(i);
            let start = 1 << i;

            // NW
            if square.file() >= 2 && square.rank() < 7 {
                moves |= start << 6;
            }
            if square.file() >= 1 && square.rank() < 6 {
                moves |= start << 15;
            }
            // NE
            if square.file() <= 5 && square.rank() < 7 {
                moves |= start << 10;
            }
            if square.file() <= 6 && square.rank() < 6 {
                moves |= start << 17;
            }
            // SW
            if square.file() >= 2 && square.rank() >= 1 {
                moves |= start >> 10;
            }
            if square.file() >= 1 && square.rank() >= 2 {
                moves |= start >> 17;
            }
            // SE
            if square.file() <= 5 && square.rank() >= 1 {
                moves |= start >> 6;
            }
            if square.file() <= 6 && square.rank() >= 2 {
                moves |= start >> 15;
            }

            all_moves[i as usize] = BitBoard(moves);
            i += 1;
        }

        all_moves
    };

    fn knight_attacks(square: Square) -> BitBoard {
        Self::KNIGHT_ATTACKS[square.raw_index() as usize]
    }

    #[inline]
    pub fn pseudo_knight_moves(square: Square, position: &Position) -> BitBoard {
        Self::knight_attacks(square) & !position.board.get_color_bb(position.active_color)
    }

    fn pawn_attacks(square: Square, color: Color) -> BitBoard {
        let mut moves = BitBoard(0);
        let start = BitBoard::from_square(square);

        match color {
            Color::White => {
                if square.rank() < 7 {
                    if square.file() > 0 {
                        moves |= start << 7;
                    }
                    if square.file() < 7 {
                        moves |= start << 9;
                    }
                }
            }
            Color::Black => {
                if square.rank() > 0 {
                    if square.file() > 0 {
                        moves |= start >> 9;
                    }
                    if square.file() < 7 {
                        moves |= start >> 7;
                    }
                }
            }
        }

        moves
    }

    pub fn pseudo_pawn_moves(square: Square, position: &Position) -> PawnMoves {
        let mut moves = PawnMoves::default();
        let start = BitBoard::from_square(square);

        let attacks = Self::pawn_attacks(square, position.active_color);
        moves.attacks = attacks
            & position
                .board
                .get_color_bb(position.active_color.opposite());

        let blockers = position.board.get_full_bb();

        match position.active_color {
            Color::White => {
                let next = start << 8;

                if blockers & next == 0 {
                    // promotion
                    if square.rank() == 6 {
                        moves.promotions |= next;
                    }
                    // single push
                    else if square.rank() < 6 {
                        moves.pushes |= next;

                        // double push
                        let next_2 = next << 8;

                        if square.rank() == 1 && blockers & next_2 == 0 {
                            moves.pushes |= next_2;
                        }
                    }
                }

                // en passant
                if let Some(ep_target) = position.ep_target {
                    let offset = ep_target.raw_index().saturating_sub(square.raw_index());

                    if offset == 7 || offset == 9 {
                        moves.attacks |= BitBoard::from_square(ep_target);
                    }
                }
            }
            Color::Black => {
                // single push
                let next = start >> 8;

                if blockers & next == 0 {
                    // promotion
                    if square.rank() == 1 {
                        moves.promotions |= next;
                    }
                    // single push
                    else if square.rank() > 1 {
                        moves.pushes |= next;

                        // double push
                        let next_2 = next >> 8;

                        if square.rank() == 6 && blockers & next_2 == 0 {
                            moves.pushes |= next_2;
                        }
                    }
                }

                // en passant
                if let Some(ep_target) = position.ep_target {
                    let offset = square.raw_index().saturating_sub(ep_target.raw_index());

                    if offset == 7 || offset == 9 {
                        moves.attacks |= BitBoard::from_square(ep_target);
                    }
                }
            }
        }

        moves
    }

    /// Pre-calculated king move table for each position on the board.
    const KING_ATTACKS: [BitBoard; 64] = {
        let mut all_moves = [BitBoard(0); 64];

        let mut i = 0;
        while i < 64 {
            let mut moves = 0;
            let square = Square::from_index_unchecked(i);
            let start = 1 << i;

            // E
            if square.file() > 0 {
                moves |= start >> 9;
                moves |= start >> 1;
                moves |= start << 7;
            }
            // W
            if square.file() < 7 {
                moves |= start >> 7;
                moves |= start << 1;
                moves |= start << 9;
            }
            // N
            let mut mask = 0;
            mask |= start << 7;
            mask |= start << 8;
            mask |= start << 9;

            if square.rank() == 7 {
                moves &= !mask;
            } else {
                moves |= mask;
            }
            // S
            mask >>= 16;

            if square.rank() == 0 {
                moves &= !mask;
            } else {
                moves |= mask;
            }

            all_moves[i as usize] = BitBoard(moves);
            i += 1;
        }

        all_moves
    };

    fn king_attacks(square: Square) -> BitBoard {
        Self::KING_ATTACKS[square.raw_index() as usize]
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

#[derive(Debug, Clone, Default)]
pub struct PawnMoves {
    /// Possible moves.
    pushes: BitBoard,
    /// Squares defended by the pawn.
    attacks: BitBoard,
    /// Promotion squares.
    promotions: BitBoard,
}

impl PawnMoves {
    #[inline]
    pub fn reduce(&self) -> BitBoard {
        self.pushes | self.attacks | self.promotions
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
            test.do_gen(|x, y| MoveList::pseudo_pawn_moves(x, y).reduce());
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
