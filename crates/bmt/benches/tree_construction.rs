//! Benchmark for Merkle tree construction.
use criterion::{
    BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
    measurement::WallTime,
};
use digest::{Digest, FixedOutputReset, Output};
use sha2::Sha256;
use sha3::Keccak256;
use std::hint::black_box;
use uts_bmt::UnorderdMerkleTree;

const INPUT_SIZES: &[usize] = &[8, 1024, 65536, 1_048_576];

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_construction");
    bench_digest::<Sha256>(&mut group, "uts-bmt/Sha256");
    bench_digest::<Keccak256>(&mut group, "uts-bmt/Keccak256");
    bench_commonware::<commonware_cryptography::Sha256>(
        &mut group,
        "commonware_storage::bmt/Sha256",
    );
    group.finish();
}

fn bench_digest<D>(group: &mut BenchmarkGroup<'_, WallTime>, id: &str)
where
    D: Digest + FixedOutputReset,
    Output<D>: Copy,
{
    for &size in INPUT_SIZES {
        let leaves = generate_leaves::<D>(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new(id, size), move |b| {
            // Tree construction is the operation under test.
            b.iter(|| {
                let tree = UnorderdMerkleTree::<D>::new(black_box(leaves.as_slice()));
                black_box(tree);
            });
        });
    }
}

fn bench_commonware<H: commonware_cryptography::Hasher>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    id: &str,
) {
    use commonware_storage::bmt::Builder;

    for &size in INPUT_SIZES {
        let leaves: Vec<H::Digest> = generate_commonware_leaves::<H>(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new(id, size), move |b| {
            b.iter_batched(
                || {
                    let mut builder = Builder::<H>::new(leaves.len());
                    for digest in leaves.iter() {
                        builder.add(black_box(digest));
                    }
                    builder
                },
                |builder| {
                    let tree = builder.build();
                    black_box(tree);
                },
                BatchSize::SmallInput,
            );
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

fn generate_commonware_leaves<H: commonware_cryptography::Hasher>(count: usize) -> Vec<H::Digest> {
    (0..count)
        .map(|i| {
            let input = i.to_le_bytes();
            H::hash(&input)
        })
        .collect()
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
