use std::fmt;
use std::io::{self, ErrorKind, Read, Write};
use std::sync::{Mutex, MutexGuard};

use crate::*;

#[cfg(feature = "debug")]
macro_rules! log {
    ($($args:expr),*) => {eprintln!($($args),*)};
}

#[cfg(not(feature = "debug"))]
macro_rules! log {
    ($($args:expr),*) => {};
}

#[derive(Debug)]
pub struct LockPoisonError;

impl std::error::Error for LockPoisonError {}

impl fmt::Display for LockPoisonError {
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

    fn extract(&mut self, buf: &mut [u8]) {
        let nbuf = buf.len();
        let start = self.start;
        self.start += nbuf;
        buf.copy_from_slice(&self.buf[start..self.start]);
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
    in_closed: bool,
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
            in_closed: false,
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

    fn read_buf(
        &mut self,
        buf: Option<&mut [u8]>,
    ) -> io::Result<usize> {
        assert!(!self.in_closed);
        let file = self.guard.as_mut().unwrap().get_file();
        log!("starting read_buf f:{:?} b:{}", file, buf.is_some());
        match buf {
            None => {
                assert!(self.in_buf.len() == 0);
                let buf_size = self.in_buf.capacity();
                self.in_buf.start = 0;
                self.in_buf.buf.resize(buf_size, 0);
                let n = file.read(&mut self.in_buf.buf)?;
                log!("read_buf free fill n:{}", n);
                if n == 0 {
                    self.in_closed = true;
                }
                if n < buf_size {
                    self.in_buf.buf.truncate(n);
                }
                Ok(n)
            }
            Some(mut buf) => {
                let nbuf = buf.len();
                assert!(nbuf > 0);
                loop {
                    let nleft = buf.len();
                    if nleft == 0 {
                        return Ok(nbuf);
                    }
                    let n = file.read(buf)?;
                    log!("read_buf target fill n:{}", n);
                    if n == 0 {
                        self.in_closed = true;
                        return Ok(nbuf - nleft);
                    }
                    buf = &mut buf[n..];
                }
            }
        }
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
        if self.in_closed {
            return Ok(0);
        }
        let buf_size = self.in_buf.capacity();
        let nbuf = buf.len();
        log!("starting read n:{} bs:{}", nbuf, buf_size);
        let prelocked = self.is_locked();
        let mut locked = prelocked;
        loop {
            let nleft = buf.len();
            let buf_fill = self.in_buf.len();
            log!("starting loop n:{} bs:{}", nleft, buf_fill);
            if nleft <= buf_fill {
                self.in_buf.extract(buf);
                if locked && !prelocked {
                    self.unlock_buf();
                }
                log!("satisfying read from buffer n:{}", nbuf);
                return Ok(nbuf);
            }
            if buf_fill > 0 {
                log!("partial read from buffer n:{}", buf_fill);
                self.in_buf.extract(&mut buf[..buf_fill]);
                buf = &mut buf[buf_fill..];
                continue;
            }
            if !locked {
                self.lock_buf()?;
                locked = true;
            }
            if nleft > buf_size {
                let n = self.read_buf(Some(buf))?;
                if locked && !prelocked {
                    self.unlock_buf();
                }
                let nread = nbuf - nleft + n;
                log!("overfull read n:{} r:{}", nbuf, nread);
                return Ok(nread);
            }
            let n = self.read_buf(None)?;
            log!("filling buffer n:{}", n);
            if n == 0 {
                if locked && !prelocked {
                    self.unlock_buf();
                }
                self.in_closed = true;
                let n = nbuf - nleft;
                log!("returning n:{}", n);
                return Ok(n);
            }
        }
    }
}

impl<'a> Drop for StdioBuf<'a> {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}

fn make_stdio(fd: usize, nbuf: usize) -> StdioBuf<'static> {
    StdioBuf::new(nbuf, &stdio().0[fd])
}

pub fn stdin() -> StdioBuf<'static> {
    make_stdio(0, STDIO_NBUF)
}

pub fn stdout() -> StdioBuf<'static> {
    make_stdio(1, STDIO_NBUF)
}

pub fn stderr() -> StdioBuf<'static> {
    make_stdio(2, STDIO_NBUF)
}
