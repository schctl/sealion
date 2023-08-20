//! Extended board state.

use sealion_board::{BitBoard, Capture, Piece, PieceKind, Position, Square};
use smallvec::SmallVec;

use PieceKind::*;

use crate::movegen::{merge_bb, Generator};

#[derive(Debug, Clone)]
pub struct BoardExt {
    pub pieces: [Option<Piece>; 64],
    pub king_bb: BitBoard,
}

impl BoardExt {
    #[inline]
    pub const fn get(&self, square: Square) -> Option<Piece> {
        self.pieces[square.raw_index() as usize]
    }
}

impl Default for BoardExt {
    #[inline]
    fn default() -> Self {
        Self {
            pieces: [None; 64],
            king_bb: BitBoard::ZERO,
        }
    }
}

/// Pseudo evaluation after an initial run through of the position.
#[derive(Debug, Clone, Default)]
pub struct PseudoScore {
    pub pieces: i16,
    pub position: i16,
    pub attacked: i16,
    // -- movegen
    pub attack: i16,
    // --
}

#[derive(Debug, Clone, Default)]
pub struct Checkers {
    /// Nearby attackers.
    ///
    /// Have to be captured or evaded by king.
    pub melee: SmallVec<[Square; 2]>,
    /// Faraway attacker ray.
    ///
    /// Have to be captured, evaded or blocked.
    pub sliders: SmallVec<[BitBoard; 2]>,
}

/// Opponent attacking information.
#[derive(Debug, Clone, Default)]
pub struct Attacks {
    /// Attacked squares to restrict king movement.
    pub bb: BitBoard,
    /// Attackers on our king.
    pub checkers: Checkers,
    /// Sliders pinning pieces to the king.
    ///
    /// Restrict movement for those piece only along the pinning ray.
    pub pinners: SmallVec<[BitBoard; 2]>,
}

/// Extended position information.
#[derive(Debug, Clone)]
pub struct PositionState<'a> {
    pub position: &'a Position,
    pub board_ext: BoardExt,
    pub score: PseudoScore,
    pub attacks: Attacks,
}

impl<'a> PositionState<'a> {
    pub fn generate(position: &'a Position) -> Self {
        let mut this = Self {
            position,
            board_ext: BoardExt::default(),
            score: PseudoScore::default(),
            attacks: Attacks::default(),
        };

        this.board_ext.king_bb = position.board.get_piece_bb(Piece {
            color: position.active_color,
            kind: PieceKind::King,
        });

        for square in position.board.get_full_bb().set_iter() {
            if let Some(piece) = position.board.get(square) {
                this.board_ext.pieces[square.raw_index() as usize] = Some(piece);

                if piece.color == position.active_color {
                    this.score.pieces += piece.kind.score();
                } else {
                    this.score.pieces -= piece.kind.score();
                    this.generate_attacks(square, piece.kind);
                }

                // TODO: positional + attacked score
            }
        }

        this
    }

    #[inline]
    fn generate_attacks(&mut self, square: Square, kind: PieceKind) {
        let square_bb = BitBoard::from_square(square);

        let friendly = self.position.board.get_color_bb(self.position.active_color);
        let minions = friendly & !self.board_ext.king_bb;
        let unfriendly = self
            .position
            .board
            .get_color_bb(self.position.active_color.opposite());

        let mut handle_king_atk = |pinner: [BitBoard; 4]| {
            for ray in pinner {
                if ray & self.board_ext.king_bb != 0 {
                    let intersect = ray & unfriendly;
                    let n_intersect = intersect.0.count_ones();

                    if n_intersect == 1 {
                        // only king intersects - check
                        self.attacks.checkers.sliders.push(square_bb | ray);
                    } else if n_intersect == 2 {
                        // king and one more piece intersect - pin
                        self.attacks.pinners.push(square_bb | ray);
                    }

                    break;
                }
            }
        };

        match kind {
            Bishop => {
                let king_atk =
                    Generator::sliding_attacks::<0>(square, unfriendly | self.board_ext.king_bb);
                (handle_king_atk)(king_atk);

                // ignore king while generating ray attacks
                // this is so king movement is restricted along the ray as well
                // also will reveal hidden moves during evaluation
                let attack = Generator::sliding_attacks::<0>(square, unfriendly | minions);
                self.attacks.bb |= merge_bb(attack);
            }
            Rook => {
                let king_atk =
                    Generator::sliding_attacks::<1>(square, unfriendly | self.board_ext.king_bb);
                (handle_king_atk)(king_atk);

                let attack = Generator::sliding_attacks::<1>(square, unfriendly | minions);
                self.attacks.bb |= merge_bb(attack);
            }
            Queen => {
                self.generate_attacks(square, Bishop);
                self.generate_attacks(square, Rook);
            }
            Knight => {
                let attack = Generator::knight_attacks(square);

                if attack & self.board_ext.king_bb != 0 {
                    self.attacks.checkers.melee.push(square);
                }

                self.attacks.bb |= attack;
            }
            Pawn => {
                let attack = Generator::pawn_attacks(square, self.position.active_color.opposite());

                if attack & self.board_ext.king_bb != 0 {
                    self.attacks.checkers.melee.push(square);
                }

                self.attacks.bb |= attack;
            }
            King => {
                let attack = Generator::king_attacks(square);
                // king can't check another king
                self.attacks.bb |= attack;
            }
        }
    }

    #[inline]
    pub fn resolve_capture_only(&self, to_sq: Square) -> Option<Capture> {
        if let Some(piece) = self.board_ext.pieces[to_sq.raw_index() as usize] {
            return Some(Capture::Regular(piece.kind));
        }

        None
    }

    #[inline]
    pub fn resolve_ep(&self, to_sq: Square) -> Option<Capture> {
        if Some(to_sq) == self.position.ep_target {
            return Some(Capture::EnPassant);
        }

        None
    }

    #[inline]
    pub fn resolve_capture(&self, to_sq: Square) -> Option<Capture> {
        self.resolve_capture_only(to_sq)
            .or_else(|| self.resolve_ep(to_sq))
    }
}
