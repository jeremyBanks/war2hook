use {
    dll_syringe::{
        process::{OwnedProcess, Process},
        Syringe,
    },
    std::{thread, time},
};

fn main() {
    println!("Launching game");
    std::process::Command::new("D:\\Program Files\\Warcraft II\\Warcraft II BNE.exe")
        .spawn()
        .expect("failed to launch warcraft 2");

    thread::sleep(time::Duration::from_secs(2));

    // find target process by name
    let target_process = OwnedProcess::find_first_by_name("Warcraft II BNE.exe")
        .expect("failed to find WarCraft II process");

    let dll_path = "target\\i686-pc-windows-msvc\\debug\\war2injection.dll";

    let syringe = Syringe::for_process(target_process);

    println!("Injecting DLL");
    syringe.inject(dll_path).expect("failed to inject DLL");
}
