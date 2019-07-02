use criterion::{black_box as bb, Criterion};
use criterion::{criterion_group, criterion_main};

use softposit::MathConsts;
use softposit::{P32, Q32};

fn criterion_p32(c: &mut Criterion) {
    const X: P32 = P32::new(0x_5c80_0000); // 12.5
    const Y: P32 = P32::new(0x_6b55_5810); // 117.334
    const Z: P32 = P32::new(0x_3c2e_48e9); // 0.7613

    c.bench_function("p32_add", |c| c.iter(|| bb(X) + bb(Y)));
    c.bench_function("p32_sub", |c| c.iter(|| bb(X) - bb(Y)));
    c.bench_function("p32_mul", |c| c.iter(|| bb(X) * bb(Y)));
    c.bench_function("p32_div", |c| c.iter(|| bb(X) / bb(Y)));

    c.bench_function("p32_sqrt", |c| c.iter(|| bb(X).sqrt()));
    c.bench_function("p32_round", |c| c.iter(|| bb(Y).round()));

    c.bench_function("p32_sin", |c| c.iter(|| bb(Z).sin()));
    c.bench_function("p32_sin2", |c| c.iter(|| bb(Y).sin()));
    c.bench_function("p32_cos", |c| c.iter(|| bb(Z).cos()));
    c.bench_function("p32_tan", |c| c.iter(|| bb(Z).tan()));
    c.bench_function("p32_exp", |c| c.iter(|| bb(X).exp()));
    c.bench_function("p32_ln", |c| c.iter(|| bb(X).ln()));
    c.bench_function("p32_pow", |c| c.iter(|| bb(X).powf(Z)));

    c.bench_function("q32_add_product", |c| {
        c.iter(|| {
            let mut q = Q32::PI;
            q += (bb(X), bb(Y));
        })
    });

    c.bench_function("q32_add_posit", |c| {
        c.iter(|| {
            let mut q = Q32::PI;
            q += bb(X);
        })
    });
    c.bench_function("q32_to_posit", |c| {
        c.iter(|| {
            let q = Q32::PI;
            q.to_posit()
        })
    });
}

use softposit::{P16, Q16};

fn criterion_p16(c: &mut Criterion) {
    const X: P16 = P16::new(0x_6c80); // 12.5
    const Y: P16 = P16::new(0x_79ab); // 117.334

    c.bench_function("p16_add", |c| c.iter(|| bb(X) + bb(Y)));
    c.bench_function("p16_sub", |c| c.iter(|| bb(X) - bb(Y)));
    c.bench_function("p16_mul", |c| c.iter(|| bb(X) * bb(Y)));
    c.bench_function("p16_div", |c| c.iter(|| bb(X) / bb(Y)));

    c.bench_function("p16_sqrt", |c| c.iter(|| bb(X).sqrt()));
    c.bench_function("p16_round", |c| c.iter(|| bb(Y).round()));

    c.bench_function("q16_add_product", |c| {
        c.iter(|| {
            let mut q = Q16::PI;
            q += (bb(X), bb(Y));
        })
    });

    c.bench_function("q16_add_posit", |c| {
        c.iter(|| {
            let mut q = Q16::PI;
            q += bb(X);
        })
    });
    c.bench_function("q16_to_posit", |c| {
        c.iter(|| {
            let q = Q16::PI;
            q.to_posit()
        })
    });
}

criterion_group!(benches, criterion_p32, criterion_p16);
criterion_main!(benches);
