### Objective

Run my own Rust code inside of the Warcraft II process, hooking into game
functions, calling game functions, and manipulating game state.

### Current Progress

The `dll-syringe` crate made it shockingly easy to get my own code running
inside of the Warcraft II process. I _think_ I was able to modify memory from
there by using a raw pointer (by looking at the memory in Cheat Engine and
seeing that it had changed). However, I only changed an arbitrarily-chosen
dynamically-allocated value with no visible effect. The one I'm trying to change
is statically allocated. Cheat Engine doesn't give me statically allocated
addresses directly, but relative to an "image base" offset, which I didn't know,
so my attempts to modify those values wrote to invalid addresses (non-mapped
pages, I think) and crashed the program.

### Planned Approach

- Use Cheat Engine to determine memory addresses of data structures and
  functions of note. This seems to be a widely-used tool for such purposes. It
  allows you to scan memory for specific values, or for values that are changing
  according to certain patterns, and modify them (like GameShark) so you can
  confirm the effect they have on the running program. But it also includes a
  remote debugger that can attach to arbitrary processes find and disassemble
  the instructions that are accessing specific memory addresses.
- Use the `exe` crate (or alternatives) to read metadata from the Warcraft II
  executable file, in order to determine the "image base" address, which IIUC is
  an offset into the process's virtual memory where the statically-allocated
  memory (mapped from the executable file) starts. Cheat Engine shows us
  addresses relative to this offset for static allocations, so we need its
  value, but it will never change so we could just hard-code it once known.
- Write some Rust FFI definitions for those data structures, and utilities to
  calculate those function and data structure addresses of interest relative to
  an "image base" offset. Write some simple function to manipulate game state in
  a conspicuous way (e.g. set Gold to 1337 once per second) so we can see when
  we're able to get it to run.
- Compile my Rust logic into a DLL, which will, when attached to a running
  Warcraft II process, patch Warcraft II functions in memory to add calls to
  Rust functions in the DLL to create the hooks we need.
  - This DLL will export a `DllMain` FFI function, which is how we're able to
    run code when it's attached to the process.
  - We'll use the `iced-x86` crate to generate the x86 machine code for those
    hooks. (It'll probably be just a few instructions we could write by hand,
    but this will probably come in handy for other purposes later.)
  - I'm hoping we can just take function pointers to C FFI functions inside the
    DLL and use those as jump addresses, but there may be other' complications.
  - The DLL will need to be built for 32-bit Windows because it's being injected
    into a 32-bit process, but that's just a matter of specifying a simple flag
    to the Rust compiler.
- Create a smaller launcher program which launches Warcraft II, then uses
  Windows APIs to inject the DLL into a running Warcraft II process, either
  directly via the low-level `windows` crate, or via the higher-level
  `dll-syringe` crate.

### Resources

- https://samrambles.com/guides/window-hacking-with-rust/injecting-dlls-with-rust/
- https://fluxsec.red/remote-process-dll-injection
- https://github.com/cheat-engine/cheat-engine
- https://crates.io/crates/dll-syringe
- https://crates.io/crates/exe
- https://crates.io/crates/windows
- https://crates.io/crates/iced-x86
