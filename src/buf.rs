use std::fmt;
use std::io::{self, ErrorKind, Read, Write};
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

#[derive(Debug)]
pub struct EarlyEofError(usize);

impl std::error::Error for EarlyEofError {}

impl fmt::Display for EarlyEofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct UnVec {
    buf: Vec<u8>,
    start: usize,
}

impl UnVec {
    fn with_capacity(cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
            start: 0,
        }
    }

    fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    fn len(&self) -> usize {
        self.buf.len() - self.start
    }

    fn extract(&mut self, buf: &mut[u8]) {
        let nbuf = buf.len();
        let start = self.start;
        self.start += nbuf;
        buf.copy_from_slice(&mut self.buf[start..self.start]);
        if self.start == self.capacity() {
            self.buf.clear();
            self.start = 0;
        }
    }
}

pub struct StdioBuf<'a> {
    in_buf: UnVec,
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
            in_buf: UnVec::with_capacity(nbuf),
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

    fn read_buf(&mut self, buf: Option<&mut [u8]>) -> io::Result<usize> {
        assert!(self.in_buf.len() == 0);
        assert!(self.in_buf.start == 0);
        let file = self.guard.as_mut().unwrap().get_file();
        match buf {
            None => {
                let n = file.read(&mut self.in_buf.buf)?;
                if n < self.in_buf.capacity() {
                    self.in_buf.buf.truncate(n);
                }
                return Ok(n);
            },
            Some(mut buf) => {
                let nbuf = buf.len();
                assert!(nbuf > 0);
                loop {
                    let n = file.read(buf)?;
                    let nleft = buf.len();
                    if n == nleft {
                        return Ok(nbuf);
                    }
                    if n == 0 {
                        return Err(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            EarlyEofError(nleft),
                        ));
                    }
                    buf = &mut buf[n..];
                }
            }
        };
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

impl<'a> Read for StdioBuf<'a> {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let buf_size = self.in_buf.capacity();
        let nbuf = buf.len();
        let prelocked = self.is_locked();
        let mut locked = prelocked;
        loop {
            let nleft = buf.len();
            let buf_fill = self.in_buf.len();
            if nleft <= buf_fill {
                self.in_buf.extract(buf);
                if locked && !prelocked {
                    self.unlock_buf();
                }
                return Ok(nbuf);
            }
            if buf_fill > 0 {
                self.in_buf.extract(&mut buf[..buf_fill]);
                buf = &mut buf[buf_fill..];
                continue;
            }
            if !locked {
                self.lock_buf()?;
                locked = true;
            }
            if nleft > buf_size {
                let _ = self.read_buf(Some(buf))?;
                if locked && !prelocked {
                    self.unlock_buf();
                }
                return Ok(nbuf);
            }
            let n = self.read_buf(None)?;
            if n == 0 {
                return Ok(0);
            }
        }
    }
}

impl<'a> Drop for StdioBuf<'a> {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}

fn make_stdio(n: usize) -> StdioBuf<'static> {
    StdioBuf::new(1024, &stdio().0[n])
}

pub fn stdin() -> StdioBuf<'static> {
    make_stdio(0)
}
    
pub fn stdout() -> StdioBuf<'static> {
    make_stdio(1)
}
    
pub fn stderr() -> StdioBuf<'static> {
    make_stdio(2)
}
