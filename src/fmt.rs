#[macro_export]
macro_rules! nprintln {
    ($($args:tt)*) => {{
        use std::io::Write;
        let mut stdout = $crate::stdout().lock();
        stdout.write_fmt(format_args!($($args)*)).unwrap();
        let _ = stdout.write(b"\n").unwrap();
    }}
}

#[macro_export]
macro_rules! nprint {
    ($($args:tt)*) => {{
        use std::io::Write;
        let mut stdout = $crate::stdout();
        stdout.write_fmt(format_args!($($args)*)).unwrap();
    }}
}

#[macro_export]
macro_rules! neprintln {
    ($($args:tt)*) => {{
        use std::io::Write;
        let mut stderr = $crate::stderr().lock();
        stderr.write_fmt(format_args!($($args)*)).unwrap();
        let _ = stderr.write(b"\n").unwrap();
    }}
}

#[macro_export]
macro_rules! neprint {
    ($($args:tt)*) => {{
        use std::io::Write;
        let mut stderr = $crate::stderr();
        stderr.write_fmt(format_args!($($args)*)).unwrap();
    }}
}
