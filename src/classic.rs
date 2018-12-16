use crate::buckets::Buckets;
use crate::{BloomFilter, DefaultHashKernals, HashKernals};
use std::hash::{BuildHasher, Hash};

pub struct Filter<BH> {
    buckets: Buckets,                     // filter data
    hash_kernals: DefaultHashKernals<BH>, // a hash function builder
}

impl<BH: BuildHasher> Filter<BH> {
    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// fp_rate is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new(items_count: usize, fp_rate: f64, build_hasher: BH) -> Self {
        let buckets = Buckets::with_fp_rate(items_count, fp_rate, 1);
        let hash_kernals = DefaultHashKernals::with_fp_rate(fp_rate, buckets.len(), build_hasher);
        Self { buckets, hash_kernals }
    }

    pub fn with_raw_data(raw_data: &[u8], k: usize, build_hasher: BH) -> Self {
        let buckets = Buckets::with_raw_data(raw_data.len() * 8, 1, raw_data);
        let hash_kernals = DefaultHashKernals::with_k(k, buckets.len(), build_hasher);
        Self { buckets, hash_kernals }
    }

    pub fn buckets(&self) -> &Buckets {
        &self.buckets
    }
}

impl<BH: BuildHasher> BloomFilter for Filter<BH> {
    fn insert<T: Hash>(&mut self, item: &T) {
        self.hash_kernals.hash_iter(item).for_each(|i| self.buckets.set(i, 1))
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        self.hash_kernals.hash_iter(item).all(|i| self.buckets.get(i) == 1)
    }

    fn reset(&mut self) {
        self.buckets.reset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::DefaultBuildHasher;
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

    #[test]
    fn raw_data() {
        let data = vec![0; 8];
        let mut filter = Filter::with_raw_data(&data, 2, DefaultBuildHasher);
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(8).collect();
        assert!(items.iter().all(|i| !filter.contains(i)));
        items.iter().for_each(|i| filter.insert(i));

        let data = filter.buckets().raw_data();
        let filter = Filter::with_raw_data(&data, 2, DefaultBuildHasher);
        assert!(items.iter().all(|i| filter.contains(i)));
    }
}
