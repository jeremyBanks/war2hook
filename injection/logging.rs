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

/// Prints a message to the log file and to the in-game text output.
#[macro_export]
macro_rules! wcprintln {
    ($($arg:tt)*) => {
        {
            let message_string = format!($($arg)*);

            let state = unsafe { GAME_STATE.get().read_volatile() };
            let state_s = format!("{state:?}");

            crate::logln!("[{state_s:12}] {message_string}");

            if crate::war2types::GameState::InGame == state {
                let message_cstring = std::ffi::CString::new(message_string).unwrap_or(c"<unable to encode as CString>".into());
                let message_pointer = message_cstring.as_ptr();
                crate::war2types::DISPLAY_MESSAGE(message_pointer, 7, 0);
            }
        }
    };
}

#[macro_export]
macro_rules! wcstatus {
    ($($arg:tt)*) => {
        {
            let message_string = format!($($arg)*);

            let state = unsafe { GAME_STATE.get().read_volatile() };

            if crate::war2types::GameState::InGame == state {
                let message_cstring = std::ffi::CString::new(message_string).unwrap_or(c"<unable to encode as CString>".into());
                let message_pointer = message_cstring.as_ptr();
                crate::war2types::DISPLAY_MESSAGE(message_pointer, 8, 0);
            }
        }
    };
}

pub use crate::{logln, wcprintln, wcstatus};
