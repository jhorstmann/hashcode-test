use std::str::from_utf8_unchecked;

#[derive(Default)]
pub struct ArrowStringArray {
    offsets: Vec<usize>,
    data: Vec<u8>,
}

impl ArrowStringArray {
    pub fn is_empty(&self) -> bool {
        self.offsets.len() <= 1
    }

    pub fn len(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    pub fn get(&self, i: usize) -> &str {
        let end = self.offsets[i + 1];
        let start = self.offsets[i];
        let bytes = &self.data[start..end];
        // Safety: can only be constructed from valid strings
        unsafe { from_utf8_unchecked(bytes) }
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        (0..self.len()).map(|i| self.get(i))
    }
}

impl<S: AsRef<str>> FromIterator<S> for ArrowStringArray {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut string_array = ArrowStringArray::default();
        string_array.offsets.push(0);
        iter.into_iter().for_each(|s| {
            string_array.data.extend_from_slice(s.as_ref().as_bytes());
            string_array.offsets.push(string_array.data.len());
        });
        string_array
    }
}
