use core::alloc::Layout;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use verus_bumpalo::Bump;
use vstd::prelude::Tracked;

const N: usize = 10_000;

fn bump(c: &mut Criterion) {
    for (name, layout) in [
        ("small", Layout::new::<u8>()),
        ("big", Layout::new::<[usize; 32]>()),
    ] {
        let cap = N * layout.size().max(1) * 2;
        let mut group = c.benchmark_group(format!("try-alloc/{name}"));
        group.throughput(Throughput::Elements(N as u64));

        group.bench_function("verus-bumpalo", |b| {
            let (mut arena, _st) = Bump::try_with_capacity(cap).unwrap();
            b.iter(|| {
                for _ in 0..N {
                    let (p, _, _) = arena
                        .try_alloc_layout(black_box(layout), Tracked::assume_new())
                        .unwrap();
                    black_box(p);
                }
                arena.reset(Tracked::assume_new(), Tracked::assume_new());
            });
            arena.free(Tracked::assume_new(), Tracked::assume_new());
        });

        group.bench_function("bumpalo", |b| {
            let mut arena = bumpalo::Bump::with_capacity(cap);
            b.iter(|| {
                for _ in 0..N {
                    let p = arena.try_alloc_layout(black_box(layout)).unwrap();
                    black_box(p.as_ptr());
                }
                arena.reset();
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bump);
criterion_main!(benches);
