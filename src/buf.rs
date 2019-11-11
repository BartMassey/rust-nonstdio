use std::fmt;
use std::io::{self, ErrorKind, Write};
use std::sync::{Mutex, MutexGuard};

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
    guard: Option<MutexGuard<'a, StdioFile>>,
}

impl<'a> StdioBuf<'a> {
    pub fn new(
        nbuf: usize,
        file: &'a Mutex<StdioFile>,
    ) -> StdioBuf<'a> {
        StdioBuf {
            in_buf: Vec::with_capacity(nbuf),
            out_buf: Vec::with_capacity(nbuf),
            file,
            guard: None,
        }
    }

    pub fn lock_buf(&mut self) -> io::Result<()> {
        if self.is_locked() {
            return Ok(());
        }
        let guard = self.file.lock().or_else(|_| {
            Err(io::Error::new(ErrorKind::Other, LockPoisonError))
        })?;
        self.guard = Some(guard);
        Ok(())
    }

    pub fn is_locked(&self) -> bool {
        self.guard.is_some()
    }

    pub fn unlock_buf(&mut self) {
        if self.is_locked() {
            self.guard = None;
        }
    }

    #[must_use]
    pub fn lock(mut self) -> Self {
        self.lock_buf().unwrap();
        self
    }

    fn write_buf(&mut self, buf: Option<&[u8]>) -> io::Result<usize> {
        let buf = match buf {
            None => &self.out_buf,
            Some(buf) => buf,
        };
        self.guard.as_mut().unwrap().get_file().write(buf)
    }
}

impl<'a> Write for StdioBuf<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf_size = self.out_buf.capacity();
        let buf_fill = self.out_buf.len();
        let mut nbuf = buf.len();
        let prelocked = self.is_locked();
        let mut locked = prelocked;
        if nbuf + buf_fill > buf_size {
            if !locked {
                self.lock_buf()?;
                locked = true;
            }
            self.flush()?;
            nbuf = buf_size;
        }
        if nbuf + buf_fill <= buf_size {
            if locked && !prelocked {
                self.unlock_buf();
            }
            self.out_buf.extend_from_slice(buf);
            return Ok(nbuf);
        }
        if !locked {
            self.lock_buf()?;
        }
        let n = self.write_buf(Some(buf))?;
        if !prelocked {
            self.unlock_buf();
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        let buf_fill = self.out_buf.len();
        if buf_fill == 0 {
            return Ok(());
        }
        let prelocked = self.is_locked();
        if !prelocked {
            self.lock_buf()?;
        }
        let n = self.write_buf(None)?;
        if !prelocked {
            self.unlock_buf();
        }
        if n == buf_fill {
            self.out_buf.clear();
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
