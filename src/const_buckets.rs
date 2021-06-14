use std::mem::size_of;
use std::ptr::copy_nonoverlapping;

type Word = u64;
const BYTES_PER_WORD: usize = size_of::<Word>();
const BITS_PER_WORD: usize = BYTES_PER_WORD * 8;

#[allow(non_upper_case_globals)]
#[derive(Clone)]
/// Buckets Implemetation with const generics
/// WordCount: the count of `Word`
/// BucketCount: the count of bucket
/// BucketSize: the size of one bucket
pub struct ConstBuckets<const WordCount: usize, const BucketCount: usize, const BucketSize: u8> {
    data: [Word; WordCount],
    max: u8,
}

#[allow(non_upper_case_globals)]
impl<const WordCount: usize, const BucketCount: usize, const BucketSize: u8> ConstBuckets<WordCount, BucketCount, BucketSize> {
    /// Creates a new Buckets with the provided number of buckets where
    /// each bucket is the specified number of bits.
    pub fn new() -> Self {
        debug_assert!(BucketSize < 8);
        Self {
            data: [0; WordCount],
            max: (1u8 << BucketSize) - 1,
        }
    }

    pub fn with_raw_data(raw_data: &[u8]) -> Self {
        debug_assert!(BucketSize < 8);
        debug_assert!(WordCount * 8 == raw_data.len());
        let data = [0; WordCount];
        for (idx, buf) in raw_data.chunks(BYTES_PER_WORD).enumerate() {
            let d_slice = &data[idx] as *const _ as *mut u8;
            unsafe {
                copy_nonoverlapping(buf.as_ptr(), d_slice, BYTES_PER_WORD);
            }
        }
        Self {
            data,
            max: (1u8 << BucketSize) - 1,
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
        BucketCount
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
        let offset = bucket * BucketSize as usize;
        let length = BucketSize as usize;
        let word = if byte > self.max as u8 { self.max } else { byte } as Word;
        self.set_word(offset, length, word);
    }

    pub fn get(&self, bucket: usize) -> u8 {
        self.get_word(bucket * BucketSize as usize, BucketSize as usize) as u8
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

pub const fn compute_word_num(bucket_cout: usize, bucket_size: u8) -> usize {
    (bucket_cout * bucket_size as usize + BITS_PER_WORD - 1) / BITS_PER_WORD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_bit() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 1) }, 100, 1>::new();
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
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 3) }, 100, 3>::new();
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
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 1) }, 100, 1>::new();
        buckets.set(1, 1);
        assert_eq!(1, buckets.get(1));
        buckets.reset();
        assert_eq!(0, buckets.get(1));
    }

    #[test]
    fn increment() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 3) }, 100, 3>::new();
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
        let mut buckets = ConstBuckets::<{ compute_word_num(3, 7) }, 3, 7>::new();
        buckets.increment(0, 127);
        assert_eq!(127, buckets.get(0));
        buckets.increment(0, 1);
        assert_eq!(127, buckets.get(0));
    }

    #[test]
    fn with_raw_data() {
        let mut buckets = ConstBuckets::<{ compute_word_num(100, 1) }, 100, 1>::new();
        buckets.set(0, 1);
        buckets.set(1, 0);
        buckets.set(2, 1);
        buckets.set(3, 0);
        let raw_data = buckets.raw_data();
        let buckets = ConstBuckets::<{ compute_word_num(100, 1) }, 100, 1>::with_raw_data(&raw_data);
        assert_eq!(1, buckets.get(0));
        assert_eq!(0, buckets.get(1));
        assert_eq!(1, buckets.get(2));
        assert_eq!(0, buckets.get(3));

        let mut buckets = ConstBuckets::<{ compute_word_num(100, 3) }, 100, 3>::new();
        buckets.set(0, 1);
        buckets.set(1, 2);
        buckets.set(10, 3);
        buckets.set(11, 4);
        buckets.set(20, 5);
        buckets.set(21, 6);
        let raw_data = buckets.raw_data();
        let buckets = ConstBuckets::<{ compute_word_num(100, 3) }, 100, 3>::with_raw_data(&raw_data);
        assert_eq!(1, buckets.get(0));
        assert_eq!(2, buckets.get(1));
        assert_eq!(3, buckets.get(10));
        assert_eq!(4, buckets.get(11));
        assert_eq!(5, buckets.get(20));
        assert_eq!(6, buckets.get(21));
    }

    #[test]
    fn update() {
        let mut b1 = ConstBuckets::<{ compute_word_num(100, 1) }, 100, 1>::new();
        b1.set(0, 1);
        b1.set(20, 1);
        b1.set(63, 1);

        let mut b2 = ConstBuckets::<{ compute_word_num(50, 1) }, 50, 1>::new();
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
