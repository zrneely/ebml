
use std::io::{self, Bytes, Read};

/// A utility to allow peeking up to 8 bytes into a reader.
#[derive(Debug)]
pub struct PeekableReader<R: Read> {
    buf: Vec<u8>,
    source: Bytes<R>,
}
impl<R: Read> PeekableReader<R> {
    /// Creates a new `PeekableReader` from any `Read` source.
    pub fn new(source: R) -> io::Result<Self> {
        let mut source = source.bytes();
        let buf = source.by_ref().take(8).collect::<Result<Vec<_>, _>>()?;
        Ok(PeekableReader { buf, source })
    }

    /// "Peeks" at the next 8 bytes. Repeated calls return the same values unless `advance` is
    /// called between them.
    ///
    /// Can return fewer than 8 bytes if there are not enough bytes available to be read.
    pub fn peek8(&self) -> &[u8] {
        self.buf.as_slice()
    }

    /// Advances the position of the reader by the specified amount. Returns true if we hit EOF.
    pub fn advance(&mut self, amount: usize) -> io::Result<bool> {
        if amount < 8 {
            self.buf = self.buf.split_off(amount);
            self.buf.append(&mut self.source
                .by_ref()
                .take(amount)
                .collect::<Result<Vec<_>, _>>()?
            );
        } else {
            self.buf = self.source
                .by_ref()
                .skip(amount - 8)
                .take(8)
                .collect::<Result<Vec<_>, _>>()?;
        }
        Ok(self.buf.len() < 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn advancing() {
        let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let source = Cursor::new(data);
        let mut reader = PeekableReader::new(source).unwrap();

        assert_eq!([0, 1, 2, 3, 4, 5, 6, 7], reader.peek8());
        assert!(!reader.advance(1).unwrap());
        assert_eq!([1, 2, 3, 4, 5, 6, 7, 8], reader.peek8());
        assert!(!reader.advance(4).unwrap());
        assert_eq!([5, 6, 7, 8, 9, 10, 11, 12], reader.peek8());

        let mut data = vec![0u8; 255];
        for i in 0..255 {
            data[i] = i as u8;
        }
        let source = Cursor::new(data);
        let mut reader = PeekableReader::new(source).unwrap();

        assert!(!reader.advance(13).unwrap());
        assert_eq!([13, 14, 15, 16, 17, 18, 19, 20], reader.peek8());
    }

    #[test]
    fn eof() {
        let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let source = Cursor::new(data);

        let mut reader = PeekableReader::new(source).unwrap();
        assert!(reader.advance(100).unwrap());
        assert_eq!(0, reader.peek8().len());
    }
}
