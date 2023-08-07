use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_board::{Piece, PieceKind, Position, Square};
use sealion_maven::{Generator, MoveList, OpponentMoves};

pub fn piece_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("Piece Move Gen");

    let start = Position::starting();
    let sq = Square::at(4, 5).unwrap();
    let generator = Generator::new(&start);

    group.bench_function("Bishop", |b| {
        b.iter(|| {
            black_box(generator.pseudo_bishop_moves(black_box(sq)));
        })
    });
    group.bench_function("Rook", |b| {
        b.iter(|| {
            black_box(generator.pseudo_rook_moves(black_box(sq)));
        })
    });
    group.bench_function("Knight", |b| {
        b.iter(|| {
            black_box(generator.pseudo_knight_moves(black_box(sq)));
        })
    });
    group.bench_function("Pawn", |b| {
        b.iter(|| {
            black_box(generator.pseudo_pawn_moves(black_box(sq)));
        })
    });
}

pub fn opp_moves(c: &mut Criterion) {
    let start = Position::starting();
    let king_bb = start.board.get_piece_bb(Piece {
        color: start.active_color,
        kind: PieceKind::King,
    });

    c.bench_function("Opponent Move Gen", |b| {
        b.iter(|| {
            black_box(OpponentMoves::generate(
                black_box(&start),
                black_box(king_bb),
            ));
        })
    });
}

pub fn move_gen(c: &mut Criterion) {
    let mut group = c.benchmark_group("Move Gen");

    for (name, pos) in MOVE_GEN_POSITIONS {
        let position = sealion_fen::from_str(pos).unwrap();

        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(MoveList::generate(black_box(&position)));
            })
        });
    }
}

criterion_group!(benches, piece_moves, opp_moves, move_gen);
criterion_main!(benches);

const MOVE_GEN_POSITIONS: [(&str, &str); 4] = [
    (
        "StartPos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "Noisy",
        "R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1",
    ),
    (
        "Rand 1",
        "rn2kbn1/p1pp1r2/3bp2q/1p3pp1/3P1PNp/1NQ5/PPP1PRPP/R1B1KB2 b Qq - 0 2",
    ),
    (
        "Rand 2",
        "r4rk1/pQ1nppbp/2p1b1p1/8/3q2n1/2N1N1P1/PP2PPBP/R1B2RK1 b - - 2 9",
    ),
];
