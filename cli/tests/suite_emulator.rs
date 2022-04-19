use std::ffi::OsString;

/// Sets up an emulator
pub fn setup_emulator() -> OsString {
    if let Some(tty_path) = std::env::var_os("BENCHMARK_TEST_TTY") {
        tty_path
    } else {
        // TODO: mock an emulator
        panic!("Mocked emulator not yet implemented")
    }
}
