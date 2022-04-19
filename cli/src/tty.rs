use std::{
    ffi::OsString,
    fmt::Write,
    io::{Error, ErrorKind, Read},
};

type Line = Result<String, Error>;

pub struct SerialConnection {
    port: serial::SystemPort,
}

impl SerialConnection {
    pub fn new(tty: &OsString) -> Result<SerialConnection, serial::Error> {
        let term = SerialConnection {
            port: serial::open(tty)?,
        };

        Ok(term)
    }

    pub fn read_line(&mut self) -> Line {
        let mut buf = [0];
        let mut msg = String::new();

        loop {
            if self.port.read(&mut buf)? > 0 {
                let c = buf[0] as char;
                if c == '\n' {
                    return Ok(msg);
                } else {
                    msg.push(buf[0] as char)
                }
            } else {
                return Err(Error::from(ErrorKind::UnexpectedEof));
            }
        }
    }
}

impl Write for SerialConnection {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        std::io::Write::write_all(&mut self.port, s.as_bytes()).unwrap();
        Ok(())
    }
}

/// Simple wrapper for interacting with the benchmarking suite,
/// uses a serial connection and an iterator of input lines
/// to send them and retrieve the response
pub struct RawTerminal<'a> {
    connection: &'a mut SerialConnection,
    input_lines: Box<dyn Iterator<Item = Line>>,
    done: bool,
}

impl RawTerminal<'_> {
    pub fn new<'a, 'b>(
        connection: &'a mut SerialConnection,
        input_lines: Box<dyn Iterator<Item = Line>>,
    ) -> RawTerminal<'a> {
        RawTerminal {
            connection,
            input_lines,
            done: false,
        }
    }
}

impl Iterator for RawTerminal<'_> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.input_lines.next() {
            Some(Ok(input_line)) => {
                writeln!(self.connection, "{}", input_line).expect("Writing to serial failed");

                let ret = self.connection.read_line();

                self.done = ret.is_err();

                Some(ret)
            }
            x => x,
        }
    }
}
