use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_fen;

pub fn benchmark_de(c: &mut Criterion) {
    let mut group = c.benchmark_group("fen_de");

    for (name, pos) in FEN_DE_POSITIONS {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(sealion_fen::from_str(black_box(pos)));
            })
        });
    }
}

criterion_group!(benches, benchmark_de);
criterion_main!(benches);

const FEN_DE_POSITIONS: [(&str, &str); 4] = [
    (
        "start_pos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "noisy",
        "1rb1kb1r/p1p1P1pp/1q1p1p2/1p1nN1n1/2BP1B1N/1Q2p3/PPP1P1PP/R4RK1 w Qk e6 0 1",
    ),
];
