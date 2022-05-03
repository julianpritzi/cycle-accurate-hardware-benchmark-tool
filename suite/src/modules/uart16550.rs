use core::fmt::Write;

use crate::modules::{ByteRead, Module};
use bitflags::bitflags;

bitflags! {
    /// Abstract representation of the status registers flags.
    struct StatusFlags: u8 {
        const INPUT_FULL = 1;
        const OUTPUT_EMPTY = 1 << 5;
    }
}

/// Uart Driver implementation, that does not rely on atomic operations
pub struct Uart16550 {
    initialized: bool,
    base_address: *mut u8,
}

impl Uart16550 {
    /// Creates a new Uart16550 driver at the given base address
    ///
    /// # Arguments
    ///
    /// * `base_address` - A pointer to the MMIO address of the uart device
    ///
    /// # Safety:
    ///  - a valid uart device must be at the base_address
    ///  - no other uart must use the same base_address
    pub const unsafe fn new(base_address: *mut u8) -> Uart16550 {
        Uart16550 {
            initialized: false,
            base_address,
        }
    }

    /// Returns abstract representation of the status register.
    fn status(&self) -> StatusFlags {
        unsafe { StatusFlags::from_bits_truncate(*self.base_address.add(5)) }
    }

    /// Send a byte to this uart, may block until the uart is ready.
    ///
    /// # Arguments
    ///
    /// * `data` - the byte that should be sent
    fn put(&mut self, data: u8) {
        unsafe {
            while !self.status().contains(StatusFlags::OUTPUT_EMPTY) {
                core::hint::spin_loop();
            }
            // directly write into MMIO
            self.base_address.add(0).write_volatile(data);
        }
    }
}

impl Module for Uart16550 {
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        // Set the word length to 8-bits by writing 1 into LCR[1:0]
        self.base_address.add(3).write_volatile((1 << 0) | (1 << 1));
        // Enable FIFO
        self.base_address.add(2).write_volatile(1 << 0);

        let divisor: u16 = 9600;
        let divisor_l: u8 = (divisor & 0xff).try_into().unwrap();
        let divisor_h: u8 = (divisor >> 8).try_into().unwrap();

        // Enable divisor latch
        let lcr = self.base_address.add(3).read_volatile();
        self.base_address.add(3).write_volatile(lcr | 1 << 7);
        // Write divisor
        self.base_address.add(0).write_volatile(divisor_l);
        self.base_address.add(1).write_volatile(divisor_h);
        // Close divisor latch
        self.base_address.add(3).write_volatile(lcr);

        self.initialized = true;

        Ok(())
    }

    fn initialized(&self) -> bool {
        self.initialized
    }
}

impl Write for Uart16550 {
    fn write_str(&mut self, data: &str) -> core::fmt::Result {
        if !self.initialized {
            Err(core::fmt::Error)
        } else {
            for c in data.as_bytes() {
                self.put(*c);
            }

            Ok(())
        }
    }
}

impl ByteRead for Uart16550 {
    fn read_byte(&self) -> Option<u8> {
        unsafe {
            if self.status().contains(StatusFlags::INPUT_FULL) {
                Some(self.base_address.add(0).read_volatile())
            } else {
                None
            }
        }
    }
}
