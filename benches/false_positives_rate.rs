extern crate bloom_filters;
extern crate rand;
#[macro_use]
extern crate criterion;

use bloom_filters::{BloomFilter, ClassicBloomFilter, StableBloomFilter};
use criterion::{Criterion, Fun};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use std::collections::hash_map::RandomState;

// This is an empty bench, only print false positives rate
fn bench(c: &mut Criterion) {
    let false_positives: usize = (0..1000)
        .map(|_| {
            let mut filter = ClassicBloomFilter::new(100, 0.03, RandomState::new());
            let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(100).collect();
            items.iter().for_each(|i| filter.insert(i));
            let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(100).collect();
            items.iter().filter(|i| filter.contains(i)).count()
        })
        .sum();
    println!("ClassicBloomFilter false positives: {:?}", false_positives as f32 / 100000.0);

    let mut filter = StableBloomFilter::new(100, 2, 0.03, RandomState::new());
    let false_positives: usize = (0..100000)
        .filter(|_| {
            let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(2).collect();
            filter.insert(&items[0]);
            filter.contains(&items[1])
        })
        .count();

    println!("StableBloomFilter false_positives: {:?}", false_positives as f32 / 100000.0);

    let classic = Fun::new("classic", |b, _| b.iter(|| {}));

    let stable = Fun::new("stable", |b, _| b.iter(|| {}));
    let functions = vec![classic, stable];
    c.bench_functions("false_positives_rate", functions, ());
}

criterion_group!(benches, bench);
criterion_main!(benches);
