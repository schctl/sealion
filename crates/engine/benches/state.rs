use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_board::Position;
use sealion_engine::state::PositionState;

pub fn pos_ext(c: &mut Criterion) {
    let mut group = c.benchmark_group("PositionState");

    group.bench_function("StartPos", |b| {
        let start = Position::starting();
        b.iter(|| black_box(PositionState::generate(black_box(&start))));
    });
}

criterion_group!(benches, pos_ext);
criterion_main!(benches);
