use {
    std::{
        ffi::{c_char, CString},
        fs::OpenOptions,
        io::Write,
        ptr::NonNull,
        time::Duration,
    },
    volatile::VolatilePtr,
    windows::{
        core::*,
        Win32::{
            Foundation::*,
            System::{SystemServices::*, Threading::GetCurrentProcessId},
            UI::WindowsAndMessaging::MessageBoxA,
        },
    },
};

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => {
            panic!("detaching not supported. panicking to avoid memory unsafety.")
        },
        _ => (),
    }

    true
}

fn attach() {
    let mut log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("C:\\Users\\_\\war2hook\\log.txt")
        .expect("Unable to open log file");

    std::thread::spawn(move || {
        let gold = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABB18 as *mut u32)) };
        let lumber = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ACB6C as *mut u32)) };
        let oil = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABBFC as *mut u32)) };

        let displayMessage: extern fn(message: *const i8, _2: u8, _3: u32) =
            unsafe { std::mem::transmute(0x0042ca40) };

        let mut last_line = String::new();

        loop {
            let current_gold = gold.read();
            let current_lumber = lumber.read();
            let current_oil = oil.read();

            let line =
                format!("gold: {current_gold}, lumber: {current_lumber}, oil: {current_oil}\n");

            if line != last_line {
                log.write_all(line.as_bytes()).unwrap();
                last_line = line;
            }

            if current_gold > 0 {
                gold.write(1337);
                lumber.write(1337);
                oil.write(1337);

                displayMessage(CString::new("let's top you up!").unwrap().as_ptr(), 0, 0);
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    });
}
