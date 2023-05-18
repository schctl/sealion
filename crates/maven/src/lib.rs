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

        if is_x86_feature_detected!("avx2") {
            unsafe {
                Self::sliding_moves_avx2(
                    square,
                    friendly,
                    unfriendly,
                    [9, 7, 0, 0],
                    [0, 0, 7, 9],
                    max_moves
                )
            }
        } else {
            Self::sliding_moves(square, friendly, unfriendly, [9, 7, -7, -9], max_moves)
        }
    }

    fn sliding_moves(
        square: Square,
        friendly: BitBoard,
        unfriendly: BitBoard,
        shifts: [i8; 4],
        max_moves: [i8; 4]
    ) -> BitBoard {
        let mut moves = [0; 4];

        for direction in 0..4 {
            for index in 1..(max_moves[direction] + 1) {
                let square = BitBoard(1 << (square.raw_index() as i8 + (index * shifts[direction])));

                if square & friendly != 0 {
                    break;
                }

                moves[direction] |= square.0;

                if square & unfriendly != 0 {
                    break;
                }
            }
        }

        BitBoard(moves[0] | moves[1] | moves[2] | moves[3])
    }

    #[target_feature(enable = "avx2")]
    unsafe fn sliding_moves_avx2(
        square: Square,
        friendly: BitBoard,
        unfriendly: BitBoard,
        lshifts: [i8; 4],
        rshifts: [i8; 4],
        max_moves: [i8; 4]
    ) -> BitBoard {
        use core::arch::x86_64::*;
        use std::mem::transmute;

        // Does:
        // a &= !(mask)((b & c) == c)
        let is_set_mask = |a: __m256i, b: __m256i, c: __m256i| -> __m256i {
            _mm256_andnot_si256(_mm256_cmpeq_epi64(_mm256_and_si256(b, c), c), a)
        };

        // Does
        // a &= (mask)(b < c)
        let and_lte_mask = |a: __m256i, b: __m256i, c: __m256i| -> __m256i {
            _mm256_andnot_si256(_mm256_cmpgt_epi64(b, c), a)
        };

        // Does
        // a |= b & c
        let or_and_mask = |a: __m256i, b: __m256i, c: __m256i| -> __m256i {
            _mm256_or_si256(a, _mm256_and_si256(b, c))
        };

        // Horizontal OR reduction to 64 bits
        let reduce_or = |a: __m256i| -> u64 {
            // OR across 128 bits
            let low_128 = _mm256_extracti128_si256::<0>(a);
            let high_128 = _mm256_extracti128_si256::<1>(a);
            let or_128 = _mm_or_si128(low_128, high_128);
            // OR across 64 bits
            let low_64 = _mm_extract_epi64::<0>(or_128);
            let high_64 = _mm_extract_epi64::<1>(or_128);

            transmute(low_64 | high_64)
        };

        // Compute max iterations
        let max_moves = max_moves.map(i64::from);
        let max_moves = _mm256_loadu_si256(max_moves.as_ptr() as *const _);

        // Direction iterator progress
        let lshifts = lshifts.map(i64::from);
        let rshifts = rshifts.map(i64::from);

        let lshifts = _mm256_loadu_si256(lshifts.as_ptr() as *const _);
        let rshifts = _mm256_loadu_si256(rshifts.as_ptr() as *const _);

        // Full move set
        let mut moves = _mm256_set1_epi64x(0);

        // Player masks
        let friendly = _mm256_set1_epi64x(transmute(friendly.0));
        let unfriendly = _mm256_set1_epi64x(transmute(unfriendly.0));

        // Direction enabled mask
        let mut enabled = _mm256_set1_epi64x(!0);

        // Isolated square iterator
        let mut square = _mm256_set1_epi64x(1 << square.raw_index());

        for index in 0..7 {
            // Get the next set of squares in the loop
            square = _mm256_sllv_epi64(square, lshifts);
            square = _mm256_srlv_epi64(square, rshifts);

            // Check if max moves exceeded
            enabled = and_lte_mask(enabled, _mm256_set1_epi64x(index + 1), max_moves);

            // Check for friendly blocker
            enabled = is_set_mask(enabled, friendly, square);

            // Register move
            moves = or_and_mask(moves, square, enabled);

            // Check for unfriendly blocker
            enabled = is_set_mask(enabled, unfriendly, square);
        }

        BitBoard(reduce_or(moves))
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
