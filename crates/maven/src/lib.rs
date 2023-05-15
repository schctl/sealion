//! Move generator implementation.

#![feature(portable_simd)]
#![feature(const_trait_impl)]

use std::simd::{u64x8, SimdUint};

use sealion_board::{BitBoard, Square};

pub mod tables;
use tables::direction;

/// The primary structure which contains relevant piece state information, such as attacks and checks.
pub struct Maven {}

impl Maven {
    pub fn bishop_moves(square: Square, friendly: BitBoard, unfriendly: BitBoard) -> BitBoard {
        let quasi_move_table = [
            tables::DIAGONAL_SHADOWS[0][square.raw_index() as usize],
            tables::DIAGONAL_SHADOWS[1][square.raw_index() as usize],
            tables::DIAGONAL_SHADOWS[2][square.raw_index() as usize],
            tables::DIAGONAL_SHADOWS[3][square.raw_index() as usize],
        ];

        let quasi_moves =
            quasi_move_table[0] | quasi_move_table[1] | quasi_move_table[2] | quasi_move_table[3];

        // friendly blockers
        let all_blockers = quasi_moves & (friendly | unfriendly);
        let our_blockers = quasi_moves & friendly;

        let shadow = Self::diagonal_shadow_or(&quasi_move_table, all_blockers) | our_blockers;

        quasi_moves ^ shadow
    }

    /// Generate diagonal shadow across only one direction.
    fn diagonal_shadow_or(move_tables: &[BitBoard; 4], blockers: BitBoard) -> BitBoard {
        use direction::Diagonal;

        let mut shadow = [u64x8::splat(0); 4];

        for index in (0..64).step_by(8) {
            let square = 1 << index;

            shadow[0] |= shadow_or_mask_simd(
                move_tables[Diagonal::NorthEast as u8 as usize],
                Diagonal::NorthEast,
                blockers,
                index,
                square,
            );
            shadow[1] |= shadow_or_mask_simd(
                move_tables[Diagonal::NorthWest as u8 as usize],
                Diagonal::NorthWest,
                blockers,
                index,
                square,
            );
            shadow[2] |= shadow_or_mask_simd(
                move_tables[Diagonal::SouthEast as u8 as usize],
                Diagonal::SouthEast,
                blockers,
                index,
                square,
            );
            shadow[3] |= shadow_or_mask_simd(
                move_tables[Diagonal::SouthWest as u8 as usize],
                Diagonal::SouthWest,
                blockers,
                index,
                square,
            );
        }

        BitBoard(
            shadow[0].reduce_or()
                | shadow[1].reduce_or()
                | shadow[2].reduce_or()
                | shadow[3].reduce_or(),
        )
    }
}

const fn shadow_or_mask(
    move_table: BitBoard,
    direction: direction::Diagonal,
    blockers: BitBoard,
    index: usize,
    square: u64,
) -> u64 {
    // check if the current square
    // 1) blocks movement (contains a piece)
    // 2) is only in the direction of movement
    let valid_shadow = square & blockers.0 & move_table.0 > 0;

    tables::DIAGONAL_SHADOWS[direction as u8 as usize][index].0
        & !(valid_shadow as u64).wrapping_sub(1)
}

const fn shadow_or_mask_simd(
    move_table: BitBoard,
    direction: direction::Diagonal,
    blockers: BitBoard,
    index: usize,
    square: u64,
) -> u64x8 {
    u64x8::from_array([
        shadow_or_mask(move_table, direction, blockers, index, square),
        shadow_or_mask(move_table, direction, blockers, index + 1, square << 1),
        shadow_or_mask(move_table, direction, blockers, index + 2, square << 2),
        shadow_or_mask(move_table, direction, blockers, index + 3, square << 3),
        shadow_or_mask(move_table, direction, blockers, index + 4, square << 4),
        shadow_or_mask(move_table, direction, blockers, index + 5, square << 5),
        shadow_or_mask(move_table, direction, blockers, index + 6, square << 6),
        shadow_or_mask(move_table, direction, blockers, index + 7, square << 7),
    ])
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
