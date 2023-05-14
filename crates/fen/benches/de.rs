use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_fen;

pub fn benchmark_de(c: &mut Criterion) {
    c.bench_function("De Starting FEN", |b| {
        b.iter(|| {
            sealion_fen::de::parse(black_box(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            ))
        })
    });

    c.bench_function("De Random FEN", |b| {
        b.iter(|| {
            sealion_fen::de::parse(black_box(
                "1rb1kb1r/p1p1P1pp/1q1p1p2/1p1nN1n1/2BP1B1N/1Q2p3/PPP1P1PP/R4RK1 w Qk e6 0 1",
            ))
        })
    });
}

criterion_group!(benches, benchmark_de);
criterion_main!(benches);
