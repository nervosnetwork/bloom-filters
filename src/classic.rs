use buckets::Buckets;
use std::hash::{BuildHasher, Hash};
use {compute_k_num, compute_m_num, hash_kernals, BloomFilter};

pub struct Filter<BH> {
    buckets: Buckets, // filter data
    build_hasher: BH, // a hash function builder
    k: usize,         // number of hash functions
}

impl<BH: BuildHasher> Filter<BH> {
    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// fp_rate is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new(items_count: usize, fp_rate: f64, build_hasher: BH) -> Self {
        Self {
            buckets: Buckets::new(compute_m_num(items_count, fp_rate), 1),
            build_hasher,
            k: compute_k_num(fp_rate),
        }
    }
}

impl<BH: BuildHasher> BloomFilter for Filter<BH> {
    fn insert<T: Hash>(&mut self, item: &T) {
        let (lo, hi) = hash_kernals(item, &mut self.build_hasher.build_hasher());
        let (lo, hi) = (lo as usize, hi as usize);
        (0..self.k).for_each(|i| {
            let offset = lo.wrapping_add(hi.wrapping_mul(i)) % self.buckets.len();
            self.buckets.set(offset, 1);
        })
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        let (lo, hi) = hash_kernals(item, &mut self.build_hasher.build_hasher());
        let (lo, hi) = (lo as usize, hi as usize);
        (0..self.k).all(|i| {
            let offset = lo.wrapping_add(hi.wrapping_mul(i)) % self.buckets.len();
            self.buckets.get(offset) == 1
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
        let mut filter = Filter::new(100, 0.03, RandomState::new());
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(16).collect();
        assert!(items.iter().all(|i| !filter.contains(i)));
        items.iter().for_each(|i| filter.insert(i));
        assert!(items.iter().all(|i| filter.contains(i)));
    }
}
