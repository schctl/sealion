use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_board::{IntoEnumIterator, Piece, PieceKind, Position, Square};
use sealion_engine::movegen::{Generator, MoveList};
use sealion_engine::state::PositionState;

pub fn piece_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("movegen_p");

    let start = Position::starting();
    let state = PositionState::generate(&start);
    let sq = Square::at(4, 5).unwrap();
    let generator = Generator::new(&state);

    for kind in PieceKind::iter() {
        group.bench_function(kind.as_char(), |b| {
            b.iter(|| {
                black_box(generator.pseudo_moves(black_box(sq), kind));
            })
        });
    }
}

pub fn move_gen(c: &mut Criterion) {
    let mut group = c.benchmark_group("movegen");

    for (name, pos) in MOVE_GEN_POSITIONS {
        let position = sealion_fen::from_str(pos).unwrap();
        let state = PositionState::generate(&position);

        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(MoveList::generate(black_box(&state)));
            })
        });
    }
}

criterion_group!(benches, piece_moves, move_gen);
criterion_main!(benches);

const MOVE_GEN_POSITIONS: [(&str, &str); 4] = [
    (
        "start_pos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "noisy",
        "R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1",
    ),
    (
        "rand_0",
        "rn2kbn1/p1pp1r2/3bp2q/1p3pp1/3P1PNp/1NQ5/PPP1PRPP/R1B1KB2 b Qq - 0 2",
    ),
    (
        "rand_1",
        "r4rk1/pQ1nppbp/2p1b1p1/8/3q2n1/2N1N1P1/PP2PPBP/R1B2RK1 b - - 2 9",
    ),
];
