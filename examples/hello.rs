use std::io::Write;

use nonstdio::*;

fn main() {
    let mut stdout = stdout();
    let _ = writeln!(stdout, "hello world").unwrap();
}
