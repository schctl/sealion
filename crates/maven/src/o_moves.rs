//! Opponent moves generation.

use sealion_board::{BitBoard, Capture, PieceKind, Position, Square};
use smallvec::SmallVec;

use super::{merge_bb, Generator};

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

/// Pseudo moves for the opponent. Used to calculate checks and pins.
#[derive(Debug, Clone)]
pub struct OpponentMoves {
    /// Pre-calculated piece kinds for each square.
    pub pieces: [Option<PieceKind>; 64],
    /// Attacked squares to restrict king movement.
    pub attacks: BitBoard,
    /// Attackers on our king.
    pub checkers: Checkers,
    /// Sliders pinning pieces to the king.
    ///
    /// Restrict movement for those piece only along the pinning ray.
    pub pinners: SmallVec<[BitBoard; 2]>,
}

impl Default for OpponentMoves {
    fn default() -> Self {
        Self {
            pieces: [None; 64],
            attacks: Default::default(),
            checkers: Default::default(),
            pinners: Default::default(),
        }
    }
}

impl OpponentMoves {
    #[inline]
    pub const fn resolve_capture(&self, square: Square) -> Option<Capture> {
        if let Some(piece) = self.pieces[square.raw_index() as usize] {
            return Some(Capture::Regular(piece));
        }

        None
    }

    #[inline]
    pub fn resolve_ep(&self, to_sq: Square, ep_target: Option<Square>) -> Option<Capture> {
        if let Some(ep_target) = ep_target {
            if to_sq == ep_target {
                return Some(Capture::EnPassant);
            }
        }

        None
    }

    pub fn generate(position: &Position, friendly_king: BitBoard) -> Self {
        let mut this = Self::default();

        let mut pos_opp = position.clone();
        pos_opp.active_color = pos_opp.active_color.opposite();
        pos_opp.ep_target = None;

        let friendly = pos_opp.board.get_color_bb(pos_opp.active_color);
        let unfriendly = pos_opp.board.get_color_bb(pos_opp.active_color.opposite());
        let unfriendly_minions = unfriendly & !friendly_king;

        for square in friendly.set_iter() {
            let square_bb = BitBoard::from_square(square);

            // Handle pins/checks
            let mut handle_pin = |pinner: [BitBoard; 4]| {
                for ray in pinner {
                    if ray & friendly_king != 0 {
                        let intersect = ray & unfriendly;
                        let n_intersect = intersect.0.count_ones();

                        if n_intersect == 1 {
                            // only king intersects - check
                            this.checkers.sliders.push(square_bb | ray);
                        } else if n_intersect == 2 {
                            // king and one more piece intersect - pin
                            this.pinners.push(square_bb | ray);
                        }

                        break;
                    }
                }
            };

            // Generate moves
            let mut p_moves = BitBoard::ZERO;
            let mut p_kind = PieceKind::Pawn;

            // Bishop
            if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Bishop) != 0 {
                let attack = Generator::sliding_attacks::<0>(square, friendly | unfriendly_minions);
                let pinner = Generator::sliding_attacks::<0>(square, friendly | friendly_king);

                (handle_pin)(pinner);

                p_moves = merge_bb(attack);
                p_kind = PieceKind::Bishop;
            // Rook
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Rook) != 0 {
                let attack = Generator::sliding_attacks::<1>(square, friendly | unfriendly_minions);
                let pinner = Generator::sliding_attacks::<1>(square, friendly | friendly_king);

                (handle_pin)(pinner);

                p_moves = merge_bb(attack);
                p_kind = PieceKind::Rook;
            // Queen
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Queen) != 0 {
                // bishop moves
                let attack_b =
                    Generator::sliding_attacks::<0>(square, friendly | unfriendly_minions);
                let pinner_b = Generator::sliding_attacks::<0>(square, friendly | friendly_king);
                // rook moves
                let attack_r =
                    Generator::sliding_attacks::<1>(square, friendly | unfriendly_minions);
                let pinner_r = Generator::sliding_attacks::<1>(square, friendly | friendly_king);

                (handle_pin)(pinner_b);
                (handle_pin)(pinner_r);

                p_moves = merge_bb(attack_b) | merge_bb(attack_r);
                p_kind = PieceKind::Queen;
            // Knight
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Knight) != 0 {
                let melee = Generator::knight_attacks(square);

                if melee & friendly_king != 0 {
                    this.checkers.melee.push(square);
                }

                p_moves = melee;
                p_kind = PieceKind::Knight;
            // Pawn
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Pawn) != 0 {
                let melee = Generator::pawn_attacks(square, pos_opp.active_color);

                if melee & friendly_king != 0 {
                    this.checkers.melee.push(square);
                }

                p_moves = melee;
            // King
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::King) != 0 {
                let melee = Generator::king_attacks(square);
                // king can't check another king
                p_moves = melee;
                p_kind = PieceKind::King;
            }

            this.attacks |= p_moves;
            this.pieces[square.raw_index() as usize] = Some(p_kind);
        }

        this
    }
}
