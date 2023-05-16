//! Move generator implementation.

#![feature(portable_simd)]
#![feature(const_trait_impl)]

use std::simd::{u64x4, u64x8, SimdUint};

use sealion_board::{BitBoard, Square};

pub mod tables;

/// The primary structure which contains relevant piece state information, such as attacks and checks.
pub struct Maven {}

impl Maven {
    pub fn bishop_moves(square: Square, friendly: BitBoard, unfriendly: BitBoard) -> BitBoard {
        let mut quasi_move_table = u64x4::splat(0);

        for i in 0..4 {
            quasi_move_table[i] = tables::DIAGONAL_SHADOWS[i][square.raw_index() as usize].0;
        }

        let quasi_moves = BitBoard(quasi_move_table.reduce_or());

        // friendly blockers
        let all_blockers = quasi_move_table & u64x4::splat((friendly | unfriendly).0);
        let our_blockers = quasi_moves & friendly;

        let shadow = Self::diagonal_shadow_or(all_blockers) | our_blockers;

        quasi_moves ^ shadow
    }

    /// Generate diagonal shadow across only one direction.
    pub fn diagonal_shadow_or(blockers: u64x4) -> BitBoard {
        let mut shadow = u64x8::splat(0);

        for index in 0..8 {
            shadow[index] = chunk_diagonal_shadow_mask(blockers, index * 8);
        }

        BitBoard(shadow.reduce_or())
    }
}

pub fn partial_diagonal_shadow_mask(direction: usize, blockers: u64, index: usize) -> u64 {
    // check if the current square
    // 1) blocks movement (contains a piece)
    // 2) is only in the direction of movement
    let valid_shadow = (blockers >> index) & 1;

    tables::DIAGONAL_SHADOWS[direction][index].0 & !(valid_shadow.wrapping_sub(1))
}

pub fn chunk_partial_diagonal_shadow_mask(direction: usize, blockers: u64, index: usize) -> u64 {
    let mut mask = u64x8::splat(0);

    for shift in 0..8 {
        mask[shift] = partial_diagonal_shadow_mask(direction, blockers, index + shift);
    }

    mask.reduce_or()
}

pub fn chunk_diagonal_shadow_mask(blockers: u64x4, index: usize) -> u64 {
    chunk_partial_diagonal_shadow_mask(0, blockers[0], index) |
    chunk_partial_diagonal_shadow_mask(1, blockers[1], index) |
    chunk_partial_diagonal_shadow_mask(2, blockers[2], index) |
    chunk_partial_diagonal_shadow_mask(3, blockers[3], index)
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
