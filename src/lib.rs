use std::{
    io,
    ops::{Deref, DerefMut, Range},
};

pub mod bytes;

/// Result type which is simply an alias for the standard library's IO [Result](io::Result).
pub type Result<T> = io::Result<T>;

/// A function that simply returns an [ErrorKind](io::ErrorKind) of [UnexpectedEof](io::ErrorKind::UnexpectedEof). This serves as a helper
/// function to act as a shorthand form for returning EOF errors within a fallible context.
pub fn eof<T>() -> io::Result<T> {
    Err(io::ErrorKind::UnexpectedEof.into())
}

/// Creates a buffer in which implementors can read or write multiple values of a type into an internal buffer so long as the targeted type can be represented as a slice of bytes.
/// This is useful for defining structured routines when processing IO objects within networking.
pub trait ToSlice: Sized {
    /// Creates a slice containing all elements within the specified bounds. This is equivalent to the interval notation `[start..end).`
    fn slice(&self, range: Range<usize>) -> Option<&[u8]>;

    /// Creates a slice containing all elements up until the range specified. This is equivalent to the interval notation `[0..end).`
    fn slice_to(&self, range: std::ops::RangeTo<usize>) -> Option<&[u8]>;

    /// Used to optimistically assume sizes of a slice by using the size of a single item.
    fn item_size_hint() -> usize {
        0
    }

    /// Returns the count of items within the container.
    fn len(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct Buffered<T: ToSlice> {
    buffer: T,
    pos: usize,
}

impl<T> Buffered<T>
where
    T: ToSlice + Default,
{
    /// Constructs a new buffer, using the provided vector as the initial contents and cursor position starting at 0.
    pub fn using(container: T) -> Self {
        Self {
            pos: 0,
            buffer: container,
        }
    }

    /// Constructs a new buffer whose contents are initialized using the designated [Default](std::default::Default) implementation of type `T`.
    pub fn new() -> Self {
        Self {
            buffer: T::default(),
            pos: 0
        }
    }

    /// Returns an immutable reference to the underlying [ToSlice] implementor for the buffer.
    pub fn get_inner(&self) -> &T {
        &self.buffer
    }

    /// Returns a mutable reference to the underlying [ToSlice] implementor for the buffer.
    pub fn get_inner_mut(&mut self) -> &mut T {
        &mut self.buffer
    }

    /// Returns the raw position of the cursor within the buffer.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the index of the cursor within the buffer.
    pub fn index(&self) -> usize {
        self.pos / T::item_size_hint()
    }

    /// Advances the cursor forward by an order of magnitude of `amount * size_hint`.
    pub fn advance_index(&mut self, amount: usize) {
        self.pos += amount * T::item_size_hint();
    }

    /// Sets the cursor within the buffer to the specified index.
    pub fn set_position(&mut self, index: usize) {
        self.pos = index;
    }

    /// Returns the remaining space available within the buffer.
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.pos
    }

    pub fn is_available(&self, amount: usize) -> bool {
        self.remaining() >= amount
    }
}

impl<T> Deref for Buffered<T>
where
    T: ToSlice,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<T> DerefMut for Buffered<T>
where
    T: ToSlice,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!("[{}] {:?}", record.level(), record.line());
    }

    fn flush(&self) {
    }
}


#[cfg(test)]
mod tests {
    use std::env;

    use crate::{Buffered, bytes::Bytes, Logger};

    #[test]
    pub fn single_read() {
        let mut bytes = Buffered::<Bytes>::using(Bytes::new(vec![10u8]));
        let value = bytes.get_u8().expect("read first entry");
        assert!(value == 10)
    }

    #[test]
    pub fn default_read() {
        let bytebuffer: Buffered<Bytes> = Buffered::<Bytes>::new();
        assert!(bytebuffer.is_empty())
    }

    #[test]
    pub fn throw_eof() {
        let mut bytes: Buffered<Bytes> = Buffered::using(vec![10u8].into());
        assert!(bytes.is_available(1));
        bytes.get_i8().expect("read first entry");
        assert!(bytes.remaining() == 0);
        bytes.get_u8().expect("eof");
    }
}