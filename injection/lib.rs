use {
    crate::war2types::*,
    eyre,
    iced_x86::{self, code_asm::CodeAssembler},
    std::{
        ffi::{c_char, CString},
        fs::{File, OpenOptions},
        io::Write,
        mem::transmute,
        ptr::NonNull,
        sync::{LazyLock, Mutex},
        time::Duration,
    },
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

mod war2types;

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

/// Prints a message to the log file and to the in-game text output.
macro_rules! wcprintln {
    ($($arg:tt)*) => {
        {
            let date = chrono::Utc::now();
            let date = date.format("%H:%M:%S%.3f");

            let message_string = format!($($arg)*);
            writeln!(LOG_FILE.lock().unwrap(), "{date} {message_string}").unwrap();
            let message_cstring = CString::new(message_string).unwrap();
            let message_pointer = message_cstring.as_ptr();
            DISPLAY_MESSAGE(message_pointer, 0, 0);
        }
    };
}

static LOG_FILE: LazyLock<Mutex<File>> = LazyLock::new(|| {
    let date = chrono::Utc::now();
    let date = date.format("%Y-%M-%d-%H");

    let log_path = format!("C:\\Users\\_\\war2hook\\logs\\{date}.log");

    Mutex::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(log_path)
            .expect("Unable to open log file"),
    )
});

extern fn apply_cheats_hook() {
    wcprintln!("handling 'day' cheat code");

    let current_gold = unsafe { PLAYERS_GOLD.get().read()[0] };
    let current_lumber = unsafe { PLAYERS_LUMBER.get().read()[0] };
    let current_oil = unsafe { PLAYERS_OIL.get().read()[0] };

    unsafe {
        PLAYERS_GOLD
            .get()
            .write_volatile([1337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        PLAYERS_LUMBER
            .get()
            .write_volatile([1337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        PLAYERS_OIL
            .get()
            .write_volatile([1337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    wcprintln!("Set all of your resources to 1337 and removed all of your opponent's resources.");

    let state = unsafe { GAME_STATE.get().read_volatile() };
    wcprintln!("game state: {state:?}");
}

fn attach() {
    let date = chrono::Utc::now();
    let date = date.format("%H:%M:%S%.3f [attaching]");

    let mut log = LOG_FILE.lock().unwrap();

    writeln!(log, "{date} assembling and installing hooks").unwrap();

    let hook_function_address = apply_cheats_hook as u32;

    // Address at the beginning of the 'day' cheat code branch inside the
    // function that applies cheat codes.
    let replacement_address: u32 = 0x4_160A4;

    // The instructions we're going to be putting in that branch instead,
    // to call our hook function instead of the default behavior.
    let replacement_instructions = {
        use iced_x86::code_asm::*;

        let mut asm = CodeAssembler::new(32).unwrap();

        // iced_x86 expects a u64 for absolute addresses, even though
        // this program and the assembler are both targeting 32-bit.
        asm.call(u64::from(hook_function_address)).unwrap();

        // After calling our function, we immediately return from the patched
        // function to avoid running any subsequent instructions (which may
        // no longer even decode as real instructions, since we may have just
        // overwritten the first bytes of one and screwed up their alignment).
        // We copied these instructions from other returns in the function.
        //
        // Restore two registers, which have been used for local variables in
        // this function, to their values from before the function was called.
        asm.pop(esi).unwrap();
        asm.pop(ebp).unwrap();
        // Adjust the stack to remove 128 bytes that had been allocated for
        // a too-large-for-registers local variable within the function.
        asm.add(esp, 0x80).unwrap();
        // Return
        asm.ret().unwrap();

        asm.assemble(u64::from(replacement_address)).unwrap()
    };

    // Slice of the memory we're overwriting.
    let replacement_slice: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(
            transmute(replacement_address),
            replacement_instructions.len(),
        )
    };

    // Remove read-only protection from the target memory, which the original
    // compiler applied automatically to executable memory/instructions.
    unsafe {
        VirtualProtect(
            transmute(replacement_address),
            replacement_instructions.len(),
            PAGE_EXECUTE_READWRITE,
            &mut PAGE_PROTECTION_FLAGS(0),
        )
        .unwrap();
    }

    // Apply the change.
    replacement_slice.copy_from_slice(&replacement_instructions);

    writeln!(log, "{date} attach(): hook installed").unwrap();
}
