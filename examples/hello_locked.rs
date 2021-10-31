use nonstdio::*;

fn main() {
    let mut stdout = nonstdio::stdout().lock();
    nwriteln!(stdout, "hello world");
}
