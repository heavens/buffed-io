use crate::bytes::composite::write_u24;
use crate::{eof, Buffered, Result, ToSlice};
use std::io;
use std::mem;
use std::ops::{Deref, Index, RangeFrom, RangeInclusive, RangeTo};

use self::composite::read_u24;

macro_rules! impl_get_bytes {
    ($buf:ident, $byte_ty:ty, $conversion:expr) => {{
        const SIZE: usize = mem::size_of::<$byte_ty>();
        let limit = $buf.len();
        let pos = $buf.pos();
        if pos + SIZE > limit {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let slice = unsafe { *($buf.bytes[pos..pos + SIZE].as_ptr() as *const [_; SIZE]) };
        $buf.advance_index(SIZE);
        Ok($conversion(slice))
    }};
}

macro_rules! impl_put_bytes {
    ($this:tt, $value:tt) => {{
        let pos = $this.pos();
        let slice_len = $value.len();
        let buf_len = $this.bytes.len();
        if pos + slice_len >= buf_len {
            $this.bytes.resize(buf_len * 2, 0u8);
        }

        $this.bytes[pos..pos + slice_len].copy_from_slice($value);
        $this.advance_index(slice_len);
    }};
}

#[derive(Clone, Debug, Default)]
pub struct Bytes {
    bytes: Vec<u8>,
}

impl Bytes {
    /// Constructs a new byte buffer using the provided vector as the initial contents.
    pub fn new(contents: Vec<u8>) -> Self {
        Self { bytes: contents }
    }
}

impl Buffered<Bytes> {
    /// Returns an immutable reference to the underlying byte slice.
    pub fn bytes(&self) -> &[u8] {
        &self.buffer.bytes
    }

    /// Returns a mutable reference to the underlying byte slice.
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Attempts to return an unsigned byte from the reader, incrementing the position by `1` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_u8(&mut self) -> Result<u8> {
        impl_get_bytes!(self, u8, u8::from_be_bytes)
    }

    /// Attempts to return a signed byte from the reader, incrementing the position by `1` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_i8(&mut self) -> Result<i8> {
        impl_get_bytes!(self, i8, i8::from_be_bytes)
    }

    /// Attempts to return a signed short from the reader, incrementing the position by `2` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_i16(&mut self) -> Result<i16> {
        impl_get_bytes!(self, i16, i16::from_be_bytes)
    }

    /// Attempts to return an unsigned short from the reader, incrementing the position by `2` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_u16(&mut self) -> Result<u16> {
        impl_get_bytes!(self, u16, u16::from_be_bytes)
    }

    /// Attempts to return a 24-bit unsigned integer from the reader, incrementing the position by `3` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_u24(&mut self) -> Result<usize> {
        if self.is_available(3) {
            let value = read_u24(&self.bytes[self.pos..self.pos + 3]);
            self.advance_index(3);
            Ok(value)
        } else {
            eof()
        }
    }

    /// Attempts to return a signed integer from the reader, incrementing the position by `4` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_i32(&mut self) -> Result<i32> {
        impl_get_bytes!(self, i32, i32::from_be_bytes)
    }

    /// Attempts to return an unsigned integer from the reader, incrementing the position by `4` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_u32(&mut self) -> Result<u32> {
        impl_get_bytes!(self, u32, u32::from_be_bytes)
    }

    /// Attempts to return a signed long from the reader, incrementing the position by `8` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_i64(&mut self) -> Result<i64> {
        impl_get_bytes!(self, i64, i64::from_be_bytes)
    }

    /// Attempts to return an unsigned long from the reader, incrementing the position by `8` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_u64(&mut self) -> Result<u64> {
        impl_get_bytes!(self, u64, u64::from_be_bytes)
    }

    /// Tries to read a null-terminated string (c-string) from the reader, returning an error if the operation could not complete. The reader
    /// position is incremented based on the width of the string read.
    pub fn get_str(&mut self) -> Result<String> {
        let pos = self.pos;
        let Some(index) = self.iter().position(|c| *c == 0) else {
            return eof();
        };

        String::from_utf8(self.bytes[pos..index].to_vec())
            .map(|str| {
                self.pos += str.len() + 1;
                str
            })
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    /// Writes an unsigned byte value into the buffer, incrementing the position by `1`.
    pub fn put_u8(&mut self, value: u8) {
        let slice = &u8::to_be_bytes(value);
        impl_put_bytes!(self, slice);
    }

    /// Writes a signed byte value into the buffer, incrementing the position by `1`.
    pub fn put_i8(&mut self, value: i8) {
        let slice = &i8::to_be_bytes(value);
        impl_put_bytes!(self, slice);
    }

    /// Writes a signed short value into the buffer, incrementing the position by `2`.
    pub fn put_i16(&mut self, value: i16) {
        let slice = &i16::to_be_bytes(value);
        impl_put_bytes!(self, slice);
    }

    /// Writes an unsigned short value into the buffer, incrementing the position by `2`.
    pub fn put_u16(&mut self, value: u16) {
        let slice: &[u8; 2] = &u16::to_be_bytes(value);
        impl_put_bytes!(self, slice);
    }

    pub fn put_u24(&mut self, value: u32) {
        let slice = &write_u24(value);
        impl_put_bytes!(self, slice);
    }

    /// Writes a signed int value into the buffer, incrementing the position by `4`.
    pub fn put_i32(&mut self, value: i32) {
        let slice = &i32::to_be_bytes(value);
        impl_put_bytes!(self, slice);
    }

    /// Writes an unsigned int value into the buffer, incrementing the position by `4`.
    pub fn put_u32(&mut self, value: u32) {
        let slice = &u32::to_be_bytes(value);
        impl_put_bytes!(self, slice);
    }

    /// Writes an unsigned int value into the buffer, incrementing the position by `8`.
    pub fn put_u64(&mut self, value: u64) {
        let slice = &u64::to_be_bytes(value);
        impl_put_bytes!(self, slice);
    }

    /// Writes a null-terminated string value into the buffer, incremeneting the position by `value.len() + 1`.
    pub fn put_str<S: AsRef<str>>(&mut self, value: S) {
        let bytes: &[u8] = value.as_ref().as_bytes();
        impl_put_bytes!(self, bytes);
        self.put_u8(0);
    }
}

impl ToSlice for Bytes {
    fn slice(&self, range: std::ops::Range<usize>) -> Option<&[u8]> {
        self.get(range.start..range.end)
    }

    fn slice_to(&self, range: std::ops::RangeTo<usize>) -> Option<&[u8]> {
        self.get(..range.end)
    }

    fn item_size_hint() -> usize {
        1
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.bytes.deref()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Self {
        Bytes { bytes: value }
    }
}

impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Bytes {
            bytes: value.to_vec(),
        }
    }
}

impl<const N: usize> From<&[u8; N]> for Bytes {
    fn from(value: &[u8; N]) -> Self {
        Bytes::new(value.to_vec())
    }
}

impl Index<RangeInclusive<usize>> for Bytes {
    type Output = [u8];

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        let start = *index.start();
        let end = *index.end();
        if end <= start {
            return &[];
        }
        &self[start..=end]
    }
}

impl Index<RangeTo<usize>> for Bytes {
    type Output = [u8];

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        let end = index.end;
        if end == 0 {
            return &[];
        }
        &self[..end]
    }
}

impl Index<RangeFrom<usize>> for Bytes {
    type Output = [u8];

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        let start = index.start;
        if start >= self.len() {
            return &[];
        }
        &self[start..]
    }
}

/// Composites used within buffer operations.
pub(crate) mod composite {

    /// A helper function reading a 24-bit value from the byte slice.
    pub(crate) fn read_u24(buf: &[u8]) -> usize {
        (((buf[0] as u32) << 16) + ((buf[1] as u32) << 8) + (buf[2] as u32 & 255)) as usize
    }

    /// A helper function writing a 24-bit value into a fixed-length byte slice.
    pub(crate) fn write_u24(value: u32) -> [u8; 3] {
        let mut bytes = [0u8; 3];
        bytes[0] = (value >> 16) as u8;
        bytes[1] = (value >> 8) as u8;
        bytes[2] = value as u8;
        bytes
    }
}
