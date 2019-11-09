use std::cell::RefCell;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::sync::{Arc, Mutex, MutexGuard};

use once_cell::sync::OnceCell;

struct Stdio([Arc<Mutex<RefCell<File>>>; 3]);

fn stdio() -> &'static Stdio {
    static STDIO: OnceCell<Stdio> = OnceCell::new();
    let make_file = |n| {
        Arc::new(Mutex::new(RefCell::new(unsafe {FromRawFd::from_raw_fd(n)})))
    };
    STDIO.get_or_init(|| {
        Stdio([
            make_file(0),
            make_file(1),
            make_file(2),
        ])
    })
}

pub fn stdin_raw() -> MutexGuard<'static, RefCell<File>> {
    stdio().0[0].lock().unwrap()
}

pub fn stdout_raw() -> MutexGuard<'static, RefCell<File>> {
    stdio().0[1].lock().unwrap()
}

pub fn stderr_raw() -> MutexGuard<'static, RefCell<File>> {
    stdio().0[2].lock().unwrap()
}
