use std::io::Read;

use nonstdio::*;

fn main() {
    let mut stdin = stdin();
    let mut contents = Vec::new();
    let _ = stdin.read_to_end(&mut contents).unwrap();
    println!("{}", std::str::from_utf8(&contents).unwrap());
}
