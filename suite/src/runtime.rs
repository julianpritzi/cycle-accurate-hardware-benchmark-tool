//! Contains functions and macros for providing a runtime environment to the bechmarking suite
use core::panic::PanicInfo;

use crate::platform::{self, Platform};

/// Initializes the communication module and enables the use of the print & println macro
///
/// # Safety
///  - only call once
pub unsafe fn init() -> Result<(), &'static str> {
    // Safety:
    // This should be the first time the communication module is accessed,
    // invalidating previous references is ok
    let comm = platform::current().get_communication_module();
    comm.init()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        // Safety: invalidating previous references is ok, since we are in a unrecoverable state
        let comm = platform::current().get_communication_module();
        if comm.init().is_ok() {
            let _ = writeln!(comm, "! {}", info);
        }
    }

    platform::current().suspend(101)
}

// Safety of calling get_communication_module() inside the macros:
// invalidating previous references is ok,
// because all macros reference the module only in a closed scope
// and the architecture is assumed to be on a single core

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (unsafe {
        use crate::platform::Platform;
        write!($crate::platform::current().get_communication_module(), $($arg)*).unwrap();
    });
}

#[macro_export]
macro_rules! println {
    () => (unsafe {
        use crate::platform::Platform;
        writeln!($crate::platform::current().get_communication_module()).unwrap();
    });
    ($($arg:tt)*) => (unsafe {
        use crate::platform::Platform;
        writeln!($crate::platform::current().get_communication_module(), $($arg)*).unwrap();
    });
}
