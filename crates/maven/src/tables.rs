//! Pre-generated shadow tables.

use sealion_board::{BitBoard, Square};

use direction::*;

const fn min(v1: u8, v2: u8) -> u8 {
    if v1 <= v2 {
        v1
    } else {
        v2
    }
}

/// Direction enums.
pub(crate) mod direction {
    use super::*;

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Orthogonal {
        North,
        South,
        East,
        West,
    }

    impl Orthogonal {
        pub const fn shift(&self) -> i8 {
            match self {
                Self::North => 8,
                Self::South => -8,
                Self::East => 1,
                Self::West => -1,
            }
        }

        pub const fn max_iters(&self, square: Square) -> i8 {
            match self {
                Self::North => 7 - square.rank() as i8,
                Self::South => square.rank() as i8,
                Self::East => 7 - square.file() as i8,
                Self::West => square.file() as i8,
            }
        }
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Diagonal {
        NorthEast,
        NorthWest,
        SouthEast,
        SouthWest,
    }

    impl Diagonal {
        pub const fn shift(&self) -> i8 {
            match self {
                Self::NorthEast => 9,
                Self::NorthWest => 7,
                Self::SouthEast => -7,
                Self::SouthWest => -9,
            }
        }

        pub const fn max_iters(&self, square: Square) -> i8 {
            (match self {
                Self::NorthEast => min(7 - square.file(), 7 - square.rank()),
                Self::NorthWest => min(square.file(), 7 - square.rank()),
                Self::SouthEast => min(7 - square.file(), square.rank()),
                Self::SouthWest => min(square.file(), square.rank()),
            }) as i8
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Direction {
        Orthogonal(Orthogonal),
        Diagonal(Diagonal),
    }

    impl Direction {
        pub const fn shift(&self) -> i8 {
            match self {
                Self::Orthogonal(o) => o.shift(),
                Self::Diagonal(o) => o.shift(),
            }
        }

        pub const fn max_iters(&self, square: Square) -> i8 {
            match self {
                Self::Orthogonal(o) => o.max_iters(square),
                Self::Diagonal(o) => o.max_iters(square),
            }
        }
    }
}

const fn generate_shadow(square: Square, max_iters: i8, shift: i8) -> BitBoard {
    let mut bb = BitBoard(0);

    let mut iter = 1;

    while iter <= max_iters {
        bb.0 |= 1 << (square.raw_index() as i8 + (iter * shift));
        iter += 1;
    }

    bb
}

const fn generate_shadows(direction: direction::Direction) -> [BitBoard; 64] {
    let mut dir_shadows = [BitBoard(0); 64];

    let mut index = 0;

    while index < 64 {
        let square = Square::from_index_unchecked(index);
        dir_shadows[index as usize] =
            generate_shadow(square, direction.max_iters(square), direction.shift());
        index += 1;
    }

    dir_shadows
}

pub type Shadows = [[BitBoard; 64]; 4];

pub const ORTHOGONAL_SHADOWS: [[BitBoard; 64]; 4] = [
    generate_shadows(Direction::Orthogonal(Orthogonal::North)),
    generate_shadows(Direction::Orthogonal(Orthogonal::South)),
    generate_shadows(Direction::Orthogonal(Orthogonal::East)),
    generate_shadows(Direction::Orthogonal(Orthogonal::West)),
];

pub const DIAGONAL_SHADOWS: [[BitBoard; 64]; 4] = [
    generate_shadows(Direction::Diagonal(Diagonal::NorthEast)),
    generate_shadows(Direction::Diagonal(Diagonal::NorthWest)),
    generate_shadows(Direction::Diagonal(Diagonal::SouthEast)),
    generate_shadows(Direction::Diagonal(Diagonal::SouthWest)),
];
