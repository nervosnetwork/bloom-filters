use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasher, Hash, Hasher};

// Calculates the optimal number of hash functions to use for a Bloom
// filter based on the desired rate of false positives.
pub fn compute_k_num(fp_rate: f64) -> usize {
    assert!(fp_rate > 0.0 && fp_rate < 1.0);
    fp_rate.log2().abs().ceil() as usize
}

pub trait HashKernals<H> {
    type HI: Iterator<Item = usize>;
    fn hash_iter(&self, item: &H) -> Self::HI;
}

pub struct DefaultHashKernals<BH> {
    k: usize,
    n: usize,
    build_hasher: BH,
}

impl<BH: BuildHasher> DefaultHashKernals<BH> {
    pub fn with_fp_rate(fp_rate: f64, n: usize, build_hasher: BH) -> Self {
        Self::with_k(compute_k_num(fp_rate), n, build_hasher)
    }

    pub fn with_k(k: usize, n: usize, build_hasher: BH) -> Self {
        Self { k, n, build_hasher }
    }
}

impl<H: Hash, BH: BuildHasher> HashKernals<H> for DefaultHashKernals<BH> {
    type HI = DefaultHashIter;

    fn hash_iter(&self, item: &H) -> Self::HI {
        let hasher = &mut self.build_hasher.build_hasher();
        item.hash(hasher);
        let result = hasher.finish();

        DefaultHashIter::new(result, self.k, self.n)
    }
}

pub struct DefaultHashIter {
    h1: usize,
    h2: usize,
    k: usize,
    n: usize,
    counter: usize,
}

impl DefaultHashIter {
    fn new(hash: u64, k: usize, n: usize) -> Self {
        Self {
            h1: (hash as u32) as usize,
            h2: (hash >> 32) as usize,
            k,
            n,
            counter: 0,
        }
    }
}

impl Iterator for DefaultHashIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.k == self.counter {
            return None;
        }
        let r = self.h1.wrapping_add(self.h2.wrapping_mul(self.counter)) % self.n;
        self.counter += 1;
        Some(r)
    }
}

pub struct DefaultBuildHasher;

impl BuildHasher for DefaultBuildHasher {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher::new()
    }
}
