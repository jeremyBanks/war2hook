macro_rules! fn_by_address {
    // We _should_ be able to declare functions statically, not require LazyLock
    // runtime initialization, but the Rust compiler rejects it as unsound.
    // See https://github.com/rust-lang/rust/issues/63359 for some details.
    // I think the compiler is correct when it applies this check in most
    // cases, but I think that if we're creating a function pointer, not
    // a reference to data, what we're doing should be sound. (And if it's
    // not sound, doing this at runtime instead of statically won't help.)

    ($(
        $(#[$($attrss:tt)*])*
        $vis:vis
        $name:ident: $signature:ty = $address:literal;
    )+) => {
        $(
            $(#[$($attrss)*])*
            #[allow(non_upper_case_globals)]
            $vis static $name: ::std::sync::LazyLock<$signature> =
                ::std::sync::LazyLock::new(||
                    unsafe { ::core::mem::transmute($address)
                });
        )+
    };
}

macro_rules! data_by_address {
    // This stack of types attempts to make accessing this data as safe as
    // possible, although it still definitely falls short of actually being
    // safe by Rust's standards.
    //
    // Fragile enforces that will only every access it from a single thread,
    // and VolatilePtr should prevent the compiler from inappropriately
    // optimizing out loads and stores. However, it's still possible that the
    // data could be changed from under us if we call back into the game's own
    // code while we're referencing it, and we do nothing to prevent unsafe
    // conditions like that.

    ($(
        $(#[$($attrss:tt)*])*
        $vis:vis
        $name:ident: $signature:ty = $address:literal;
    )+) => {
        $(
            $(#[$($attrss)*])*
            $vis static $name: ::std::sync::LazyLock<
                ::fragile::Fragile<
                    ::volatile::VolatilePtr<
                        $signature
                    >
                >
            > = unsafe {
                ::std::sync::LazyLock::new(||
                    ::fragile::Fragile::new(
                        ::volatile::VolatilePtr::new(
                            ::core::ptr::NonNull::new_unchecked(
                                ::core::mem::transmute($address)
                            )
                        )
                    )
                )
            };
        )+
    };
}

fn_by_address! {
    /// Displays a message at the bottom of the game screen, such as for chat.
    pub display_message: extern fn(message: *const i8, _2: u8, _3: u32) = 0x4_2CA40;
}

data_by_address! {
    pub PLAYER_1_GOLD: u32 = 0x4_ABB18;
}
