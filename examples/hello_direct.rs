use std::io::Write;

fn main() {
    let mut stdout = nonstdio::stdout();
    stdout.write(b"hello world\n").unwrap();
}
