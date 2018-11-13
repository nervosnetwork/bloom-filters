use std::hash::{BuildHasher, Hash, Hasher};

// Calculates the optimal number of hash functions to use for a Bloom
// filter based on the desired rate of false positives.
pub fn compute_k_num(fp_rate: f64) -> usize {
    assert!(fp_rate > 0.0 && fp_rate < 1.0);
    fp_rate.log2().abs().ceil() as usize
}

pub trait HashKernals {
    type HI: Iterator<Item = usize>;
    fn hash_iter<T: Hash>(&self, item: &T) -> Self::HI;
}

pub struct DoubleHashing<BH> {
    k: usize,
    n: usize,
    build_hasher: BH,
}

impl<BH: BuildHasher> DoubleHashing<BH> {
    pub fn with_fp_rate(fp_rate: f64, n: usize, build_hasher: BH) -> Self {
        Self::with_k(compute_k_num(fp_rate), n, build_hasher)
    }

    pub fn with_k(k: usize, n: usize, build_hasher: BH) -> Self {
        Self { k, n, build_hasher }
    }
}

impl<BH: BuildHasher> HashKernals for DoubleHashing<BH> {
    type HI = DoubleHashingIter;

    fn hash_iter<T: Hash>(&self, item: &T) -> Self::HI {
        let hasher = &mut self.build_hasher.build_hasher();
        item.hash(hasher);
        let result = hasher.finish();

        DoubleHashingIter {
            h1: (result as u32) as usize,
            h2: (result >> 32) as usize,
            k: self.k,
            n: self.n,
            counter: 0,
        }
    }
}

pub struct DoubleHashingIter {
    h1: usize,
    h2: usize,
    k: usize,
    n: usize,
    counter: usize,
}

impl Iterator for DoubleHashingIter {
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
