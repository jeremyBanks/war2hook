use {
    std::{fs::OpenOptions, io::Write, ptr::NonNull, time::Duration},
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
    let pid = unsafe { GetCurrentProcessId() };

    let mut log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("C:\\Users\\_\\war2hook\\log.txt")
        .expect("Unable to open log file");

    std::thread::spawn(move || loop {
        let gold = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABB18 as *mut u32)) };
        let lumber = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ACB6C as *mut u32)) };
        let oil = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABBFC as *mut u32)) };

        let current_gold = gold.read();
        let current_lumber = lumber.read();
        let current_oil = oil.read();

        writeln!(
            log,
            "gold: {current_gold}, lumber: {current_lumber}, oil: {current_oil}"
        )
        .unwrap();

        if current_gold > 0 {
            gold.write(1337);
            lumber.write(1337);
            oil.write(1337);
        }

        std::thread::sleep(Duration::from_secs(1));
    });
}
