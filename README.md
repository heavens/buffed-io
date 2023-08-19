# Buffed IO

Create flexible, type-specific buffers capable of catering towards PODs or DSTs.

## Usage

A `Buffered` instance has intimate knowledge of its contents by utilizing the associated `ToSlice` implementor. 

```rust

/// Implementor for a DST which serves as a sophisticated IO object for working with byte vectors.
impl ToSlice for Bytes {
    fn slice(&self, range: std::ops::Range<usize>) -> Option<&[u8]> {
        self.get(range.start..range.end)
    }

    fn slice_to(&self, range: std::ops::RangeTo<usize>) -> Option<&[u8]> {
        self.get(..range.end)
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }
}
```

With this, a `Buffered` object now has the appropriate information to work with a `Bytes` structure such as the size of a single item, the total length as well as indexing methods for convenience.

Going one step further, we may extend the capabilities of a type-specific buffer by providing an implementor with specialized read & write methods:

```rust

impl Buffered<Bytes> {

    /// Attempts to return an unsigned byte from the reader, incrementing the position by `1` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_u8(&mut self) -> Result<u8> {
        ...
    }

    /// Attempts to return a signed byte from the reader, incrementing the position by `1` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_i8(&mut self) -> Result<i8> {
        ...
    }

    /// Attempts to return a signed short from the reader, incrementing the position by `2` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_i16(&mut self) -> Result<i16> {
        ...
    }

    /// Attempts to return an unsigned short from the reader, incrementing the position by `2` if successful. Otherwise
    /// an error is returned if not enough bytes remain.
    pub fn get_u16(&mut self) -> Result<u16> {
        ...
    }
}
```