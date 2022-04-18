use std::{
    ffi::OsString,
    fmt::Write,
    io::{Error, ErrorKind, Read},
};

pub struct SerialTerminal {
    port: serial::SystemPort,
}

impl SerialTerminal {
    pub fn new(tty: &OsString) -> Result<SerialTerminal, serial::Error> {
        let term = SerialTerminal {
            port: serial::open(tty)?,
        };

        Ok(term)
    }

    pub fn read_line(&mut self) -> Result<String, Error> {
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

impl Write for SerialTerminal {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        std::io::Write::write_all(&mut self.port, s.as_bytes()).unwrap();
        Ok(())
    }
}
