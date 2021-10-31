use nonstdio::nwriteln;

fn main() {
    let n = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "10000000".to_string())
        .parse::<usize>()
        .unwrap();
    let mut stdout = nonstdio::stdout().lock();
    for i in 1..=n {
        match (i % 3 == 0, i % 5 == 0) {
            (false, false) => nwriteln!(stdout, "{}", i),
            (true, false) => nwriteln!(stdout, "fizz"),
            (false, true) => nwriteln!(stdout, "buzz"),
            (true, true) => nwriteln!(stdout, "fizzbuzz"),
        }
    } 
}
