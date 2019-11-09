use std::io::Write;
use nonstdio::*;

fn main() {
    let mut stdout = stdout_raw();
    writeln!(stdout.get_mut(), "hello world").unwrap();
}
