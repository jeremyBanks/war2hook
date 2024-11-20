use std::time::Duration;

use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::{core::*, Win32::UI::WindowsAndMessaging::MessageBoxA};
use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => { /* is this going to segfault? */ }
        _ => (),
    }

    true
}

fn attach() {
    let _pid = unsafe { GetCurrentProcessId() };

    std::thread::spawn(|| loop {
        let addr = 0x089813DC;
        let addr = 0x095213DC;
        let addr = 0xABB18;

        let gold = addr as *mut u32;
        if unsafe { *gold } >= 1000 {
            unsafe { *gold += 1 };
        }

        std::thread::sleep(Duration::from_secs(2));
    });
}
