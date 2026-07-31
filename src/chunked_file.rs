use std::{
    fs::File,
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
};

use bitcode::{DecodeOwned, Encode};

/// handles writing a type T to, and then reading that type T from, a file in chunks
pub struct ChunkedFile<'a, T: Encode + DecodeOwned> {
    file: &'a mut File,
    finished_writing: bool,
    _marker: PhantomData<T>,
}
impl<'a, T> ChunkedFile<'a, T>
where
    T: Encode + DecodeOwned,
{
    pub fn new(file: &'a mut File) -> Self {
        Self {
            file,
            finished_writing: false,
            _marker: PhantomData,
        }
    }
    pub fn finish_writing(&mut self) {
        self.finished_writing = true;
        self.file.rewind().unwrap();
    }
    //TODO: is it bad to unwrap() all of these io tasks
    pub fn size(&self) -> u64 {
        self.file.metadata().unwrap().len()
    }
    pub fn write_chunk(&mut self, chunk: T) {
        if self.finished_writing {
            panic!("attempt to call write_chunk when finished_writing=true");
        }
        let bytes = bitcode::encode(&chunk);
        self.file
            .write_all(&(bytes.len() as u64).to_le_bytes())
            .unwrap();
        self.file.write_all(&bytes).unwrap();
    }
}
impl<'a, T> Iterator for ChunkedFile<'a, T>
where
    T: Encode + DecodeOwned,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.finished_writing {
            panic!("attempt to read a chunk when finished_writing=false");
        }
        let mut size_bytes = [0; 8];
        match self.file.read_exact(&mut size_bytes) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                return None;
            }
            Err(e) => panic!("failed to read chunk: {}", e),
        }
        let size = u64::from_le_bytes(size_bytes) as usize;
        let mut bytes = vec![0; size];
        self.file.read_exact(&mut bytes).unwrap();
        Some(bitcode::decode(&bytes).unwrap())
    }
}
