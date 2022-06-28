use std::{
    ffi::OsString,
    fmt::Write,
    io::{BufRead, BufReader, BufWriter, Error},
    time::Duration,
};

use benchmark_common::{deserialize, serialize, IncomingMessage, OutgoingMessage, SuiteStatus};
use serialport::TTYPort;

type Line = Result<String, Error>;

/// SerialConnection, representing a connection over a serial TTYPort
pub struct SerialConnection {
    writer: BufWriter<TTYPort>,
    reader: BufReader<TTYPort>,
}

impl SerialConnection {
    /// Creates a new SerialConnection connected to the provided tty
    ///
    /// # Arguments
    ///
    /// * `tty` - the path to the tty to connect to
    pub fn new(tty: &OsString) -> Result<SerialConnection, serialport::Error> {
        let port = serialport::new(tty.to_string_lossy(), 115200)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .timeout(Duration::from_secs(60))
            .open_native()?;
        let term = SerialConnection {
            reader: BufReader::new(port.try_clone_native().expect("Failed to clone port")),
            writer: BufWriter::new(port),
        };

        Ok(term)
    }

    /// Reads a single line form the serial port
    pub fn read_line(&mut self) -> Line {
        let mut buf = vec![];

        self.reader.read_until(0xA, &mut buf)?;

        Ok(String::from_utf8_lossy(&buf).trim().to_string())
    }
}

impl Write for SerialConnection {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if std::io::Write::write(&mut self.writer, s.as_bytes()).is_err()
            || std::io::Write::flush(&mut self.writer).is_err()
        {
            Err(std::fmt::Error)
        } else {
            Ok(())
        }
    }
}

/// SuiteConnection, representing a connection to a benchmarking suite instance
///
/// Connection is realized over a serial port, and communication is done using
/// messages.
///
/// For information about supported messages an their (de)serialization check the common create.

pub struct SuiteConnection {
    serial: SerialConnection,
    verbose: bool,
}

impl SuiteConnection {
    /// Establish a new SuiteConnection over the provided SerialConnection,
    /// fails if any errors occur during communication using the SerialConnection.
    ///
    /// # Arguments
    ///
    /// * `serial` - the serial connection to use to communicate with the suite
    pub fn new(serial: SerialConnection, verbose: bool) -> Result<SuiteConnection, std::io::Error> {
        let mut conn = SuiteConnection { serial, verbose };

        if verbose {
            println!("New Connection.");
        }
        conn.send_message(&OutgoingMessage::GetStatus);

        loop {
            match conn.read_message() {
                Ok(msg) => {
                    if matches!(msg, IncomingMessage::Status(SuiteStatus::Ready)) {
                        return Ok(conn);
                    }
                }
                Err(x) => return Err(x),
            }
        }
    }

    /// Read a message sent by the suite,
    /// fails if any errors occur during communication using the SerialConnection.
    pub fn read_message(&mut self) -> Result<IncomingMessage, std::io::Error> {
        let msg = deserialize(self.serial.read_line()?);
        if self.verbose {
            println!("< {msg:?}");
        }
        Ok(msg)
    }

    /// Send a message to the suite
    ///
    /// # Arguments
    ///
    /// * `msg` - the message that should be sent to the suite
    pub fn send_message(&mut self, msg: &OutgoingMessage) {
        if self.verbose {
            println!("> {msg:?}");
        }
        writeln!(self.serial, "{}", serialize(msg)).expect("Failed to write to serial");
    }
}
