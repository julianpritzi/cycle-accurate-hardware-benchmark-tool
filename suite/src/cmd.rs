use core::{fmt::Display, num::ParseIntError};

use alloc::string::String;

use crate::platform::{self, Platform};

/// Executes a command
pub fn run_cmd(mut cmd: core::str::Split<char>) -> Result<Option<String>, CmdError> {
    match cmd.next() {
        Some("suspend") => platform::current().suspend(cmd.next().unwrap_or("0").parse()?),
        Some(_) => Err(CmdError("Unknown command")),
        None => Ok(None),
    }
}

/// Error type, representing an Error that ocurred during the execution of a command
pub struct CmdError(&'static str);

impl Display for CmdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

impl From<ParseIntError> for CmdError {
    fn from(_: ParseIntError) -> Self {
        CmdError("Failed to parse number")
    }
}
