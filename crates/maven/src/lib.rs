//! Move generator implementation.

use std::cmp::min;

use sealion_board::{BitBoard, Square};

/// The primary structure which contains relevant piece state information, such as attacks and checks.
pub struct Maven {}

impl Maven {
    pub fn bishop_moves(square: Square, friendly: BitBoard, unfriendly: BitBoard) -> BitBoard {
        let max_moves = [
            min(7 - square.rank(), 7 - square.file()) as i8,
            min(7 - square.rank(), square.file()) as i8,
            min(square.rank(), 7 - square.file()) as i8,
            min(square.rank(), square.file()) as i8,
        ];

        Self::sliding_moves(square, friendly, unfriendly, [[9, 7, 0, 0], [0, 0, 7, 9]], max_moves)
    }

    fn sliding_moves(
        square: Square,
        friendly: BitBoard,
        unfriendly: BitBoard,
        shifts: [[i8; 4]; 2],
        max_moves: [i8; 4]
    ) -> BitBoard {
        let mut moves = BitBoard(0);

        for direction in 0..4 {
            let mut square = BitBoard(1 << square.raw_index());

            for _ in 1..(max_moves[direction] + 1) {
                square <<= shifts[0][direction];
                square >>= shifts[1][direction];

                if square & friendly != 0 {
                    break;
                }

                moves |= square;

                if square & unfriendly != 0 {
                    break;
                }
            }
        }

        moves
    }
}

#[cfg(test)]
mod test {
    use sealion_board::{Board, Color};

    use super::*;

    #[test]
    fn test() {
        let start = Board::starting_position();
        let sq = Square::at(4, 5).unwrap();

        let result = Maven::bishop_moves(
            sq,
            start.get_color_bb(Color::White),
            start.get_color_bb(Color::Black),
        );

        assert_eq!(result, 0x88_50_00_50_88_00_00);
    }
}
