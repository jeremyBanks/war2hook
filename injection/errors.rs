use {
    crate::logln,
    std::{
        panic::{catch_unwind, UnwindSafe},
        process::exit,
    },
};

pub fn try_or_die(f: impl Fn() -> Result<(), eyre::Error> + UnwindSafe) {
    std::panic::set_hook(Box::new(|panic_info| {
        let payload_str: Option<&str> = if let Some(s) = panic_info.payload().downcast_ref::<&str>()
        {
            Some(s)
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            Some(s)
        } else {
            None
        };

        if let Some(payload_str) = payload_str {
            logln!("panic: {payload_str}");
        }

        let backtrace = std::backtrace::Backtrace::capture();
        logln!("panic at:\n{backtrace}");
    }));

    match catch_unwind(f) {
        Err(_panic) => {
            logln!("Exiting due to panic.");
            exit(1);
        },
        Ok(Err(error)) => {
            logln!("error: {error:?}");
            logln!("Exiting due to error.");
            exit(1);
        },
        Ok(Ok(())) => {},
    }
}
