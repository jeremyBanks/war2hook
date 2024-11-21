use {
    bstr::BStr,
    eyre,
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
            System::{
                Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
                SystemServices::*,
                Threading::GetCurrentProcessId,
            },
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
extern fn apply_cheats_hook() {
    let mut log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("C:\\Users\\_\\war2hook\\main-thread.log")
        .expect("Unable to open log file");

    writeln!(log, "you did it! it worked!").unwrap();

    let gold = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABB18 as *mut u32)) };
    let lumber = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ACB6C as *mut u32)) };
    let oil = unsafe { VolatilePtr::new(NonNull::new_unchecked(0x4_ABBFC as *mut u32)) };

    let display_message: extern fn(message: *const i8, _2: u8, _3: u32) =
        unsafe { std::mem::transmute(0x4_2CA40) };

    let current_gold = gold.read();
    let current_lumber = lumber.read();
    let current_oil = oil.read();

    let line = format!("gold: {current_gold}, lumber: {current_lumber}, oil: {current_oil}\n");

    log.write_all(line.as_bytes()).unwrap();

    display_message(c"Let's give you some resources!".as_ptr(), 8, 100);

    gold.write(1337);
    lumber.write(1337);
    oil.write(1337);
}

fn attach() {
    let mut log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("C:\\Users\\_\\war2hook\\attachment-thread.log")
        .expect("Unable tzo open log file");

    // TODO: eyre for ?, and catch panic and log it

    writeln!(log, "assembling hook!").unwrap();
    let call_hook = {
        use iced_x86::code_asm::*;

        let mut a = CodeAssembler::new(32).unwrap();

        // iced_x86 expects a u64 for this absolute address, even though
        // this program and the assembler are both targeting 32-bit.
        a.call(apply_cheats_hook as u64).unwrap();
        a.pop(esi).unwrap();
        a.pop(ebp).unwrap();
        a.add(esp, 0x80).unwrap();
        a.ret().unwrap();

        a.assemble(0x4_160A4).unwrap()
    };

    writeln!(log, "assembled hook call:    {}", hex::encode(&call_hook)).unwrap();

    // We're hooking the "day" cheat code, by overwriting the
    // instructions at 0x4_160a4 with our assembled bytes. It returns,
    // so we don't have to worry about corrupting the subsequent
    // instructions if ours doesn't align.
    let target: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(0x4_160A4), call_hook.len()) };

    writeln!(log, "replacing at 0x4_160A4: {}", hex::encode(&target)).unwrap();

    writeln!(log, "disabling memory protection!").unwrap();

    unsafe {
        VirtualProtect(
            std::mem::transmute(0x4_160A4),
            call_hook.len(),
            PAGE_EXECUTE_READWRITE,
            &mut PAGE_PROTECTION_FLAGS(0),
        )
        .unwrap();
    }

    writeln!(log, "installing hook!").unwrap();

    target.copy_from_slice(&call_hook);

    writeln!(log, "installed hook!").unwrap();
}
