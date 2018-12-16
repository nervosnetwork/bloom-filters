use crate::buckets::Buckets;
use crate::hash::compute_k_num;
use crate::{BloomFilter, DefaultHashKernals, HashKernals};
use rand::random;
use std::hash::{BuildHasher, Hash};

pub struct Filter<BH> {
    buckets: Buckets,                     // filter data
    hash_kernals: DefaultHashKernals<BH>, // a hash function builder
    p: usize,                             // number of buckets to decrement,
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

        let buckets = Buckets::new(m, d);
        let hash_kernals = DefaultHashKernals::with_k(k, buckets.len(), build_hasher);

        Self {
            buckets,
            hash_kernals,
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
        let max = self.buckets.max_value();
        self.hash_kernals.hash_iter(item).for_each(|i| self.buckets.set(i, max))
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        self.hash_kernals.hash_iter(item).all(|i| self.buckets.get(i) > 0)
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
