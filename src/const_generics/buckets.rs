use std::f64::consts::LN_2;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;

type Word = u64;
const BYTES_PER_WORD: usize = size_of::<Word>();
const BITS_PER_WORD: usize = BYTES_PER_WORD * 8;

#[derive(Clone)]
/// Buckets Implemetation with const generics
/// W: the count of `Word`
pub struct ConstBuckets<const W: usize> {
    data: [Word; W],
    bucket_count: usize,
    bucket_size: u8,
    max: u8,
}

impl<const W: usize> ConstBuckets<W> {
    /// Creates a new Buckets with the provided number of buckets where
    /// each bucket is the specified number of bits.
    pub fn new(bucket_count: usize, bucket_size: u8) -> Self {
        debug_assert!(bucket_size < 8);
        Self {
            data: [0; W],
            bucket_count,
            bucket_size,
            max: (1u8 << bucket_size) - 1,
        }
    }

    pub fn with_fp_rate(items_count: usize, fp_rate: f64, bucket_size: u8) -> Self {
        Self::new(optimal_bucket_count(items_count, fp_rate), bucket_size)
    }

    pub fn with_raw_data(bucket_count: usize, bucket_size: u8, raw_data: &[u8]) -> Self {
        debug_assert!(bucket_size < 8);
        debug_assert!(W * 8 == raw_data.len());
        let data = [0; W];
        for (idx, buf) in raw_data.chunks(BYTES_PER_WORD).enumerate() {
            let d_slice = &data[idx] as *const _ as *mut u8;
            unsafe {
                copy_nonoverlapping(buf.as_ptr(), d_slice, BYTES_PER_WORD);
            }
        }
        Self {
            data,
            bucket_count,
            bucket_size,
            max: (1u8 << bucket_size) - 1,
        }
    }

    pub fn raw_data(&self) -> Vec<u8> {
        let mut result = vec![0; self.data.len() * BYTES_PER_WORD];
        for (d, chunk) in self.data.iter().zip(result.chunks_mut(BYTES_PER_WORD)) {
            unsafe {
                let bytes = *(&d.to_le() as *const _ as *const [u8; BYTES_PER_WORD]);
                copy_nonoverlapping((&bytes).as_ptr(), chunk.as_mut_ptr(), BYTES_PER_WORD);
            }
        }
        result
    }

    pub fn update(&mut self, raw_data: &[u8]) {
        self.data
            .iter_mut()
            .zip(raw_data.chunks(BYTES_PER_WORD))
            .for_each(|(word, bytes)| {
                let value = bytes.iter().enumerate().fold(*word, |acc, (offset, byte)| {
                    acc | (*byte as Word) << (offset * BYTES_PER_WORD)
                });
                *word = value;
            });
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.bucket_count
    }

    #[inline(always)]
    pub fn max_value(&self) -> u8 {
        self.max
    }

    pub fn reset(&mut self) {
        self.data.iter_mut().for_each(|x| *x = 0)
    }

    pub fn increment(&mut self, bucket: usize, delta: i8) {
        let v = (self.get(bucket) as i8).saturating_add(delta);
        let value = if v < 0 {
            0u8
        } else if v > self.max as i8 {
            self.max
        } else {
            v as u8
        };
        self.set(bucket, value);
    }

    pub fn set(&mut self, bucket: usize, byte: u8) {
        let offset = bucket * self.bucket_size as usize;
        let length = self.bucket_size as usize;
        let word = if byte > self.max as u8 { self.max } else { byte } as Word;
        self.set_word(offset, length, word);
    }

    pub fn get(&self, bucket: usize) -> u8 {
        self.get_word(bucket * self.bucket_size as usize, self.bucket_size as usize) as u8
    }

    fn set_word(&mut self, offset: usize, length: usize, word: Word) {
        let word_index = offset / BITS_PER_WORD;
        let word_offset = offset % BITS_PER_WORD;

        if word_offset + length > BITS_PER_WORD {
            let remain = BITS_PER_WORD - word_offset;
            self.set_word(offset, remain, word);
            self.set_word(offset + remain, length - remain, word >> remain);
        } else {
            let bit_mask = (1 << length) - 1;
            self.data[word_index] &= !(bit_mask << word_offset);
            self.data[word_index] |= (word & bit_mask) << word_offset;
        }
    }

    fn get_word(&self, offset: usize, length: usize) -> Word {
        let word_index = offset / BITS_PER_WORD;
        let word_offset = offset % BITS_PER_WORD;
        if word_offset + length > BITS_PER_WORD {
            let remain = BITS_PER_WORD - word_offset;
            self.get_word(offset, remain) | (self.get_word(offset + remain, length - remain) << remain)
        } else {
            let bit_mask = (1 << length) - 1;
            (self.data[word_index] & (bit_mask << word_offset)) >> word_offset
        }
    }
}

pub const fn compute_word_num(bucket_count: usize, bucket_size: u8) -> usize {
    (bucket_count * bucket_size as usize + BITS_PER_WORD - 1) / BITS_PER_WORD
}

const LN_2_2: f64 = LN_2 * LN_2;

// Calculates the optimal buckets count, m, based on the number of
// items and the desired rate of false positives.
// optimal buckets count = - items_count * ln(fp_rate) / (ln2) ^ 2
fn optimal_bucket_count(items_count: usize, fp_rate: f64) -> usize {
    debug_assert!(items_count > 0);
    debug_assert!(fp_rate > 0.0 && fp_rate < 1.0);
    ((items_count as f64) * fp_rate.ln().abs() / LN_2_2).ceil() as usize
}

// approximate buckets count
// optimal buckets count = - items_count * ln(fp_rate) / (ln2) ^ 2
// = items_count * (ln100 - ln(fp_rate100)) / (ln2) ^ 2
// < items_count * (5 - 1) / 0. 5 ^ 2
// = items_count * 16
pub const fn approximate_bucket_count(items_count: usize) -> usize {
    items_count * 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_bit() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 1) }>::new(100, 1);
        buckets.set(0, 1);
        buckets.set(1, 0);
        buckets.set(2, 1);
        buckets.set(3, 0);
        assert_eq!(1, buckets.get(0));
        assert_eq!(0, buckets.get(1));
        assert_eq!(1, buckets.get(2));
        assert_eq!(0, buckets.get(3));
    }

    #[test]
    fn three_bits() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 3) }>::new(100, 3);
        buckets.set(0, 1);
        buckets.set(1, 2);
        buckets.set(10, 3);
        buckets.set(11, 4);
        buckets.set(20, 5);
        buckets.set(21, 6);
        assert_eq!(1, buckets.get(0));
        assert_eq!(2, buckets.get(1));
        assert_eq!(3, buckets.get(10));
        assert_eq!(4, buckets.get(11));
        assert_eq!(5, buckets.get(20));
        assert_eq!(6, buckets.get(21));
    }

    #[test]
    fn reset() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 1) }>::new(100, 1);
        buckets.set(1, 1);
        assert_eq!(1, buckets.get(1));
        buckets.reset();
        assert_eq!(0, buckets.get(1));
    }

    #[test]
    fn increment() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 3) }>::new(100, 3);
        buckets.increment(10, 2);
        assert_eq!(2, buckets.get(10));
        buckets.increment(10, 1);
        assert_eq!(3, buckets.get(10));
        buckets.increment(10, 100);
        assert_eq!(7, buckets.get(10));
        buckets.increment(10, -1);
        assert_eq!(6, buckets.get(10));
        buckets.increment(10, -10);
        assert_eq!(0, buckets.get(10));

        // test overflow
        let mut buckets = ConstBuckets::<{ compute_word_num(3, 7) }>::new(3, 7);
        buckets.increment(0, 127);
        assert_eq!(127, buckets.get(0));
        buckets.increment(0, 1);
        assert_eq!(127, buckets.get(0));
    }

    #[test]
    fn with_raw_data() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 1) }>::new(100, 1);
        buckets.set(0, 1);
        buckets.set(1, 0);
        buckets.set(2, 1);
        buckets.set(3, 0);
        let raw_data = buckets.raw_data();
        let buckets = ConstBuckets::<{ compute_word_num(100, 1) }>::with_raw_data(100, 1, &raw_data);
        assert_eq!(1, buckets.get(0));
        assert_eq!(0, buckets.get(1));
        assert_eq!(1, buckets.get(2));
        assert_eq!(0, buckets.get(3));

        let mut buckets = ConstBuckets::<{ compute_word_num(100, 3) }>::new(100, 3);
        buckets.set(0, 1);
        buckets.set(1, 2);
        buckets.set(10, 3);
        buckets.set(11, 4);
        buckets.set(20, 5);
        buckets.set(21, 6);
        let raw_data = buckets.raw_data();
        let buckets = ConstBuckets::<{ compute_word_num(100, 3) }>::with_raw_data(100, 3, &raw_data);
        assert_eq!(1, buckets.get(0));
        assert_eq!(2, buckets.get(1));
        assert_eq!(3, buckets.get(10));
        assert_eq!(4, buckets.get(11));
        assert_eq!(5, buckets.get(20));
        assert_eq!(6, buckets.get(21));
    }

    #[test]
    fn update() {
        let mut b1 = ConstBuckets::<{ compute_word_num(100, 1) }>::new(100, 1);
        b1.set(0, 1);
        b1.set(20, 1);
        b1.set(63, 1);

        let mut b2 = ConstBuckets::<{ compute_word_num(50, 1) }>::new(50, 1);
        b2.set(7, 1);
        b2.set(20, 1);
        b2.set(21, 1);
        b2.set(49, 1);

        b1.update(&b2.raw_data());
        assert_eq!(1, b1.get(0));
        assert_eq!(0, b1.get(1));
        assert_eq!(1, b1.get(20));
        assert_eq!(1, b1.get(21));
        assert_eq!(1, b1.get(49));
        assert_eq!(1, b1.get(63));
    }
}
