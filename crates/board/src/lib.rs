use std::fmt::Display;
use std::str::FromStr;

/// All possible piece types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Piece belonging to a side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    White(PieceKind),
    Black(PieceKind),
}

impl Piece {
    #[inline]
    pub const fn as_char(&self) -> char {
        match self {
            Self::White(p) => match p {
                PieceKind::Pawn => 'P',
                PieceKind::Knight => 'N',
                PieceKind::Bishop => 'B',
                PieceKind::Rook => 'R',
                PieceKind::Queen => 'Q',
                PieceKind::King => 'K',
            },
            Self::Black(p) => match p {
                PieceKind::Pawn => 'p',
                PieceKind::Knight => 'n',
                PieceKind::Bishop => 'b',
                PieceKind::Rook => 'r',
                PieceKind::Queen => 'q',
                PieceKind::King => 'k',
            },
        }
    }
}

/// A position on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(u8);

impl Square {
    /// The square at a particular rank and file.
    #[inline]
    pub const fn at(rank: u8, file: u8) -> Option<Self> {
        if rank > 7 || file > 7 {
            return None;
        }

        Some(Self(rank * 8 + file))
    }

    /// Rank of this square.
    #[inline]
    pub const fn rank(&self) -> u8 {
        self.0 / 8
    }

    /// File of this square.
    #[inline]
    pub const fn file(&self) -> u8 {
        self.0 % 8
    }
}

impl TryFrom<(u8, u8)> for Square {
    type Error = ();

    /// Determine a square from a (rank, file) pair.
    #[inline]
    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        Self::at(value.0, value.1).ok_or(())
    }
}

impl FromStr for Square {
    type Err = ();

    /// Determine a square's position from algebraic notation.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(());
        }

        let rank = s.as_bytes()[0];
        let rank = rank
            .overflowing_sub(if rank > b'H' { b'a' } else { b'A' })
            .0;

        let file = s.as_bytes()[1];
        let file = file.overflowing_sub(b'1').0;

        Self::try_from((rank, file))
    }
}

impl Display for Square {
    /// Format the square into algebraic notation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            (self.rank() + b'a') as char,
            (self.file() + b'1') as char
        )
    }
}

/// Chessboard with piece positions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    internal: [Option<Piece>; 64],
}

impl Position {
    /// Get the piece present at a certain square.
    #[inline]
    pub fn get<ToSquare>(&self, index: ToSquare) -> Option<Piece>
    where
        Square: TryFrom<ToSquare>,
    {
        let square = Square::try_from(index).ok()?;
        self.internal.get(square.0 as usize).copied().flatten()
    }
}

#[cfg(test)]
mod square_tests {
    use super::*;

    #[test]
    fn square_to_str() {
        assert_eq!(&Square::at(0, 0).unwrap().to_string(), "a1");
        assert_eq!(&Square::at(5, 7).unwrap().to_string(), "f8");
        assert_eq!(&Square::at(4, 3).unwrap().to_string(), "e4");
        assert_eq!(&Square::at(2, 6).unwrap().to_string(), "c7");
        assert_eq!(&Square::at(8, 8), &None);
    }

    #[test]
    fn square_from_str() {
        assert_eq!(Square::from_str("a2"), Square::at(0, 1).ok_or(()));
        assert_eq!(Square::from_str("h8"), Square::at(7, 7).ok_or(()));
        assert_eq!(Square::from_str("C5"), Square::at(2, 4).ok_or(()));
        assert!(Square::from_str("5c").is_err());
        assert!(Square::from_str("b-").is_err());
        assert!(Square::from_str("^8").is_err());
        assert!(Square::from_str("b891").is_err());
        assert!(Square::from_str("b0").is_err());
    }
}
