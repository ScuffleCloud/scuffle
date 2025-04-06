#[derive(Debug, Clone)]
pub(crate) enum BitField {
    Inline([u8; Self::MAX_INLINE]),
    Heap(Vec<u8>),
}

const _: () = {
    assert!(std::mem::size_of::<BitField>() == 24);
};

impl BitField {
    const MAX_INLINE: usize = 16;

    pub fn set(&mut self, idx: usize) -> bool {
        let (byte_idx, bit_idx) = (idx / 8, idx % 8);
        if byte_idx >= Self::MAX_INLINE {
            self.make_heap(byte_idx);
        }

        let bit = 1 << bit_idx;
        let byte = match self {
            Self::Inline(bytes) => &mut bytes[byte_idx],
            Self::Heap(bits) => &mut bits[byte_idx],
        };

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
        let byte = match self {
            Self::Inline(bytes) => bytes[byte_idx],
            Self::Heap(bits) => bits[byte_idx],
        };

        byte & bit != 0
    }

    fn make_heap(&mut self, reserve: usize) {
        match self {
            Self::Inline(value) => {
                let mut bits = vec![0; reserve];
                bits.copy_from_slice(value);
                *self = Self::Heap(bits);
            }
            Self::Heap(bits) => bits.resize(reserve.max(bits.len()), 0),
        }
    }

    fn capacity(&self) -> usize {
        match self {
            Self::Inline(bytes) => bytes.len() * 8,
            Self::Heap(bits) => bits.len() * 8,
        }
    }
}

impl Default for BitField {
    fn default() -> Self {
        Self::Inline([0; Self::MAX_INLINE])
    }
}
