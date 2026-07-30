pub struct ChunkedIter<I: Iterator> {
    iterator: I,
    chunk_size: usize,
}
impl<I: Iterator> ChunkedIter<I> {
    pub fn new(iterator: I, chunk_size: usize) -> Self {
        Self {
            iterator,
            chunk_size,
        }
    }
}
impl<I: Iterator> Iterator for ChunkedIter<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk: Vec<I::Item> = self.iterator.by_ref().take(self.chunk_size).collect();
        if chunk.is_empty() { None } else { Some(chunk) }
    }
}
