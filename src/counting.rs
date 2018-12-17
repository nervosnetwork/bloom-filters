use crate::buckets::Buckets;
use crate::{BloomFilter, BuildHashKernals, HashKernals, RemovableBloomFilter};
use std::hash::Hash;

pub struct Filter<BHK: BuildHashKernals> {
    buckets: Buckets,      // filter data
    hash_kernals: BHK::HK, // hash kernals
}

impl<BHK: BuildHashKernals> Filter<BHK> {
    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// bucket_size is the specified number of bits
    /// fp_rate is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new(items_count: usize, bucket_size: u8, fp_rate: f64, build_hash_kernals: BHK) -> Self {
        let buckets = Buckets::with_fp_rate(items_count, fp_rate, bucket_size);
        let hash_kernals = build_hash_kernals.with_fp_rate(fp_rate, buckets.len());
        Self { buckets, hash_kernals }
    }
}

impl<BHK: BuildHashKernals> BloomFilter for Filter<BHK> {
    fn insert<T: Hash>(&mut self, item: &T) {
        self.hash_kernals.hash_iter(item).for_each(|i| self.buckets.increment(i, 1))
    }

    fn contains<T: Hash>(&self, item: &T) -> bool {
        self.hash_kernals.hash_iter(item).all(|i| self.buckets.get(i) > 0)
    }

    fn reset(&mut self) {
        self.buckets.reset()
    }
}

impl<BHK: BuildHashKernals> RemovableBloomFilter for Filter<BHK> {
    fn remove<T: Hash>(&mut self, item: &T) {
        self.hash_kernals.hash_iter(item).for_each(|i| self.buckets.increment(i, -1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::DefaultBuildHashKernals;
    use rand::distributions::Standard;
    use rand::{random, thread_rng, Rng};
    use std::collections::hash_map::RandomState;

    #[test]
    fn contains() {
        let mut filter = Filter::new(100, 4, 0.03, DefaultBuildHashKernals::new(random(), RandomState::new()));
        let items: Vec<usize> = thread_rng().sample_iter(&Standard).take(16).collect();
        assert!(items.iter().all(|i| !filter.contains(i)));
        items.iter().for_each(|i| filter.insert(i));
        assert!(items.iter().all(|i| filter.contains(i)));
    }

    #[test]
    fn remove() {
        let mut filter = Filter::new(100, 4, 0.03, DefaultBuildHashKernals::new(random(), RandomState::new()));
        let item: usize = thread_rng().gen();
        filter.insert(&item);
        filter.remove(&item);
        assert!(!filter.contains(&item));
    }
}
