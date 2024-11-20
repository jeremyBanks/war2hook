use {
    iced_x86::{self, code_asm::CodeAssembler},
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

#[no_mangle]
extern fn apply_cheats_hook(newCheats: u32, _2: i32) {
    let mut hook_log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("C:\\Users\\_\\war2hook\\hook.txt")
        .expect("Unable to open log file");

    writeln!(hook_log, "you did it! it worked!").unwrap();
}

fn attach() {
    let mut log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("C:\\Users\\_\\war2hook\\log.txt")
        .expect("Unable to open log file");

    writeln!(log, "assembling hook!").unwrap();

    let call_hook = {
        use iced_x86::code_asm::*;

        let mut a = CodeAssembler::new(32).unwrap();

        a.call(ptr(apply_cheats_hook as *const (u32, i32) as u32))
            .unwrap();

        a.ret().unwrap();

        a.assemble(0).unwrap()
    };

    // We're hooking the "day" cheat code, by overwriting the
    // instructions at 0x4_160a4 with our assembled bytes. It returns,
    // so we don't have to worry about corrupting the subsequent
    // instructions if ours doesn't align.
    let target: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(0x4_160A4), 1000) };

    writeln!(log, "installing hook!").unwrap();

    target.copy_from_slice(&call_hook);

    writeln!(log, "installed hook!").unwrap();

    std::thread::spawn(move || {
        let gold = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABB18 as *mut u32)) };
        let lumber = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ACB6C as *mut u32)) };
        let oil = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABBFC as *mut u32)) };

        let apply_cheats: extern fn(newCheats: u32, _2: i32) =
            unsafe { std::mem::transmute(0x4_15EC0) };

        let display_message: extern fn(message: *const i8, _2: u8, _3: u32) =
            unsafe { std::mem::transmute(0x4_2CA40) };

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

            if current_gold > 0 && current_gold < 1337 {
                gold.write(1337);
                lumber.write(1337);
                oil.write(1337);

                display_message(c"Let's give you more resources!".as_ptr(), 8, 100);
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    });
}
