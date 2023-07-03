//! Move generator implementation.
//!
//! Why is it called Maven? I dunno. It sounds better than "movegen" tho.

#![feature(const_trait_impl)]

use std::cmp::min;

use sealion_board::{BitBoard, Color, Position, Square};

/// The primary structure which contains relevant piece state information, such as attacks and checks.
pub struct Maven {}

impl Maven {
    pub fn bishop_moves(square: Square, position: &Position) -> BitBoard {
        Self::sliding_moves::<0>(square, position)
    }

    pub fn rook_moves(square: Square, position: &Position) -> BitBoard {
        Self::sliding_moves::<1>(square, position)
    }

    pub fn queen_moves(square: Square, position: &Position) -> BitBoard {
        Self::sliding_moves::<0>(square, position) | Self::sliding_moves::<1>(square, position)
    }

    fn sliding_moves<const DIR: u8>(square: Square, position: &Position) -> BitBoard {
        let friendly = position.board.get_color_bb(position.active_color);
        let unfriendly = position
            .board
            .get_color_bb(position.active_color.opposite());

        let moves = Self::sliding_moves_impl::<DIR>(square, friendly, unfriendly);

        moves[0] | moves[1] | moves[2] | moves[3]
    }

    fn sliding_moves_impl<const DIR: u8>(
        square: Square,
        friendly: BitBoard,
        unfriendly: BitBoard,
    ) -> [BitBoard; 4] {
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
            _ => panic!("disallowed value for sliding move direction (should be 1 or 0)"),
        };

        let square = BitBoard::from_square(square);

        for direction in 0..4 {
            let mut square = square;

            for _ in 0..max[direction] {
                square <<= shifts[0][direction];
                square >>= shifts[1][direction];

                if square & friendly != 0 {
                    break;
                }

                moves[direction] |= square;

                if square & unfriendly != 0 {
                    break;
                }
            }
        }

        moves
    }

    /// Pre-calculated knight move table for each position on the board.
    const KNIGHT_MOVES: [BitBoard; 64] = {
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

    pub fn knight_moves(square: Square, position: &Position) -> BitBoard {
        let moves = Self::KNIGHT_MOVES[square.raw_index() as usize];
        moves & !position.board.get_color_bb(position.active_color)
    }

    pub fn pawn_moves(square: Square, position: &Position) -> BitBoard {
        let mut moves = BitBoard(0);

        let start = BitBoard::from_square(square);

        match position.active_color {
            Color::White => {
                // single push
                let next = start << 8;

                if position.board.get_full_bb() & next == 0 {
                    moves |= next;

                    // double push
                    let next = next << 8;

                    if position.board.get_full_bb() & next == 0 {
                        moves |= next;
                    }
                }

                // diagonal capture
                let diag = (start << 7) | (start << 9);
                moves |= diag
                    & position
                        .board
                        .get_color_bb(position.active_color.opposite());

                // en passant
                if let Some(ep_target) = position.ep_target {
                    let offset = ep_target.raw_index().saturating_sub(square.raw_index());

                    if offset == 7 || offset == 9 {
                        moves |= BitBoard::from_square(ep_target);
                    }
                }
            }
            Color::Black => {
                // single push
                let next = start >> 8;

                if position.board.get_full_bb() & next == 0 {
                    moves |= next;

                    // double push
                    let next = next >> 8;

                    if position.board.get_full_bb() & next == 0 {
                        moves |= next;
                    }
                }

                // diagonal capture
                let diag = (start >> 7) | (start >> 9);
                moves |= diag
                    & position
                        .board
                        .get_color_bb(position.active_color.opposite());

                // en passant
                if let Some(ep_target) = position.ep_target {
                    let offset = square.raw_index().saturating_sub(ep_target.raw_index());

                    if offset == 7 || offset == 9 {
                        moves |= BitBoard::from_square(ep_target);
                    }
                }
            }
        }

        moves
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
        let tests = [
            MoveTester {
                name: "start with bishop",
                sq: (4, 5),
                fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                result: 0x88_50_00_50_88_00_00,
            },
            MoveTester {
                name: "mostly empty bishop pin",
                sq: (4, 3),
                fen: "8/1b6/8/3B4/8/5K2/1k6/8 w - - 0 1",
                result: 0x02_04_00_10_00_00_00,
            },
        ];

        for test in tests {
            test.do_gen(Maven::bishop_moves);
        }
    }

    #[test]
    fn rook_moves() {
        let tests = [
            MoveTester {
                name: "start with bishop",
                sq: (4, 5),
                fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                result: 0x20_20_df_20_20_00_00,
            },
            MoveTester {
                name: "mostly empty bishop to rook pin",
                sq: (2, 3),
                fen: "8/8/8/1b6/8/3R4/1k2K3/8 w - - 0 1",
                result: 0,
            },
        ];

        for test in tests {
            test.do_gen(Maven::rook_moves);
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
            test.do_gen(Maven::knight_moves);
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
            test.do_gen(Maven::pawn_moves);
        }
    }
}
