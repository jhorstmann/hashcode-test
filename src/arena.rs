pub struct Arena {
    data: Vec<u8>,
}

impl Arena {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn append(&mut self, value: &[u8]) -> usize {
        assert!(value.len() <= u32::MAX as usize);
        let index = self.data.len();
        self.data
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.data.extend_from_slice(value);
        index
    }

    #[inline]
    pub fn lookup(&self, index: usize) -> &[u8] {
        let len = u32::from_le_bytes(self.data[index..index + 4].try_into().unwrap()) as usize;
        &self.data[index + 4..index + 4 + len]
    }
}
