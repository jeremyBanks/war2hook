#![allow(unused)]

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
                    *mut $signature
                >
            > = unsafe {
                ::std::sync::LazyLock::new(||
                    ::fragile::Fragile::new(
                        ::core::mem::transmute($address)
                    )
                )
            };
        )+
    };
}

/// The maximum number of players in the game is actually 8,
/// but most of the code seems to be written for 16 instead.
pub const MAX_PLAYERS: usize = 16;
pub const MAX_HUMAN_PLAYERS: u8 = 8;

fn_by_address! {
    /// Displays a message at the bottom of the game screen.
    ///
    /// `playerIndex` may be the number of a player in the game, which will
    /// prefix their name (for a chat message), or it may be
    /// `MAX_HUMAN_PLAYERS` for a non-prefixed system message.
    /// However, only one system message can be displayed at time.
    ///
    /// `duration` controls how long the message is displayed.
    pub DISPLAY_MESSAGE: extern fn(message: *const i8, playerNumber: u8, duration: u32) = 0x4_2CA40;

    pub GAME_STATE_TRANSITION_TARGET: extern fn() = 0x4_051D0;
}

data_by_address! {
    pub PLAYERS_GOLD:   [u32; MAX_PLAYERS] = 0x4_ABB18;
    pub PLAYERS_LUMBER: [u32; MAX_PLAYERS] = 0x4_ACB6C;
    pub PLAYERS_OIL:    [u32; MAX_PLAYERS] = 0x4_ABBFC;
    pub GAME_STATE:     GameState          = 0x4_AE480;
    pub GAME_SPEED:     u32                = 0x4_D6B38;
    pub RACE:           Race               = 0x4_ABB7C;
}

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum GameState {
    UnknownGameState0 = 0,
    UnknownGameState1 = 1,
    Exit = 2,
    InGame = 3,
    MainMenu = 4,
    UnknownGameState5 = 5,
    VictoryScreen = 6,
    DefeatScreen = 7,
    UnknownGameState9 = 9,
    UnknownGameState10 = 10,
    UnknownGameState11 = 11,
    UnknownGameState12 = 12,
    UnknownGameState13 = 13,
    UnknownGameState14 = 14,
    UnknownGameState15 = 15,
    UnknownGameState16 = 16,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Race {
    Human = 0,
    Orc = 1,
}
