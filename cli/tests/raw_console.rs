use cli;

mod suite_emulator;

#[test]
pub fn basic_test() {
    let tty_path = suite_emulator::setup_emulator();

    let mut suite =
        cli::tty::SerialConnection::new(&tty_path).expect("Failed to connect to serial");

    let input_lines = vec!["suspend 0"];

    let mut raw_term = cli::tty::RawTerminal::new(
        &mut suite,
        Box::new(
            input_lines
                .into_iter()
                .map(|x| x.to_string())
                .map(|x| Ok(x)),
        ),
    );

    assert!(raw_term.next().unwrap().is_err());
    assert!(raw_term.next().is_none());
}
