#[macro_export]
macro_rules! nwriteln {
    ($file:expr, $($args:tt)*) => {{
        $crate::nwrite!($file, $($args)*);
        $crate::nwrite!($file, "\n");
    }};
}

#[macro_export]
macro_rules! nprintln {
    ($($args:tt)*) => {{
        let mut stdout = $crate::stdout();
        $crate::nwriteln!(stdout, $($args)*);
    }};
}

#[macro_export]
macro_rules! nwrite {
    ($file: expr, $($args:tt)*) => {{
        use std::io::Write;
        assert!($file.write(format!($($args)*).as_bytes()).is_ok());
    }};
}

#[macro_export]
macro_rules! nprint {
    ($($args:tt)*) => {{
        let mut stdout = $crate::stdout();
        $crate::nwrite!(stdout, $($args)*);
    }};
}

#[macro_export]
macro_rules! neprintln {
    ($($args:tt)*) => {{
        let mut stderr = $crate::stderr();
        $crate::nwriteln!(stderr, $($args)*);
    }};
}

#[macro_export]
macro_rules! neprint {
    ($($args:tt)*) => {{
        let mut stderr = $crate::stderr();
        $crate::nwrite!(stderr, $($args)*);
    }};
}
