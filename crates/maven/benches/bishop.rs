use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sealion_board::{Position, Square};
use sealion_maven::Maven;

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

criterion_group!(benches, benchmark_bishop);
criterion_main!(benches);
