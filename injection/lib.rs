use {
    crate::war2types::*,
    errors::try_or_die,
    eyre,
    iced_x86::{
        self,
        code_asm::{ax, CodeAssembler},
    },
    std::{
        cell::Cell,
        ffi::CString,
        mem::transmute,
        sync::{
            atomic::{self, AtomicU64},
            Mutex,
        },
        time::Instant,
    },
    windows::Win32::System::Memory::{VirtualProtect, PAGE_PROTECTION_FLAGS, PAGE_READWRITE},
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
            let state_s = format!("{state:?}");

            logln!("[{state_s:12}] {message_string}");

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
            wcprintln!("Handling 'day' cheat code.");

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

            wcprintln!(
                "Set all of your resources to 1337 and removed all of your opponent's resources."
            );

            let state = unsafe { GAME_STATE.get().read_volatile() };
            let race = unsafe { RACE.get().read_volatile() };
            wcprintln!("{state:?} {race:?}");
        })
    })
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
    asm: impl Fn(&mut CodeAssembler, &mut CodeAssembler) -> Result<(), eyre::Error>,
) -> Result<(), eyre::Error> {
    logln!("Assembling  patch for 0x{address:0X}.");

    let mut old_assembler = CodeAssembler::new(32)?;
    let mut new_assembler = CodeAssembler::new(32)?;

    asm(&mut old_assembler, &mut new_assembler)?;

    let assembled_old = old_assembler.assemble(u64::from(address))?;
    let assembled_new = new_assembler.assemble(u64::from(address))?;

    let assembled_old_len = assembled_old.len();
    let assembled_new_len = assembled_new.len();

    // TODO: we should allow the old to be longer than the new, and that's how
    // we deal with mis-alignment. As long as we know everything we're
    // replacing, it's acceptable.
    if assembled_old_len < assembled_new_len {
        logln!(
            "WARN: Assembled original instructions are shorter ({assembled_old_len}B) as \
             assembled new instructions ({assembled_new_len}B)."
        )
    }

    logln!("Patching {assembled_new_len:3} bytes at 0x{address:0X}.");

    let target_memory: &mut [u8] =
        std::slice::from_raw_parts_mut(transmute(address), assembled_new_len);

    if target_memory != &assembled_old[..assembled_new_len] {
        logln!(
            "WARN: Assembled original instructions did not match actual instructions in memory."
        );
        logln!("  actual memory: {}", hex::encode(&target_memory));
        logln!("  assembled old: {}", hex::encode(&assembled_old));
        logln!("  assembled new: {}", hex::encode(&assembled_new));
    }

    let mut original_flags = PAGE_PROTECTION_FLAGS(0);
    VirtualProtect(
        transmute(address),
        assembled_new.len(),
        PAGE_READWRITE,
        &mut original_flags,
    )?;
    target_memory.copy_from_slice(&assembled_new);
    VirtualProtect(
        transmute(address),
        assembled_new.len(),
        original_flags,
        &mut original_flags,
    )?;

    logln!("Patched  {assembled_new_len:3} bytes at 0x{address:0X}.");

    Ok(())
}

pub fn install() -> Result<(), eyre::Error> {
    logln!("Assembling and installing hooks.");

    use iced_x86::code_asm::{eax, ebp, esi, esp};

    // This hook is very near <SOMETHING, I'M NOW CONFUSED WHAT>.
    // The hook must call `GAME_STATE_TRANSITION_TARGET()` to restore the
    // function call we're overwriting with the hook.
    logln!("Patching game state transition.");
    unsafe {
        patch_asm(0x4_2A343, |old, new| {
            old.call(*GAME_STATE_TRANSITION_TARGET as u64)?;

            new.call(hook as u64)?;

            extern fn hook() {
                try_or_die(|| {
                    Ok({
                        static LAST_TRANSITION: Mutex<Option<Instant>> = Mutex::new(None);

                        let mut last_transition = LAST_TRANSITION.lock().unwrap();
                        let now = Instant::now();
                        let state = unsafe { GAME_STATE.get().read_volatile() };
                        let state = format!("{state:?}");

                        if let Some(last_transition) = *last_transition {
                            let elapsed = now - last_transition;

                            let seconds = elapsed.as_secs();
                            let minutes = seconds / 60;
                            let seconds = seconds % 60;
                            let millis = elapsed.subsec_millis();

                            logln!("[{state:12}] after {minutes:2}m {seconds:2}.{millis:03}s");
                        } else {
                            logln!("[{state:12}]");
                        }

                        *last_transition = Some(now);
                    })
                });

                GAME_STATE_TRANSITION_TARGET();
            }

            Ok(())
        })?;
    }

    // Address near the beginning of the 'day' cheat code branch inside the
    // function that applies cheat codes, but after it resets the cheat flags,
    // so that it works every time instead of toggling.
    logln!("Patching \"day\" cheat code.");
    unsafe {
        patch_asm(0x4_160AD, |old, new| {
            old.call(0x4_20480)?;
            old.cmp(ax, 3)?;
            old.jnz(0x4_1622A)?;
            old.pop(esi)?;
            old.pop(ebp)?;
            old.add(esp, 0x80)?;
            old.ret()?;

            new.call(day_cheat_hook as u64)?;
            new.pop(esi)?;
            new.pop(ebp)?;
            new.add(esp, 0x80)?;
            new.ret()?;

            Ok(())
        })?;
    }

    println!("Hooks installed.");

    Ok(())
}
