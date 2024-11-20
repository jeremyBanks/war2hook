use {
    dll_syringe::{
        process::{OwnedProcess, Process},
        Syringe,
    },
    std::{thread, time},
};

fn main() {
    println!("Launching game");
    let mut process =
        std::process::Command::new("D:\\Program Files\\Warcraft II\\Warcraft II BNE.exe")
            .spawn()
            .expect("failed to launch warcraft 2");

    thread::sleep(time::Duration::from_secs(6));

    // find target process by name
    let target_process = OwnedProcess::find_first_by_name("Warcraft II BNE.exe")
        .expect("failed to find WarCraft II process");

    let dll_path = {
        if target_process.is_x64().unwrap() {
            "target\\x86_64-pc-windows-msvc\\debug\\war2hook.dll"
        } else {
            "target\\i686-pc-windows-msvc\\debug\\war2hook.dll"
        }
    };

    let syringe = Syringe::for_process(target_process);

    println!("Injecting DLL");
    let injected_payload = syringe.inject(dll_path).expect("failed to inject DLL");
}
