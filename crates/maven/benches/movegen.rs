use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_board::{Position, Square};
use sealion_maven::{MoveList, OpponentMoves};

pub fn sliding_moves(c: &mut Criterion) {
    c.bench_function("Sliding Move Generation", |b| {
        let start = Position::starting();
        let sq = Square::at(4, 5).unwrap();

        b.iter(|| {
            black_box(MoveList::pseudo_bishop_moves(
                black_box(sq),
                black_box(&start),
            ));
        })
    });
}

pub fn knight_moves(c: &mut Criterion) {
    c.bench_function("Knight Move Generation", |b| {
        let start = Position::starting();
        let sq = Square::at(3, 3).unwrap();

        b.iter(|| {
            black_box(MoveList::pseudo_knight_moves(
                black_box(sq),
                black_box(&start),
            ));
        })
    });
}

pub fn pawn_moves(c: &mut Criterion) {
    c.bench_function("Pawn Move Generation", |b| {
        let start = Position::starting();
        let sq = Square::at(4, 3).unwrap();

        b.iter(|| {
            black_box(MoveList::pseudo_pawn_moves(
                black_box(sq),
                black_box(&start),
            ));
        })
    });
}

pub fn opp_moves(c: &mut Criterion) {
    c.bench_function("Opponent Move Generation", |b| {
        let start = Position::starting();

        b.iter(|| {
            black_box(OpponentMoves::generate(black_box(&start)));
        })
    });
}

pub fn move_gen(c: &mut Criterion) {
    c.bench_function("Full Move Generation", |b| {
        let start = Position::starting();

        b.iter(|| {
            black_box(MoveList::generate(black_box(&start)));
        })
    });

    c.bench_function("Full Move Generation Noisy", |b| {
        let start = sealion_fen::de::parse("R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1")
            .unwrap()
            .1;

        b.iter(|| {
            black_box(MoveList::generate(black_box(&start)));
        })
    });
}

criterion_group!(
    benches,
    sliding_moves,
    knight_moves,
    pawn_moves,
    opp_moves,
    move_gen
);
criterion_main!(benches);
