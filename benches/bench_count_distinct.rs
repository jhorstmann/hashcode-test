use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use hashbrown::raw::RawTable;
use hashcode_test::{
    fold_bits, generate_random_floats, generate_random_strings, hash_bits_original,
    hash_bits_variant, hash_key_generic, Arena, ArrowStringArray, RandomStringMode,
};
use std::hash::{BuildHasher, Hash, Hasher, RandomState};

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[inline(never)]
fn count_distinct_strings<H: Fn(&[u8]) -> u64>(strings: &ArrowStringArray, hash_fn: H) -> usize {
    // let mut arena = Arena::with_capacity(1024);
    let mut table = RawTable::<usize>::with_capacity(16 * 1024);

    for (i, s) in strings.iter().enumerate() {
        let rehash_fn = |index: &usize| hash_fn(strings.get(*index).as_bytes());
        let eq_fn = |index: &usize| s == strings.get(*index);

        let h = hash_fn(s.as_bytes());
        if let Err(slot) = table.find_or_find_insert_slot(h, eq_fn, rehash_fn) {
            // Safety: table was not modified since finding the slot
            unsafe {
                table.insert_in_slot(h, slot, i);
            }
        }
    }
    table.len()
}

#[inline(never)]
fn count_distinct_floats<H: Fn(f64) -> u64>(floats: &[f64], hash_fn: H) -> usize {
    let mut table = RawTable::<f64>::with_capacity(16 * 1024);

    for f in floats.iter().copied() {
        let eq_fn = |other: &f64| other.to_bits() == f.to_bits();

        let h = hash_fn(f);
        if let Err(slot) = table.find_or_find_insert_slot(h, eq_fn, |f| hash_fn(*f)) {
            // Safety: table was not modified since finding the slot
            unsafe {
                table.insert_in_slot(h, slot, f);
            }
        }
    }
    table.len()
}

#[inline]
fn default_string_hash(random_state: &RandomState, bytes: &[u8]) -> u64 {
    let mut hasher = random_state.build_hasher();
    bytes.hash(&mut hasher);
    hasher.finish()
}

#[inline]
fn default_u64_hash(random_state: &RandomState, bits: u64) -> u64 {
    let mut hasher = random_state.build_hasher();
    bits.hash(&mut hasher);
    hasher.finish()
}

pub fn bench_count_distinct(c: &mut Criterion) {
    let strings =
        generate_random_strings(100_000, 4, "0123456789AB", RandomStringMode::Numeric, 1234);
    let floats = generate_random_floats(100_000, 0, 100_000, 100, 1234);
    c.benchmark_group("count_distinct_strings")
        .throughput(Throughput::Elements(strings.len() as u64))
        .bench_function("default", |b| {
            let random_state = RandomState::default();
            b.iter(|| {
                count_distinct_strings(&strings, |bytes| default_string_hash(&random_state, bytes))
            })
        })
        .bench_function("fx_original", |b| {
            b.iter(|| {
                count_distinct_strings(&strings, |bytes| {
                    hash_key_generic(bytes, hash_bits_original)
                })
            })
        })
        .bench_function("fx_variant", |b| {
            b.iter(|| {
                count_distinct_strings(&strings, |bytes| hash_key_generic(bytes, hash_bits_variant))
            })
        })
        .bench_function("fx_variant_folded", |b| {
            b.iter(|| {
                count_distinct_strings(&strings, |bytes| {
                    fold_bits(hash_key_generic(bytes, hash_bits_variant))
                })
            })
        });

    c.benchmark_group("count_distinct_floats")
        .throughput(Throughput::Elements(strings.len() as u64))
        .bench_function("identity", |b| {
            b.iter(|| count_distinct_floats(&floats, |f| f.to_bits()))
        })
        .bench_function("default", |b| {
            let random_state = RandomState::default();
            b.iter(|| {
                count_distinct_floats(&floats, |f| default_u64_hash(&random_state, f.to_bits()))
            })
        })
        .bench_function("fx_original_le", |b| {
            b.iter(|| {
                count_distinct_floats(&floats, |f| {
                    hash_key_generic(&f.to_le_bytes(), hash_bits_original)
                })
            })
        })
        .bench_function("fx_original_be", |b| {
            b.iter(|| {
                count_distinct_floats(&floats, |f| {
                    hash_key_generic(&f.to_be_bytes(), hash_bits_original)
                })
            })
        })
        .bench_function("fx_variant_le", |b| {
            b.iter(|| {
                count_distinct_floats(&floats, |f| {
                    hash_key_generic(&f.to_le_bytes(), hash_bits_variant)
                })
            })
        })
        .bench_function("fx_variant_be", |b| {
            b.iter(|| {
                count_distinct_floats(&floats, |f| {
                    hash_key_generic(&f.to_be_bytes(), hash_bits_variant)
                })
            })
        })
        .bench_function("fx_variant_le_folded", |b| {
            b.iter(|| {
                count_distinct_floats(&floats, |f| {
                    fold_bits(hash_key_generic(&f.to_le_bytes(), hash_bits_variant))
                })
            })
        })
        .bench_function("fx_variant_be_folded", |b| {
            b.iter(|| {
                count_distinct_floats(&floats, |f| {
                    fold_bits(hash_key_generic(&f.to_be_bytes(), hash_bits_variant))
                })
            })
        });
}

criterion_group!(benches, bench_count_distinct);
criterion_main!(benches);
