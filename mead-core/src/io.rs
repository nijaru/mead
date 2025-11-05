//! I/O abstractions for media sources

use std::io::{Read, Seek, Result};

/// Trait for media data sources (files, streams, network)
///
/// Combines Read + Seek with runtime seekability detection.
/// This allows the same API to handle both seekable (files) and
/// non-seekable (stdin, network streams) sources.
pub trait MediaSource: Read + Seek {
    /// Returns true if this source supports seeking
    ///
    /// Files return true, stdin/network streams return false
    fn is_seekable(&self) -> bool;

    /// Returns the total size of the source if known
    ///
    /// Files can report their size, streams typically cannot
    fn len(&self) -> Option<u64>;

    /// Returns true if the source is empty (size is 0)
    fn is_empty(&self) -> bool {
        self.len() == Some(0)
    }
}

/// MediaSource implementation for std::fs::File
impl MediaSource for std::fs::File {
    fn is_seekable(&self) -> bool {
        true
    }

    fn len(&self) -> Option<u64> {
        self.metadata().ok().map(|m| m.len())
    }
}

/// MediaSource implementation for std::io::Cursor
impl<T: AsRef<[u8]>> MediaSource for std::io::Cursor<T> {
    fn is_seekable(&self) -> bool {
        true
    }

    fn len(&self) -> Option<u64> {
        Some(self.get_ref().as_ref().len() as u64)
    }
}

/// Wrapper for non-seekable sources (stdin, network)
///
/// Provides a dummy Seek implementation that always returns an error.
/// This allows Read-only sources to satisfy the MediaSource trait bounds
/// while clearly indicating they don't support seeking.
#[derive(Debug)]
pub struct ReadOnlySource<R: Read> {
    inner: R,
}

impl<R: Read> ReadOnlySource<R> {
    /// Create a new read-only source
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R: Read> Read for ReadOnlySource<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> Seek for ReadOnlySource<R> {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Source does not support seeking"
        ))
    }
}

impl<R: Read> MediaSource for ReadOnlySource<R> {
    fn is_seekable(&self) -> bool {
        false
    }

    fn len(&self) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_file_is_seekable() {
        let file = std::fs::File::open("Cargo.toml").unwrap();
        assert!(file.is_seekable());
        assert!(file.len().is_some());
    }

    #[test]
    fn test_readonly_not_seekable() {
        let data = vec![1, 2, 3, 4];
        let cursor = Cursor::new(data);
        let source = ReadOnlySource::new(cursor);
        assert!(!source.is_seekable());
        assert!(source.len().is_none());
    }

    #[test]
    fn test_readonly_seek_fails() {
        let data = vec![1, 2, 3, 4];
        let cursor = Cursor::new(data);
        let mut source = ReadOnlySource::new(cursor);

        use std::io::SeekFrom;
        let result = source.seek(SeekFrom::Start(0));
        assert!(result.is_err());
    }
}
