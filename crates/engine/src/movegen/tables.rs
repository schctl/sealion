//! Pre-computed tables for piece moves.

use sealion_board::{BitBoard, Square};

pub const PAWN_ATTACKS: [BitBoard; 128] = {
    let mut all_moves = [BitBoard::ZERO; 128];

    // white
    let mut i = 0;
    while i < 64 {
        let mut moves = 0;
        let square = Square::from_index_unchecked(i);
        let start = 1 << i;

        if square.rank() < 7 {
            if square.file() > 0 {
                moves |= start << 7;
            }
            if square.file() < 7 {
                moves |= start << 9;
            }
        }

        all_moves[i as usize] = BitBoard(moves);
        i += 1;
    }

    // black
    let mut i = 0;
    while i < 64 {
        let mut moves = 0;
        let square = Square::from_index_unchecked(i);
        let start = 1 << i;

        if square.rank() > 0 {
            if square.file() > 0 {
                moves |= start >> 9;
            }
            if square.file() < 7 {
                moves |= start >> 7;
            }
        }

        all_moves[i as usize + 64] = BitBoard(moves);
        i += 1;
    }

    all_moves
};

pub const KNIGHT_ATTACKS: [BitBoard; 64] = {
    let mut all_moves = [BitBoard::ZERO; 64];

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

pub const KING_ATTACKS: [BitBoard; 64] = {
    let mut all_moves = [BitBoard::ZERO; 64];

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
        let mut mask = 0;
        mask |= start >> 7;
        mask |= start >> 8;
        mask |= start >> 9;

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
