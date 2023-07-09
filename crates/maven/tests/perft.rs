use sealion_board::Position;
use sealion_maven::MoveList;

fn perft(position: &Position, depth: usize, debug_depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;

    if let MoveList::Moves(moves) = MoveList::generate(&position) {
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

fn do_perft_x(x: usize, result: usize) {
    let position = Position::starting();
    let nodes = perft(&position, x, x);
    assert_eq!(nodes, result);
}

#[test]
fn do_perft_3() {
    do_perft_x(3, 8_902)
}

#[test]
fn do_perft_4() {
    do_perft_x(4, 197_281)
}

#[test]
fn do_perft_5() {
    do_perft_x(5, 4_865_609)
}
