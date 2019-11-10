use std::fmt;
use std::io::{self, Write, ErrorKind};
use std::sync::Mutex;

use crate::*;

#[derive(Debug)]
pub struct LockPoisonError;

impl std::error::Error for LockPoisonError {}

impl fmt::Display for LockPoisonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct StdioBuf<'a> {
    #[allow(dead_code)]
    in_buf: Vec<u8>,
    out_buf: Vec<u8>,
    file: &'a Mutex<StdioFile>,
}

impl<'a> StdioBuf<'a> {
    pub fn new(nbuf: usize, file: &'a Mutex<StdioFile>) -> StdioBuf<'a> {
        StdioBuf {
            in_buf: Vec::with_capacity(nbuf),
            out_buf: Vec::with_capacity(nbuf),
            file,
        }
    }
}

impl<'a> Write for StdioBuf<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf_size = self.out_buf.capacity();
        let buf_fill = self.out_buf.len();
        let mut nbuf = buf.len();
        if nbuf + buf_fill > buf_size {
            self.flush()?;
            nbuf = buf_size;
        }
        if nbuf + buf_fill <= buf_size {
            self.out_buf.extend_from_slice(buf);
            return Ok(nbuf);
        }
        let mut file = self.file
            .lock()
            .or_else(|_| Err(io::Error::new(
                ErrorKind::Other,
                LockPoisonError,
            )))?;
        file.get_file().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let buf_fill = self.out_buf.len();
        if buf_fill == 0 {
            return Ok(());
        }
        let mut file = self.file
            .lock()
            .or_else(|_| Err(io::Error::new(
                ErrorKind::Other,
                LockPoisonError,
            )))?;
        let n = file.get_file().write(&self.out_buf)?;
        if n == buf_fill {
            return Ok(());
        }
        unimplemented!("short write");
    }
}

impl<'a> Drop for StdioBuf<'a> {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}

pub fn stdout() -> StdioBuf<'static> {
    StdioBuf::new(1024, &stdio().0[1])
}
