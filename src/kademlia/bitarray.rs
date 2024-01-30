use std::ops::BitXor;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BitArray<const N: usize>([u8; N]);

impl<const N: usize> BitArray<N> {
    pub fn get(&self, index: usize) -> bool {
        let (pos, shift) = (index / 8, index % 8);
        self.0[pos] >> shift & 1 == 1
    }

    pub fn leading_zeros(&self) -> usize {
        let mut i = 0;
        while i != N * 8 && !self.get(i) {
            i += 1;
        }
        i
    }
}

impl<const N: usize> BitXor for BitArray<N> {
    type Output = Self;

    fn bitxor(mut self, rhs: Self) -> Self::Output {
        for i in 0..self.0.len() {
            self.0[i] ^= rhs.0[i];
        }
        self
    }
}
