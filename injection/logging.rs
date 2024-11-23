use std::{
    fs::{File, OpenOptions},
    sync::{LazyLock, Mutex},
};

pub static LOG_FILE: LazyLock<Mutex<File>> = LazyLock::new(|| {
    let date = chrono::Utc::now();
    let date = date.format("%Y-%m-%d-%H");

    let log_path = format!("C:\\Users\\_\\war2hook\\logs\\{date}.log");

    Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Unable to open log file"),
    )
});

/// Prints a message to the log file.
#[macro_export]
macro_rules! logln {
    ($($arg:tt)*) => {
        {
            use std::io::Write;

            let date = chrono::Utc::now();
            let date = date.format("%H:%m:%S%.3f");

            let message = format!($($arg)*);

            for line in message.lines() {
                writeln!(crate::logging::LOG_FILE.lock().unwrap(), "{date} {line}").unwrap();
            }
        }
    };
}

pub use crate::logln;
