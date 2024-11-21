use std::mem::transmute;

static display_message: extern fn(message: *const i8, _2: u8, _3: u32) =
    unsafe { transmute(0x4_2CA40) };
