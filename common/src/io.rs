use embedded_io::{ErrorType, Read};
use heapless::Vec;

/// A line reader that buffers input until a newline is found
pub struct LineReader<const N: usize> {
    buffer: Vec<u8, N>,
}

impl<const N: usize> LineReader<N> {
    /// Creates a new LineReader with a fixed-size buffer
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Reads from the given reader until a newline is found or the buffer is full
    /// Returns Ok(true) if a complete line was read, Ok(false) if more data is needed
    /// The line can be accessed via the line() method
    pub fn read_until_newline<R>(&mut self, reader: &mut R) -> Result<bool, R::Error>
    where
        R: Read + ErrorType,
    {
        // If buffer is full, we can't read more
        if self.buffer.is_full() {
            return Ok(true);
        }

        let mut byte_buf = [0u8; 1];

        loop {
            match reader.read(&mut byte_buf) {
                Ok(0) => return Ok(!self.buffer.is_empty()), // EOF
                Ok(_) => {
                    let byte = byte_buf[0];

                    // Check for newline before pushing to avoid unnecessary push
                    if byte == b'\n' {
                        return Ok(true);
                    }

                    // Skip carriage return
                    if byte == b'\r' {
                        continue;
                    }

                    // Try to push byte to buffer
                    if self.buffer.push(byte).is_err() {
                        return Ok(true); // Buffer full
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Returns the current line as a slice, excluding any newline character
    pub fn line(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Clears the internal buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_io::ErrorKind;

    struct MockReader {
        data: &'static [u8],
        pos: usize,
    }

    impl MockReader {
        fn new(data: &'static [u8]) -> Self {
            Self { data, pos: 0 }
        }
    }

    // Implement ErrorType trait
    impl ErrorType for MockReader {
        type Error = ErrorKind;
    }

    impl Read for MockReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            if self.pos >= self.data.len() {
                return Ok(0);
            }

            let byte = self.data[self.pos];
            buf[0] = byte;
            self.pos += 1;
            Ok(1)
        }
    }

    #[test]
    fn test_basic_line_reading() {
        let mut reader = MockReader::new(b"Hello\nWorld\n");
        let mut line_reader = LineReader::<16>::new();

        // Read first line
        assert!(line_reader.read_until_newline(&mut reader).unwrap());
        assert_eq!(line_reader.line(), b"Hello");

        // Clear and read second line
        line_reader.clear();
        assert!(line_reader.read_until_newline(&mut reader).unwrap());
        assert_eq!(line_reader.line(), b"World");
    }

    #[test]
    fn test_buffer_full() {
        let mut reader = MockReader::new(b"ThisIsAVeryLongLine\n");
        let mut line_reader = LineReader::<8>::new();

        assert!(line_reader.read_until_newline(&mut reader).unwrap());
        assert_eq!(line_reader.line(), b"ThisIsAV");
    }
}
