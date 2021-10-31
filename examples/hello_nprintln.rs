use nonstdio::*;

fn main() {
    let mut stdout = nonstdio::stdout();
    nwrite!(stdout, "prestart…\n");
    nprint!("starting…\n");
    neprint!("still starting…\n");
    neprintln!("still more starting…");
    nprintln!("hello world");
}
