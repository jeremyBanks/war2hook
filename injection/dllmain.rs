use {
    crate::{errors::try_or_die, logging::logln},
    std::{
        panic::{catch_unwind, UnwindSafe},
        process::exit,
    },
    windows::Win32::{Foundation::HINSTANCE, System::SystemServices::DLL_PROCESS_ATTACH},
};

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn DllMain(_module: HINSTANCE, event: u32, _: *mut ()) -> bool {
    match event {
        DLL_PROCESS_ATTACH => {
            logln!("Handling Dllmain attach event ({event}).");
            try_or_die(crate::install);
            true
        },
        _ => {
            logln!("Ignoring Dllmain event {event}.");
            false
        },
    }
}
