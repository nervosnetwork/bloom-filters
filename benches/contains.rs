extern crate bloom_filters;
extern crate rand;
#[macro_use]
extern crate criterion;

use bloom_filters::{BloomFilter, ClassicBloomFilter, StableBloomFilter};
use criterion::{Criterion, Fun};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::collections::hash_map::RandomState;

fn bench(c: &mut Criterion) {
    let classic = Fun::new("classic", |b, fp_rate| {
        let mut filter = ClassicBloomFilter::new(100, *fp_rate, RandomState::new());
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
        items.iter().for_each(|i| filter.insert(i));
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
        b.iter(|| {
            items.iter().for_each(|i| {
                filter.contains(i);
            })
        })
    });

    let stable = Fun::new("stable", |b, fp_rate| {
        let mut filter = StableBloomFilter::new(10, 3, *fp_rate, RandomState::new());
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
        items.iter().for_each(|i| filter.insert(i));
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
        b.iter(|| {
            items.iter().for_each(|i| {
                filter.contains(i);
            })
        })
    });
    let functions = vec![classic, stable];
    c.bench_functions("contains", functions, 0.03);
}

criterion_group!(benches, bench);
criterion_main!(benches);
