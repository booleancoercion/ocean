use colored::Colorize;

use std::fmt::Display;

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::print::error(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::print::info(format!($($arg)*))
    };
}

pub fn error<S: Display>(msg: S) {
    eprintln!(
        "{}: {}: {}",
        "ocean".white().bold(),
        "error".red().bold(),
        msg
    )
}

pub fn info<S: Display>(msg: S) {
    println!(
        "{}: {}: {}",
        "ocean".white().bold(),
        "info".white().bold(),
        msg
    )
}

pub trait OnError {
    fn on_err(self, f: impl FnOnce()) -> Self;
}

impl<T, E> OnError for Result<T, E> {
    fn on_err(self, f: impl FnOnce()) -> Self {
        self.map_err(|err| {
            f();
            err
        })
    }
}
