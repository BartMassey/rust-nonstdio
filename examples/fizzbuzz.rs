use nonstdio::nwriteln;

fn main() {
    let n = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>().unwrap())
        .unwrap_or(10_000_000);
    let mut stdout = nonstdio::stdout();
    for i in 1..=n {
        match (i % 3 == 0, i % 5 == 0) {
            (false, false) => nwriteln!(stdout, "{}", i),
            (true, false) => nwriteln!(stdout, "fizz"),
            (false, true) => nwriteln!(stdout, "buzz"),
            (true, true) => nwriteln!(stdout, "fizzbuzz"),
        }
    }
}
