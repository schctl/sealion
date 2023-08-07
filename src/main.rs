use std::io::stdin;

use sealion_maven::MoveList;

fn main() {
    println!("Position fen: ");
    let mut fen = String::new();
    stdin().read_line(&mut fen).unwrap();

    let position = sealion_fen::from_str(&fen).unwrap();

    match MoveList::generate(&position) {
        MoveList::Checkmate => println!("Checkmate"),
        MoveList::Stalemate => println!("Stalemate"),
        MoveList::Moves(moves) => {
            for p_move in &moves {
                println!("{p_move}")
            }
            println!("{} legal moves", moves.len());
        }
    }
}
