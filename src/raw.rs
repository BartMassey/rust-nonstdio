use std::cell::RefCell;
use std::fs::File;
use std::os::unix::io::{FromRawFd, RawFd};
use std::sync::MutexGuard;

use crate::*;

pub struct StdioFile(RefCell<File>);

impl StdioFile {
    pub fn get_file(&mut self) -> &mut File {
        self.0.get_mut()
    }

    pub unsafe fn from_fd(fd: RawFd) -> Self {
        let file = FromRawFd::from_raw_fd(fd);
        Self(RefCell::new(file))
    }
}

fn stdio_raw(fd: usize) -> MutexGuard<'static, StdioFile> {
    stdio().0[fd].lock().unwrap()
}

pub fn stdin_raw() -> MutexGuard<'static, StdioFile> {
    stdio_raw(0)
}

pub fn stdout_raw() -> MutexGuard<'static, StdioFile> {
    stdio_raw(1)
}

pub fn stderr_raw() -> MutexGuard<'static, StdioFile> {
    stdio_raw(2)
}
