use buckets::Buckets;
use rand::random;
use std::hash::{BuildHasher, Hash};
use {compute_k_num, hash_kernals, BloomFilter};

pub struct Filter<BH> {
    buckets: Buckets, // filter data
    build_hasher: BH, // a hash function builder
    k: usize,         // number of hash functions
    p: usize,         // number of buckets to decrement,
}

impl<BH: BuildHasher> Filter<BH> {
    /// Creates a new Stable Bloom Filter with m buckets and d
    /// bits allocated per bucket optimized for the target false-positive rate.
    pub fn new(m: usize, d: u8, fp_rate: f64, build_hasher: BH) -> Self {
        let mut k = compute_k_num(fp_rate);
        if k > m {
            k = m
        } else if k == 0 {
            k = 1
        }

        Self {
            buckets: Buckets::new(m, d),
            build_hasher,
            k,
            p: compute_p_num(m, k, d, fp_rate),
        }
    }

    fn decrement(&mut self) {
        let r: usize = random();
        (0..self.p).for_each(|i| {
            let bucket = (r + i) % self.buckets.len();
            self.buckets.increment(bucket, -1)
        })
    }
}

// returns the optimal number of buckets to decrement, p, per
// iteration for the provided parameters of an SBF.
fn compute_p_num(m: usize, k: usize, d: u8, fp_rate: f64) -> usize {
    let (m, k, d) = (m as f64, k as f64, f64::from(d));
    let max = 2f64.powf(d) - 1.0;
    let sub_denom = (1.0 - fp_rate.powf(1.0 / k)).powf(1.0 / max);
    let denom = (1.0 / sub_denom - 1.0) * (1.0 / k - 1.0 / m);
    let p = (1.0 / denom).ceil();
    if p <= 0.0 {
        1
    } else {
        p as usize
    }
}

impl<BH: BuildHasher> BloomFilter for Filter<BH> {
    fn insert<T: Hash>(&mut self, item: &T) {
        self.decrement();
        let (lo, hi) = hash_kernals(item, &mut self.build_hasher.build_hasher());
        let (lo, hi) = (lo as usize, hi as usize);
        let max = self.buckets.max_value();
        (0..self.k).for_each(|i| {
            let offset = lo.wrapping_add(hi.wrapping_mul(i)) % self.buckets.len();
            self.buckets.set(offset, max);
        })
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        let (lo, hi) = hash_kernals(item, &mut self.build_hasher.build_hasher());
        let (lo, hi) = (lo as usize, hi as usize);
        (0..self.k).all(|i| {
            let offset = lo.wrapping_add(hi.wrapping_mul(i)) % self.buckets.len();
            self.buckets.get(offset) > 0
        })
    }

    fn reset(&mut self) {
        self.buckets.reset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Standard;
    use rand::{thread_rng, Rng};
    use std::collections::hash_map::RandomState;

    #[test]
    fn contains() {
        // d = 3, max = (1 << d) - 1
        let mut filter = Filter::new(100, 3, 0.03, RandomState::new());
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(7).collect();
        assert!(items.iter().all(|i| !filter.contains(i)));
        items.iter().for_each(|i| filter.insert(i));
        assert!(items.iter().all(|i| filter.contains(i)));
    }
}
