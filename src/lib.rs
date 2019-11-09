#![allow(unused_imports)]
use std::cell::{Ref, RefMut, RefCell};
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::{Deref, DerefMut};
use std::os::unix::io::{RawFd, FromRawFd};
use std::sync::{Arc, Mutex, MutexGuard};

use once_cell::sync::OnceCell;

pub struct StdioRaw(RefCell<File>);

impl StdioRaw {
    pub fn get_file(&mut self) -> &mut File {
        self.0.get_mut()
    }

    fn new(fd: RawFd) -> Self {
        let f = unsafe {FromRawFd::from_raw_fd(fd)};
        Self(RefCell::new(f))
    }
}

struct StdioFiles([Mutex<StdioRaw>; 3]);

fn stdio() -> &'static StdioFiles {
    static STDIO: OnceCell<StdioFiles> = OnceCell::new();
    STDIO.get_or_init(|| {
        StdioFiles([
            Mutex::new(StdioRaw::new(0)),
            Mutex::new(StdioRaw::new(1)),
            Mutex::new(StdioRaw::new(2)),
        ])
    })
}

pub fn stdin_raw() -> MutexGuard<'static, StdioRaw> {
    stdio().0[0].lock().unwrap()
}

pub fn stdout_raw() -> MutexGuard<'static, StdioRaw> {
    stdio().0[1].lock().unwrap()
}

pub fn stderr_raw() -> MutexGuard<'static, StdioRaw> {
    stdio().0[2].lock().unwrap()
}
