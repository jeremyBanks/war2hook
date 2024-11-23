use {
    crate::war2types::*,
    errors::try_or_die,
    eyre,
    iced_x86::{self, code_asm::CodeAssembler},
    std::{
        ffi::CString,
        mem::transmute,
        sync::{atomic, atomic::AtomicU64},
    },
    windows::Win32::System::Memory::{
        VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
    },
};

mod dllmain;
mod errors;
mod logging;
mod war2types;

/// Prints a message to the log file and to the in-game text output.
macro_rules! wcprintln {
    ($($arg:tt)*) => {
        {
            let message_string = format!($($arg)*);

            let state = unsafe { GAME_STATE.get().read_volatile() };

            logln!("[{state:?}] {message_string}");

            if GameState::InGame == state {
                let message_cstring = CString::new(message_string).unwrap_or(c"<unable to encode as CString>".into());
                let message_pointer = message_cstring.as_ptr();
                DISPLAY_MESSAGE(message_pointer, 7, 0);
            }
        }
    };
}
/// This hook replaces the default behaviour of the `day` cheat code.
extern fn day_cheat_hook() {
    try_or_die(|| {
        Ok({
            wcprintln!("handling 'day' cheat code");

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

            wcprintln!("race: {:?}", unsafe { RACE.get().read_volatile() });

            wcprintln!(
                "Set all of your resources to 1337 and removed all of your opponent's resources."
            );

            let state = unsafe { GAME_STATE.get().read_volatile() };
            wcprintln!("game state: {state:?}");
        })
    })
}

/// The hook runs at the beginning of the main game loop.
extern fn game_state_transition_hook() {
    logln!("game state transition?");
    try_or_die(|| {
        Ok({
            logln!("getting state. it's probably something we don't have programmed in...");
            let state = unsafe { GAME_STATE.get().read_volatile() };
            logln!("[{state:?}]");
        })
    });
    GAME_STATE_TRANSITION_TARGET();
}

extern fn main_loop_hook() {
    static MAIN_LOOP_TICKS: AtomicU64 = AtomicU64::new(0);

    try_or_die(|| {
        Ok({
            let ticks = MAIN_LOOP_TICKS.fetch_add(1, atomic::Ordering::Acquire);
            if ticks % 1 == 0 {
                logln!("Main loop hook tick {ticks}.");
            }
        })
    });
}

unsafe fn patch_asm(
    address: u32,
    asm: impl Fn(&mut CodeAssembler) -> Result<(), eyre::Error>,
) -> Result<(), eyre::Error> {
    logln!("Assembling  patch for 0x{address:0X}.");

    let mut assembler = CodeAssembler::new(32)?;

    asm(&mut assembler)?;

    let assembled = assembler.assemble(u64::from(address))?;

    let assembled_len = assembled.len();

    logln!("Patching {assembled_len:3} bytes at 0x{address:0X}.");

    let target_memory: &mut [u8] =
        std::slice::from_raw_parts_mut(transmute(address), assembled_len);

    VirtualProtect(
        transmute(address),
        assembled.len(),
        PAGE_EXECUTE_READWRITE,
        &mut PAGE_PROTECTION_FLAGS(0),
    )?;

    target_memory.copy_from_slice(&assembled);

    logln!("Patched  {assembled_len:3} bytes at 0x{address:0X}.");

    Ok(())
}

pub fn install() -> Result<(), eyre::Error> {
    logln!("Assembling and installing hooks.");

    use iced_x86::code_asm::{ebp, esi, esp};

    // This hook is very near <SOMETHING, I'M NOW CONFUSED WHAT>.
    // The hook must call `GAME_STATE_TRANSITION_TARGET()` to restore the
    // function call we're overwriting with the hook.
    logln!("Patching game state transition.");
    unsafe {
        patch_asm(0x4_2A343, |asm| {
            asm.call(game_state_transition_hook as u64)?;

            Ok(())
        })?;
    }

    // Address near the beginning of the 'day' cheat code branch inside the
    // function that applies cheat codes, but after it resets the cheat flags,
    // so that it works every time instead of toggling.
    logln!("Patching \"day\" cheat code.");
    unsafe {
        patch_asm(0x4_160AD, |asm| {
            // iced_x86 expects a u64 for absolute addresses, even though
            // this program and the assembler are both targeting 32-bit.
            asm.call(day_cheat_hook as u64)?;

            // After calling our function, we immediately return from the patched
            // function to avoid running any subsequent instructions (which may
            // no longer even decode as real instructions, since we may have just
            // overwritten the first bytes of one and screwed up their alignment).
            // We copied these instructions from other returns in the function.
            //
            // Restore two registers, which have been used for local variables in
            // this function, to their values from before the function was called.
            asm.pop(esi)?;
            asm.pop(ebp)?;
            // Adjust the stack to remove 128 bytes that had been allocated for
            // a too-large-for-registers local variable within the function.
            asm.add(esp, 0x80)?;
            // Return
            asm.ret()?;

            Ok(())
        })?;
    }

    println!("Hooks installed.");

    Ok(())
}
