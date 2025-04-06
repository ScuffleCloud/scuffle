#[derive(Debug, Clone)]
pub(crate) struct BitField<const N: usize = 8> {
    inline: [u8; N],
}

impl<const N: usize> BitField<N> {
    pub const fn set(&mut self, idx: usize) -> bool {
        if idx >= self.capacity() {
            return false;
        }

        let (byte_idx, bit_idx) = (idx / 8, idx % 8);
        let bit = 1 << bit_idx;
        let byte = &mut self.inline[byte_idx];

        if *byte & bit != 0 {
            false
        } else {
            *byte |= bit;
            true
        }
    }

    pub fn get(&self, idx: usize) -> bool {
        if idx >= self.capacity() {
            return false;
        }

        let (byte_idx, bit_idx) = (idx / 8, idx % 8);

        let bit = 1 << bit_idx;
        let byte = self.inline[byte_idx];

        byte & bit != 0
    }

    pub const fn capacity(&self) -> usize {
        N * 8
    }
}

impl<const N: usize> Default for BitField<N> {
    fn default() -> Self {
        Self { inline: [0; N] }
    }
}
