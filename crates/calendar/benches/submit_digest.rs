//! Benchmark for submitting digests of varying sizes.

use alloy_primitives::b256;
use alloy_signer_local::LocalSigner;
use bytes::Bytes;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use uts_calendar::routes::ots::submit_digest_inner;

const DIGEST_SIZES: &[usize] = &[32, 64];

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("submit_digest");

    for &size in DIGEST_SIZES {
        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::from_parameter(size), move |b| {
            let signer = LocalSigner::from_bytes(&b256!(
                "9ba9926331eb5f4995f1e358f57ba1faab8b005b51928d2fdaea16e69a6ad225"
            ))
            .unwrap();
            let input = Bytes::from(vec![0u8; size]);

            b.iter_batched(
                || input.clone(),
                |input| {
                    let out = submit_digest_inner(input, &signer);
                    black_box(out)
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
