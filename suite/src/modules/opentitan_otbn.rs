#![allow(dead_code)]

use crate::modules::Module;
use bitflags::bitflags;

bitflags! {
    /// Abstract representation of the status registers flags.
    struct OTBNStatus: u32 {
        /// Set if the otbn is busy
        const BUSY = 1 << 0;
    }

    /// Abstract representation of the command registers flags.
    struct OTBNCmd: u32 {
        /// Trigger the start of otbn
        const START = 1 << 0;
    }
}

/// Offset of the command register.
const OTBN_CMD_OFFSET: usize = 0x10;
/// Offset of the status register.
const OTBN_STATUS_OFFSET: usize = 0x14;

/// Offset of instruction memory.
const OTBN_IMEM_OFFSET: usize = 0x4000;
/// Offset of data memory.
const OTBN_DMEM_OFFSET: usize = 0x8000;

/// OTBN driver implementation
pub struct OpentitanOTBN {
    initialized: bool,
    base_address: *mut u8,
}

impl OpentitanOTBN {
    /// Creates a new OpentitanOTBN driver
    ///
    /// # Arguments
    ///
    /// * `base_address` - A pointer to the MMIO address of the aes device
    ///
    /// # Safety:
    ///  - a valid otbn device must be at the base_address
    ///  - no other otbn must use the same base_address
    pub const unsafe fn new(base_address: *mut u8) -> OpentitanOTBN {
        OpentitanOTBN {
            initialized: false,
            base_address,
        }
    }

    /// Returns pointer to command register
    #[inline]
    unsafe fn _command_reg(&self) -> *mut u32 {
        self.base_address.add(OTBN_CMD_OFFSET) as *mut u32
    }

    /// Returns pointer to status register
    #[inline]
    unsafe fn _status_reg(&self) -> *mut u32 {
        self.base_address.add(0) as *mut u32
    }

    pub fn test(&self) {
        println!("Testing OTBN");

        println!("{:?}", self._status_reg().read_volatile());
    }
}

impl Module for OpentitanOTBN {
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        self.initialized = true;
        Ok(())
    }

    fn initialized(&self) -> bool {
        self.initialized
    }
}
