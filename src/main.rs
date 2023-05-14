use std::io::stdin;

fn main() {
    let mut fen = String::new();
    let stdin = stdin();

    loop {
        fen.clear();
        stdin.read_line(&mut fen).unwrap();

        let (_, position) = sealion_fen::de::parse(&fen).unwrap();
        println!("{}", position.board);
    }
}
