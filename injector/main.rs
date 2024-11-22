use {
    dll_syringe::{
        process::{OwnedProcess, Process},
        Syringe,
    },
    std::{
        ffi::{c_char, CString},
        fs::{File, OpenOptions},
        io::Write,
        mem::transmute,
        ptr::NonNull,
        sync::{LazyLock, Mutex},
        thread,
        time::{self, Duration},
    },
};

fn main() {
    println!("Launching WarCraft II");

    std::process::Command::new("D:\\Program Files\\Warcraft II\\Warcraft II BNE.exe")
        .args(["tigerlily", "human3"])
        .spawn()
        .expect("Failed to launch WarCraft II");

    thread::sleep(time::Duration::from_secs(2));

    // instead of trying once, loop rapidly so we find the process quickly to
    // inject it early.
    let target_process: OwnedProcess;

    loop {
        if let Some(process) = OwnedProcess::find_first_by_name("Warcraft II BNE.exe") {
            target_process = process;
            break;
        }
    }

    println!("Found WarCraft II process.");

    let dll_path = "target\\i686-pc-windows-msvc\\debug\\war2injection.dll";

    let syringe = Syringe::for_process(target_process);

    println!("Injecting DLL.");

    syringe.inject(dll_path).expect("Failed to inject DLL");

    println!("DLL Injected");
}
