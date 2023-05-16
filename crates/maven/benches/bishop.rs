#![feature(portable_simd)]

use std::simd::u64x4;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_board::{Position, Square};
use sealion_maven::{partial_diagonal_shadow_mask, Maven, chunk_diagonal_shadow_mask,};

pub fn benchmark_bishop(c: &mut Criterion) {
    c.bench_function("Bench Generate Bishop Moves", |b| {
        let start = Position::starting();
        let sq = Square::at(4, 5).unwrap();

        b.iter(|| {
            black_box(Maven::bishop_moves(
                black_box(sq),
                black_box(start.board.get_color_bb(start.active_color)),
                black_box(start.board.get_color_bb(start.active_color.opposite())),
            ));
        })
    });
}

pub fn benchmark_shadow(c: &mut Criterion) {
    c.bench_function("Shadow Mask", |b| {
        b.iter(|| {
            black_box(Maven::diagonal_shadow_or(black_box(u64x4::splat(5))));
        })
    });

    c.bench_function("Partial Single Shadow Mask", |b| {
        b.iter(|| {
            black_box(partial_diagonal_shadow_mask(
                black_box(2),
                black_box(83727832),
                black_box(33),
            ));
        })
    });

    c.bench_function("Single Shadow Mask 8", |b| {
        b.iter(|| {
            black_box(chunk_diagonal_shadow_mask(
                black_box(u64x4::splat(32)),
                black_box(32),
            ));
        })
    });
}

criterion_group!(benches, benchmark_bishop, benchmark_shadow);
criterion_main!(benches);
