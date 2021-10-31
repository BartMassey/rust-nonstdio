use std::io::Write;

fn main() {
    let mut stdout = nonstdio::stdout();
    stdout.write_all(b"hello world\n").unwrap();
}
