use {
    eyre,
    iced_x86::{self, code_asm::CodeAssembler},
    std::{
        ffi::{c_char, CString},
        fs::OpenOptions,
        io::Write,
        mem::transmute,
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
        unsafe { transmute(0x4_2CA40) };

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

    writeln!(log, "assembling and installing hook").unwrap();

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

    writeln!(log, "installed hook!").unwrap();
}
