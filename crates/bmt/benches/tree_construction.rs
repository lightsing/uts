//! Benchmark for Merkle tree construction using different hash algorithms.
use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
    measurement::WallTime,
};
use digest::{Digest, Output};
use sha2::Sha256;
use sha3::Keccak256;
use std::hint::black_box;
use uts_bmt::FlatMerkleTree;

const INPUT_SIZES: &[usize] = &[1, 8, 64, 512, 4096, 8192, 16384, 32768, 65536];

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_construction");
    bench_digest::<Sha256>(&mut group, "sha2::Sha256");
    bench_digest::<Keccak256>(&mut group, "sha3::Keccak256");
    group.finish();
}

fn bench_digest<D>(group: &mut BenchmarkGroup<'_, WallTime>, algorithm: &str)
where
    D: Digest,
    Output<D>: Copy,
{
    for &size in INPUT_SIZES {
        let leaves = generate_leaves::<D>(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new(algorithm, size), move |b| {
            // Tree construction is the operation under test.
            b.iter(|| {
                let tree = FlatMerkleTree::<D>::new(black_box(leaves.as_slice()));
                black_box(tree);
            });
        });
    }
}

fn generate_leaves<D>(count: usize) -> Vec<Output<D>>
where
    D: Digest,
    Output<D>: Copy,
{
    (0..count)
        .map(|i| {
            let mut hasher = D::new();
            hasher.update(i.to_le_bytes());
            hasher.finalize()
        })
        .collect()
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
