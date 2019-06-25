use criterion::{criterion_group, criterion_main};
use criterion::{Criterion, black_box as bb};

use softposit::{P32, Q32};

fn criterion_benchmark(c: &mut Criterion) {
    const X: P32 = P32::new(0x_5c80_0000); // 12.5
    const Y: P32 = P32::new(0x_6b55_5810); // 117.334
    const Z: P32 = P32::new(0x_3c2e_48e9); // 0.7613

    c.bench_function("add", |c| c.iter(|| bb(X) + bb(Y)));
    c.bench_function("sub", |c| c.iter(|| bb(X) - bb(Y)));
    c.bench_function("mul", |c| c.iter(|| bb(X) * bb(Y)));
    c.bench_function("div", |c| c.iter(|| bb(X) / bb(Y)));

    c.bench_function("sqrt", |c| c.iter(|| bb(X).sqrt()));
    c.bench_function("round", |c| c.iter(|| bb(Y).round()));

    c.bench_function("sin", |c| c.iter(|| bb(Z).sin()));
    c.bench_function("sin2", |c| c.iter(|| bb(Y).sin()));
    c.bench_function("cos", |c| c.iter(|| bb(Z).cos()));
    c.bench_function("tan", |c| c.iter(|| bb(Z).tan()));
    c.bench_function("exp", |c| c.iter(|| bb(X).exp()));
    c.bench_function("ln", |c| c.iter(|| bb(X).ln()));
    c.bench_function("pow", |c| c.iter(|| bb(X).powf(Z)));

    c.bench_function("q_add", |c| c.iter(|| {
        let mut q = Q32::init();
        q += (bb(X), bb(Y));
        q.to_posit()
    }));
    c.bench_function("q_add3", |c| c.iter(|| {
        let mut q = Q32::init();
        q += (bb(X), bb(Y));
        q += (bb(X), bb(Z));
        q += (bb(Y), bb(Z));
        q.to_posit()
    }));
    
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
