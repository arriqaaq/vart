use std::collections::BTreeMap;
use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::prelude::SliceRandom;
use rand::SeedableRng;
use rand::{rngs::StdRng, Rng};

use vart::art::Tree;
use vart::{FixedSizeKey, VariableSizeKey};

fn seeded_rng(alter: u64) -> impl Rng {
    StdRng::seed_from_u64(0xEA3C47920F94A980 ^ alter)
}

pub fn seq_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_insert");
    group.throughput(Throughput::Elements(1));
    group.bench_function("vart", |b| {
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        let mut key = 0u64;
        b.iter(|| {
            let _ = tree.insert(&key.into(), key, 0, 0);
            key += 1;
        })
    });

    group.finish();
}

pub fn seq_insert_mut(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_insert_mut");
    group.throughput(Throughput::Elements(1));
    group.bench_function("vart", |b| {
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        let mut key = 0u64;
        b.iter(|| {
            let _ = tree.insert_unchecked(&key.into(), key, 0, 0);
            key += 1;
        })
    });

    // Benchmark for BTreeMap
    group.bench_function("btreemap", |b| {
        let mut btree = BTreeMap::new();
        let mut key = 0u64;
        b.iter(|| {
            btree.insert(key, key);
            key += 1;
        })
    });

    group.finish();
}

pub fn rand_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("rand_insert");
    group.throughput(Throughput::Elements(1));

    let keys = gen_keys(3, 2, 3);

    group.bench_function("vart", |b| {
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        let mut rng = seeded_rng(0xE080D1A42C207DAF);
        b.iter(|| {
            let key = &keys[rng.gen_range(0..keys.len())];
            let _ = tree.insert(&key.into(), key.clone(), 0, 0);
        })
    });

    group.finish();
}

pub fn rand_insert_mut(c: &mut Criterion) {
    let mut group = c.benchmark_group("rand_insert_mut");
    group.throughput(Throughput::Elements(1));

    let keys = gen_keys(3, 2, 3);

    group.bench_function("vart", |b| {
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        let mut rng = seeded_rng(0xE080D1A42C207DAF);
        b.iter(|| {
            let key = &keys[rng.gen_range(0..keys.len())];
            let _ = tree.insert_unchecked(&key.into(), key.clone(), 0, 0);
        })
    });

    // Benchmark for BTreeMap
    group.bench_function("btreemap", |b| {
        let mut btree = BTreeMap::new();
        let mut rng = seeded_rng(0xE080D1A42C207DAF);
        b.iter(|| {
            let key = &keys[rng.gen_range(0..keys.len())];
            btree.insert(key.clone(), key.clone());
        })
    });

    group.finish();
}

pub fn seq_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_delete");
    group.throughput(Throughput::Elements(1));
    group.bench_function("vart", |b| {
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        b.iter_custom(|iters| {
            for i in 0..iters {
                let _ = tree.insert(&i.into(), i, 0, 0);
            }
            let start = Instant::now();
            for i in 0..iters {
                let _ = tree.remove(&i.into());
            }
            start.elapsed()
        })
    });

    group.finish();
}

pub fn rand_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("rand_delete");
    let keys = gen_keys(3, 2, 3);

    group.throughput(Throughput::Elements(1));
    group.bench_function("vart", |b| {
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        let mut rng = seeded_rng(0xE080D1A42C207DAF);
        for key in &keys {
            let _ = tree.insert(&key.into(), key, 0, 0);
        }
        b.iter(|| {
            let key = &keys[rng.gen_range(0..keys.len())];
            let _ = criterion::black_box(tree.remove(&key.into()));
        })
    });

    group.finish();
}

pub fn rand_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_get");

    group.throughput(Throughput::Elements(1));
    {
        let size = 1_000_000;
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        for i in 0..size {
            tree.insert(&i.into(), i, 0, 0).unwrap();
        }
        group.bench_with_input(BenchmarkId::new("vart", size), &size, |b, size| {
            let mut rng = seeded_rng(0xE080D1A42C207DAF);
            b.iter(|| {
                let key: u64 = rng.gen_range(0..*size);
                let _ = criterion::black_box(tree.get(&key.into(), 0));
            })
        });
    }

    group.finish();
}

pub fn rand_get_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_get_str");
    let keys = gen_keys(3, 2, 3);

    {
        let size = 1_000_000;
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        for (i, key) in keys.iter().enumerate() {
            tree.insert(&key.into(), i, 0, 0).unwrap();
        }
        group.bench_with_input(BenchmarkId::new("vart", size), &size, |b, _size| {
            let mut rng = seeded_rng(0xE080D1A42C207DAF);
            b.iter(|| {
                let key = &keys[rng.gen_range(0..keys.len())];
                let _ = criterion::black_box(tree.get(&key.into(), 0));
            })
        });
    }

    group.finish();
}

pub fn seq_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_get");

    group.throughput(Throughput::Elements(1));
    {
        let size = 1_000_000;
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        for i in 0..size as u64 {
            tree.insert(&i.into(), i, 0, 0).unwrap();
        }
        group.bench_with_input(BenchmarkId::new("vart", size), &size, |b, _size| {
            let mut key = 0u64;
            b.iter(|| {
                let _ = criterion::black_box(tree.get(&key.into(), 0));
                key += 1;
            })
        });
    }

    group.finish();
}

pub fn iter_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_benchmark");

    group.throughput(Throughput::Elements(1));
    {
        let size = 1_000_000;
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        for i in 0..size as u64 {
            tree.insert(&i.into(), i, 0, 0).unwrap();
        }
        group.bench_with_input(BenchmarkId::new("vart", size), &size, |b, _size| {
            b.iter(|| {
                let count = criterion::black_box(tree.iter()).count();
                assert_eq!(
                    count, size as usize,
                    "Not all items are present in the tree"
                );
            })
        });
    }

    group.finish();
}

pub fn range_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_benchmark");

    group.throughput(Throughput::Elements(1));
    {
        let size = 1_000_000;
        let mut tree = Tree::<FixedSizeKey<16>, _>::new();
        for i in 0..size as u64 {
            tree.insert(&i.into(), i, 0, 0).unwrap();
        }
        group.bench_with_input(BenchmarkId::new("vart", size), &size, |b, _size| {
            let start_key: FixedSizeKey<16> = 0u16.into();
            let end_key: FixedSizeKey<16> = ((size - 1) as u16).into();
            b.iter(|| {
                let count = criterion::black_box(tree.range(&start_key..=&end_key)).count();
                assert_eq!(
                    count, size as usize,
                    "Not all items are present in the tree"
                );
            })
        });
    }

    group.finish();
}

fn gen_keys(l1_prefix: usize, l2_prefix: usize, suffix: usize) -> Vec<String> {
    let mut keys = Vec::new();
    let chars: Vec<char> = ('a'..='z').collect();
    let mut rng = seeded_rng(0x740A11E72FDC215D);
    for i in 0..chars.len() {
        let level1_prefix = chars[i].to_string().repeat(l1_prefix);
        for i in 0..chars.len() {
            let level2_prefix = chars[i].to_string().repeat(l2_prefix);
            let key_prefix = level1_prefix.clone() + &level2_prefix;
            for _ in 0..=u8::MAX {
                let suffix: String = (0..suffix)
                    .map(|_| chars[rng.gen_range(0..chars.len())])
                    .collect();
                let k = key_prefix.clone() + &suffix;
                keys.push(k);
            }
        }
    }

    keys.shuffle(&mut rng);
    keys
}

fn generate_data(
    num_keys: usize,
    key_size: usize,
    value_size: usize,
) -> Vec<(VariableSizeKey, Vec<u32>)> {
    let mut data = Vec::with_capacity(num_keys);

    for i in 0..num_keys {
        // Generate key
        let mut key = vec![0xFF; key_size];
        key[0..8.min(key_size)].copy_from_slice(&i.to_le_bytes()[0..8.min(key_size)]);

        // Generate value
        let value = vec![0x42; value_size];

        data.push((VariableSizeKey::from_slice(&key), value));
    }

    data
}

fn variable_size_bulk_insert_mut(c: &mut Criterion) {
    let mut group = c.benchmark_group("vart_insert");

    // Test different combinations of key sizes, value sizes, and number of keys
    let key_sizes = vec![16, 32, 64, 128];

    let num_keys_list = vec![100_000, 500_000, 1_000_000];

    for &num_keys in &num_keys_list {
        for &key_size in &key_sizes {
            let benchmark_id = BenchmarkId::new(format!("k{}_v{}", key_size, key_size), num_keys);

            group.bench_with_input(benchmark_id, &num_keys, |b, &num_keys| {
                // Generate test data outside the benchmark loop
                let data = generate_data(num_keys, key_size, key_size);

                b.iter(|| {
                    let mut tree = Tree::new();
                    for (key, value) in data.iter() {
                        tree.insert_unchecked(key, value, 0, 0)
                            .expect("insert should succeed");
                    }
                    black_box(tree)
                });
            });
        }
    }

    group.finish();
}

criterion_group!(delete_benches, seq_delete, rand_delete);
criterion_group!(
    insert_benches,
    seq_insert,
    seq_insert_mut,
    rand_insert,
    rand_insert_mut,
    variable_size_bulk_insert_mut
);
criterion_group!(read_benches, seq_get, rand_get, rand_get_str);
criterion_group!(iter_benches, iter_benchmark);
criterion_group!(range_benches, range_benchmark);
criterion_main!(
    insert_benches,
    read_benches,
    delete_benches,
    iter_benches,
    range_benches
);
