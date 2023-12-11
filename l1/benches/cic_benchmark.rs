//! Benchmark CIC filter.
//! Based on example from
//! https://bheisler.github.io/criterion.rs/book/getting_started.html

use criterion::{criterion_group, criterion_main, Criterion};
use l1::dsp::cic;

pub fn criterion_benchmark(c: &mut Criterion) {
    let sinetable = cic::make_sinetable(1000);
    let ratio: usize = 250;
    let mut buf = vec![num::zero(); ratio];

    let mut ddc = cic::CicDdc::<4>::new(sinetable.clone(), 10);
    c.bench_function("CIC DDC", |b| b.iter(|| {
        ddc.process(&buf[..]);
    }));

    let mut duc = cic::CicDuc::<4>::new(sinetable.clone(), 10);
    c.bench_function("CIC DUC", |b| b.iter(|| {
        duc.process(cic::BufferType { re: 1, im: 2 }, &mut buf[..]);
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
