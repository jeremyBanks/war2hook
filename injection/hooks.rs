use {
    crate::{
        logln, wcprintln, wcstatus, GAME_STATE, PLAYERS_GOLD, PLAYERS_LUMBER, PLAYERS_OIL, RACE,
    },
    std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            Mutex,
        },
        time::{Duration, Instant},
    },
};

pub fn instead_of_day_cheat() -> Result<(), eyre::Error> {
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

    wcprintln!("Set all of your resources to 1337 and removed all of your opponent's resources.");

    let state = unsafe { GAME_STATE.get().read_volatile() };
    let race = unsafe { RACE.get().read_volatile() };
    wcprintln!("{state:?} {race:?}");

    Ok(())
}

#[derive(Debug, Clone)]
enum TimerState {
    WaitingToStart,
    Running { started: Instant, ticks: u64 },
    Completed { duration: Duration, ticks: u64 },
}

static TIMER_STATE: Mutex<TimerState> = Mutex::new(TimerState::WaitingToStart);

pub fn on_game_state_transition() -> Result<(), eyre::Error> {
    let mut timer_state = TIMER_STATE.lock().unwrap();

    *timer_state = TimerState::WaitingToStart;

    Ok(())
}

pub fn after_game_tick() -> Result<(), eyre::Error> {
    let mut timer_state = TIMER_STATE.lock().unwrap();

    match *timer_state {
        TimerState::WaitingToStart => {
            *timer_state = TimerState::Running {
                started: Instant::now(),
                ticks: 0,
            };

            wcstatus!(" 0m  0s");
        },
        TimerState::Running { started, mut ticks } => {
            let duration = Instant::now() - started;
            ticks += 1;

            let seconds = duration.as_secs();
            let minutes = seconds / 60;
            let seconds = seconds % 60;

            wcstatus!("{minutes:2}m {seconds:2}s");

            *timer_state = TimerState::Running { started, ticks }
        },
        TimerState::Completed { duration, ticks } => {
            let seconds = duration.as_secs();
            let minutes = seconds / 60;
            let seconds = seconds % 60;
            let millis = duration.subsec_millis();

            wcstatus!("{minutes}m {seconds}s {millis}ms ({ticks} ticks)");
        },
    }
    Ok(())
}

pub fn before_victory_dialog() -> Result<(), eyre::Error> {
    let mut timer_state = TIMER_STATE.lock().unwrap();

    match *timer_state {
        TimerState::Running { started, ticks } => {
            let duration = Instant::now() - started;
            *timer_state = TimerState::Completed { duration, ticks };

            let seconds = duration.as_secs();
            let minutes = seconds / 60;
            let seconds = seconds % 60;
            let millis = duration.subsec_millis();

            wcstatus!("{minutes}m {seconds}s {millis}ms ({ticks} ticks)");
        },
        _ => {},
    }

    Ok(())
}
