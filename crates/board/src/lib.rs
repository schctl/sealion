use std::fmt::Display;
use std::ops::Not;
use std::str::FromStr;

pub mod fen;

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

/// Represents a player or a piece's color.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White = 1,
    Black = -1,
}

impl Color {
    /// Get the opposite color for this player.
    #[inline]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Not for Color {
    type Output = Self;

    /// Does [`Color::opposite`].
    #[inline]
    fn not(self) -> Self::Output {
        self.opposite()
    }
}

/// Piece belonging to a side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    color: Color,
    kind: PieceKind,
}

impl Piece {
    /// Standard representation for this piece.
    #[inline]
    pub const fn as_char(&self) -> char {
        match self.color {
            Color::White => match self.kind {
                PieceKind::Pawn => 'P',
                PieceKind::Knight => 'N',
                PieceKind::Bishop => 'B',
                PieceKind::Rook => 'R',
                PieceKind::Queen => 'Q',
                PieceKind::King => 'K',
            },
            Color::Black => match self.kind {
                PieceKind::Pawn => 'p',
                PieceKind::Knight => 'n',
                PieceKind::Bishop => 'b',
                PieceKind::Rook => 'r',
                PieceKind::Queen => 'q',
                PieceKind::King => 'k',
            },
        }
    }

    #[inline]
    pub const fn from_char(c: char) -> Option<Self> {
        let (color, kind) = match c {
            'P' => (Color::White, PieceKind::Pawn),
            'N' => (Color::White, PieceKind::Knight),
            'B' => (Color::White, PieceKind::Bishop),
            'R' => (Color::White, PieceKind::Rook),
            'Q' => (Color::White, PieceKind::Queen),
            'K' => (Color::White, PieceKind::King),
            // -
            'p' => (Color::Black, PieceKind::Pawn),
            'n' => (Color::Black, PieceKind::Knight),
            'b' => (Color::Black, PieceKind::Bishop),
            'r' => (Color::Black, PieceKind::Rook),
            'q' => (Color::Black, PieceKind::Queen),
            'k' => (Color::Black, PieceKind::King),
            _ => return None,
        };

        Some(Self { color, kind })
    }
}

/// A position on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub(crate) u8);

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

/// Piece position information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position([Option<Piece>; 64]);

impl Position {
    /// Empty position.
    #[inline]
    pub const fn empty() -> Self {
        Self([None; 64])
    }

    /// Get the piece present at a certain square.
    #[inline]
    pub fn get<ToSquare>(&self, index: ToSquare) -> Option<Piece>
    where
        Square: TryFrom<ToSquare>,
    {
        let square = Square::try_from(index).ok()?;
        self.0.get(square.0 as usize).copied().flatten()
    }

    /// Set the piece at a certain square.
    #[inline]
    pub fn set<ToSquare>(&mut self, index: ToSquare, piece: Option<Piece>) -> Option<Piece>
    where
        Square: TryFrom<ToSquare>,
    {
        let square = Square::try_from(index).ok()?;
        let old = self.0.get_mut(square.0 as usize)?;
        std::mem::replace(old, piece)
    }
}

bitflags::bitflags! {
    /// Player castling availability.
    pub struct CastlingRights: u8 {
        /// White kingside.
        const WHITE_OO = 0x01;
        /// White queenside.
        const WHITE_OOO = 0x02;
        /// Black kingside.
        const BLACK_OO = 0x04;
        /// Black queenside.
        const BLACK_OOO = 0x08;
    }
}

/// Full chessboard state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    /// Piece positions.
    position: Position,
    /// Whose turn it is to move.
    active_color: Color,
    /// Castling rights flags.
    castling: CastlingRights,
    /// En passant target square.
    ep_target: Option<Square>,
    /// Half-move (ply) clock.
    ///
    /// A half-move is a single move made by a single player. This counts the number of half-moves
    /// since the last capture, pawn move, or check; and is used for the 50-move rule.
    halfmove_clock: u8,
    /// Full-move counter.
    ///
    /// A full-move consists of two half-moves, one by white and one by black. This counts the total
    /// number of moves since the game began. It starts at 1 and increments after black's move.
    fullmove_counter: u8,
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
