use std::sync::Mutex;

use once_cell::sync::OnceCell;

use crate::*;

pub struct StdioFiles(pub [Mutex<StdioFile>; 3]);

pub fn stdio() -> &'static StdioFiles {
    static STDIO: OnceCell<StdioFiles> = OnceCell::new();
    let stdio_make = |fd| Mutex::new(unsafe { StdioFile::from_fd(fd) });
    STDIO.get_or_init(|| StdioFiles([stdio_make(0), stdio_make(1), stdio_make(2)]))
}
