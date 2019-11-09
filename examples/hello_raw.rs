use std::io::Write;
use nonstdio::*;

fn main() {
    let mut stdout = stdout_raw();
    let stdout = stdout.get_file();
    let _ = writeln!(stdout, "hello world").unwrap();
}
