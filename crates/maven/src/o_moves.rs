//! Opponent moves generation.

use sealion_board::{BitBoard, Capture, Piece, PieceKind, Position, Square};
use smallvec::SmallVec;

use super::{merge_bb, MoveList};

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
    /// Friendly king square.
    pub friendly_king: BitBoard,
}

impl Default for OpponentMoves {
    fn default() -> Self {
        Self {
            pieces: [None; 64],
            attacks: Default::default(),
            checkers: Default::default(),
            pinners: Default::default(),
            friendly_king: Default::default(),
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

    pub fn generate(position: &Position) -> Self {
        let mut this = Self::default();

        this.friendly_king = position.board.get_piece_bb(Piece {
            color: position.active_color,
            kind: PieceKind::King,
        });

        let mut pos_opp = position.clone();
        pos_opp.active_color = pos_opp.active_color.opposite();
        pos_opp.ep_target = None;

        let friendly = pos_opp.board.get_color_bb(pos_opp.active_color);
        let unfriendly = pos_opp.board.get_color_bb(pos_opp.active_color.opposite());
        let unfriendly_minions = unfriendly & !this.friendly_king;

        for square in friendly.set_iter() {
            let square_bb = BitBoard::from_square(square);

            // Handle pins/checks
            let mut handle_pin = |pinner: [BitBoard; 4]| {
                for ray in pinner {
                    if ray & this.friendly_king != 0 {
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
            let mut p_moves = BitBoard(0);
            let mut piece_kind = PieceKind::Pawn;

            // Bishop
            if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Bishop) != 0 {
                let attack = MoveList::sliding_attacks::<0>(square, friendly | unfriendly_minions);
                let pinner = MoveList::sliding_attacks::<0>(square, friendly | this.friendly_king);

                (handle_pin)(pinner);

                p_moves = merge_bb(attack);
                piece_kind = PieceKind::Bishop;
            // Rook
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Rook) != 0 {
                let attack = MoveList::sliding_attacks::<1>(square, friendly | unfriendly_minions);
                let pinner = MoveList::sliding_attacks::<1>(square, friendly | this.friendly_king);

                (handle_pin)(pinner);

                p_moves = merge_bb(attack);
                piece_kind = PieceKind::Rook;
            // Queen
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Queen) != 0 {
                // bishop moves
                let attack_b =
                    MoveList::sliding_attacks::<0>(square, friendly | unfriendly_minions);
                let pinner_b =
                    MoveList::sliding_attacks::<0>(square, friendly | this.friendly_king);
                // rook moves
                let attack_r =
                    MoveList::sliding_attacks::<1>(square, friendly | unfriendly_minions);
                let pinner_r =
                    MoveList::sliding_attacks::<1>(square, friendly | this.friendly_king);

                (handle_pin)(pinner_b);
                (handle_pin)(pinner_r);

                p_moves = merge_bb(attack_b) | merge_bb(attack_r);
                piece_kind = PieceKind::Queen;
            // Knight
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Knight) != 0 {
                let melee = MoveList::knight_attacks(square);

                if melee & this.friendly_king != 0 {
                    this.checkers.melee.push(square);
                }

                p_moves = melee;
                piece_kind = PieceKind::Knight;
            // Pawn
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::Pawn) != 0 {
                let melee = MoveList::pawn_attacks(square, pos_opp.active_color);

                if melee & this.friendly_king != 0 {
                    this.checkers.melee.push(square);
                }

                p_moves = melee;
            // King
            } else if square_bb & pos_opp.board.get_piece_kind_bb(PieceKind::King) != 0 {
                let melee = MoveList::king_attacks(square);
                // king can't check another king
                p_moves = melee;
                piece_kind = PieceKind::King;
            }

            this.attacks |= p_moves;
            this.pieces[square.raw_index() as usize] = Some(piece_kind);
        }

        this
    }
}
