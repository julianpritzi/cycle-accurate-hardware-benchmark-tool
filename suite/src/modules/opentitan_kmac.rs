#![allow(dead_code)]

use crate::modules::{Module, SHA256Module};
use bitflags::bitflags;

bitflags! {
    /// Abstract representation of the config registers flags.
    struct KmacCFG: u32 {
        const KMAC_ENABLED = 1 << 0;
        }

    /// Abstract representation of the command registers flags.
    struct KmacCMD: u32 {
        const START = 1 << 0;
        const PROCESS = 1 << 1;
        const RUN = 1 << 2;
        const DONE = 1 << 3;
    }

    /// Abstract representation of the status registers flags.
    struct KmacSTATUS: u32 {
        const SHA3_IDLE = 1 << 0;
        const SHA3_ABSORB = 1 << 1;
        const SHA3_SQUEEZE = 1 << 2;

        const FIFO_EMPTY = 1 << 14;
        const FIFO_FULL = 1 << 15;
    }
}

/// Offset of the configuration register
const KMAC_CFG_OFFSET: usize = 0x14;
/// Offset of the command register
const KMAC_CMD_OFFSET: usize = 0x18;
/// Offset of the status register
const KMAC_STATUS_OFFSET: usize = 0x1c;
/// Offset of the digest register
///
/// Digest can be used like an [u32; 8] residing at this offset
const KMAC_DIGEST_OFFSET: usize = 0x400;
/// Offset of the message register
const KMAC_MSG_OFFSET: usize = 0x800;

/// KMAC driver implementation as described by:
/// https://docs.opentitan.org/hw/ip/kmac/doc/
pub struct OpentitanKMAC {
    initialized: bool,
    base_address: *mut u8,
}

impl OpentitanKMAC {
    /// Creates a new OpentitanKMAC driver
    ///
    /// # Arguments
    ///
    /// * `base_address` - A pointer to the MMIO address of the kmac device
    ///
    /// # Safety:
    ///  - a valid kmac device must be at the base_address
    ///  - no other kmac must use the same base_address
    pub const unsafe fn new(base_address: *mut u8) -> OpentitanKMAC {
        OpentitanKMAC {
            initialized: true,
            base_address,
        }
    }

    /// Returns pointer to configuration register
    #[inline]
    unsafe fn _config_reg(&self) -> *mut u32 {
        self.base_address.add(KMAC_CFG_OFFSET) as *mut u32
    }

    /// Returns pointer to command register
    #[inline]
    unsafe fn _command_reg(&self) -> *mut u32 {
        self.base_address.add(KMAC_CMD_OFFSET) as *mut u32
    }

    /// Returns pointer to status register
    #[inline]
    unsafe fn _status_reg(&self) -> *mut u32 {
        self.base_address.add(KMAC_STATUS_OFFSET) as *mut u32
    }

    /// Returns pointer to digest register
    #[inline]
    unsafe fn _digest(&self) -> *mut [u32; 8] {
        self.base_address.add(KMAC_DIGEST_OFFSET) as *mut [u32; 8]
    }

    /// Returns pointer to message register
    #[inline]
    unsafe fn _msg_reg(&self) -> *mut u32 {
        self.base_address.add(KMAC_MSG_OFFSET) as *mut u32
    }
}

impl Module for OpentitanKMAC {
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn initialized(&self) -> bool {
        self.initialized
    }
}

impl SHA256Module for OpentitanKMAC {
    fn init_sha256(&self) {
        unsafe { self._config_reg().write_volatile(0) }
    }

    fn write_input(&self, data: &[u32]) {
        unsafe {
            self._command_reg().write_volatile(KmacCMD::START.bits());

            for value in data {
                while KmacSTATUS::from_bits_unchecked(self._status_reg().read_volatile())
                    .contains(KmacSTATUS::FIFO_FULL)
                {
                    core::hint::spin_loop()
                }

                self._msg_reg().write_volatile(*value);
            }
        }
    }

    fn wait_for_completion(&self) {
        unsafe {
            self._command_reg().write_volatile(KmacCMD::PROCESS.bits());

            while !KmacSTATUS::from_bits_unchecked(self._status_reg().read_volatile())
                .contains(KmacSTATUS::SHA3_SQUEEZE)
            {
                core::hint::spin_loop()
            }
        }
    }

    fn read_digest(&self, buffer: &mut [u32; 8]) {
        unsafe { buffer.copy_from_slice(&self._digest().read_volatile()) }
    }
}
