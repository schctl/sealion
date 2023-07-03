use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_board::{Position, Square};
use sealion_maven::Maven;

pub fn sliding_moves(c: &mut Criterion) {
    c.bench_function("Sliding Move Generation", |b| {
        let start = Position::starting();
        let sq = Square::at(4, 5).unwrap();

        b.iter(|| {
            black_box(Maven::bishop_moves(black_box(sq), black_box(&start)));
        })
    });
}

pub fn knight_moves(c: &mut Criterion) {
    c.bench_function("Knight Move Generation", |b| {
        let start = Position::starting();
        let sq = Square::at(3, 3).unwrap();

        b.iter(|| {
            black_box(Maven::knight_moves(black_box(sq), black_box(&start)));
        })
    });
}

pub fn pawn_moves(c: &mut Criterion) {
    c.bench_function("Pawn Move Generation", |b| {
        let start =
            sealion_fen::de::parse("rnbqkbnr/pppp1ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1")
                .unwrap()
                .1;
        let sq = Square::at(4, 3).unwrap();

        b.iter(|| {
            black_box(Maven::pawn_moves(black_box(sq), black_box(&start)));
        })
    });
}

criterion_group!(benches, sliding_moves, knight_moves, pawn_moves);
criterion_main!(benches);
