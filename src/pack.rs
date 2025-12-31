pub trait BlockWord: Copy + Eq {
    const BYTES: usize = size_of::<Self>();

    fn pack(bytes: &[u8]) -> Vec<Self>;
    fn unpack(words: &[Self]) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl BlockWord for u64 {
    #[inline]
    fn pack(bytes: &[u8]) -> Vec<Self> {
        bytes
            .chunks_exact(Self::BYTES)
            .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
            .collect()
    }

    #[inline]
    fn unpack(words: &[Self]) -> Vec<u8> {
        let mut out = Vec::with_capacity(words.len() * Self::BYTES);
        for &w in words {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self {
        u64::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl BlockWord for u128 {
    #[inline]
    fn pack(bytes: &[u8]) -> Vec<Self> {
        bytes
            .chunks_exact(Self::BYTES)
            .map(|c| u128::from_le_bytes(c.try_into().unwrap()))
            .collect()
    }

    #[inline]
    fn unpack(words: &[Self]) -> Vec<u8> {
        let mut out = Vec::with_capacity(words.len() * Self::BYTES);
        for &w in words {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self {
        u128::from_le_bytes(bytes.try_into().unwrap())
    }
}
