use {
    crate::war2types::*,
    errors::try_or_die,
    eyre,
    hooks::{
        after_game_tick, before_victory_dialog, instead_of_day_cheat, on_game_state_transition,
    },
    iced_x86::{
        self,
        code_asm::{ax, ebp, esi, esp, CodeAssembler},
    },
    std::{
        ffi::CString,
        mem::transmute,
        sync::{
            atomic::{self, AtomicU64, Ordering},
            Mutex,
        },
        time::Instant,
    },
    windows::Win32::System::Memory::{VirtualProtect, PAGE_PROTECTION_FLAGS, PAGE_READWRITE},
};

mod dllmain;
mod errors;
mod hooks;
mod logging;
mod war2types;

pub fn install() -> Result<(), eyre::Error> {
    logln!("Assembling and installing hooks.");

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

            new.call(hook as u64)?;
            new.pop(esi)?;
            new.pop(ebp)?;
            new.add(esp, 0x80)?;
            new.ret()?;

            extern fn hook() {
                try_or_die(instead_of_day_cheat)
            }

            Ok(())
        })?;
    }

    logln!("Patching game state transition.");
    unsafe {
        patch_asm(0x4_2A343, |old, new| {
            old.call(*IMG_UPDATE as u64)?;

            new.call(hook as u64)?;

            extern fn hook() {
                try_or_die(on_game_state_transition);
                IMG_UPDATE();
            }

            Ok(())
        })?;
    }

    logln!("Patching end of game tick.");
    unsafe {
        patch_asm(0x4_212EE, |old, new| {
            // load game state into AX register through built-in function
            old.call(0x4_20480)?;

            // load game state into AX register through our hook function
            new.call(hook as u64)?;

            extern fn hook() -> GameState {
                try_or_die(after_game_tick);
                unsafe { GAME_STATE.get().read_volatile() }
            }

            Ok(())
        })?;
    }

    logln!("Patching before victory dialog.");
    unsafe {
        patch_asm(0x4_59AC5, |old, new| {
            old.call(*GAME_EVENT_RESET as u64)?;

            new.call(hook as u64)?;

            extern fn hook() {
                try_or_die(before_victory_dialog);
                GAME_EVENT_RESET();
            }

            Ok(())
        })?;
    }

    logln!("Hooks installed.");

    Ok(())
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
    let mut assembled_new = new_assembler.assemble(u64::from(address))?;

    let assembled_len = assembled_old.len();
    let assembled_new_len = assembled_new.len();

    if assembled_len < assembled_new_len {
        logln!(
            "WARN: Assembled original instructions are shorter ({assembled_len}B) as assembled \
             new instructions ({assembled_new_len}B)."
        )
    }

    // Pad it with NOPs.
    assembled_new.resize(assembled_len, 0);

    logln!("Patching {assembled_len:3} bytes at 0x{address:0X}.");

    let target_memory: &mut [u8] =
        std::slice::from_raw_parts_mut(transmute(address), assembled_len);

    if target_memory != &assembled_old {
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
        assembled_len,
        PAGE_READWRITE,
        &mut original_flags,
    )?;
    target_memory.copy_from_slice(&assembled_new);
    VirtualProtect(
        transmute(address),
        assembled_len,
        original_flags,
        &mut original_flags,
    )?;

    logln!("Patched  {assembled_len:3} bytes at 0x{address:0X}.");

    Ok(())
}
