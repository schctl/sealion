use paste::paste;

use sealion_board::Position;
use sealion_engine::movegen::MoveList;
use sealion_engine::state::PositionState;

pub fn perft(position: &Position, depth: usize, debug_depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;

    let state = PositionState::generate(&position);

    if let MoveList::Moves(moves) = MoveList::generate(&state) {
        for p_move in moves.into_iter() {
            let mut new_position = position.clone();
            new_position.apply_move_unchecked(p_move);
            let move_nodes = perft(&new_position, depth - 1, debug_depth);

            if depth == debug_depth {
                println!("{}: {}", p_move.to_move(), move_nodes);
            }

            nodes += move_nodes;
        }
    }

    nodes
}

fn do_perft(fen: &str, x: usize, result: usize) {
    let position = sealion_fen::from_str(fen).unwrap();
    let nodes = perft(&position, x, x);
    assert_eq!(nodes, result);
}

macro_rules! def_test {
    ($name:ident $fen:expr => [
        $($depth:expr => $result:expr),*
    ]) => {
        paste! {
            const [<$name:snake:upper>]: &'static str = $fen;

            $(
                #[test]
                fn [<$name:snake _perft_ $depth>]() {
                    do_perft([<$name:snake:upper>], $depth, $result);
                }
            )*
        }
    };
}

def_test! {
    start_pos "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" => [
        3 => 8_902,
        4 => 197_281,
        5 => 4_865_609
        // 6 => 119_060_324
    ]
}

def_test! {
    // https://www.chessprogramming.org/Perft_Results#Position_5
    pos_5 "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8" => [
        1 => 44,
        2 => 1_486,
        3 => 62_379,
        4 => 2_103_487
    ]
}
