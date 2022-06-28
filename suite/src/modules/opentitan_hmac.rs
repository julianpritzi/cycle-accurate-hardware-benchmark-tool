#![allow(dead_code)]

use crate::modules::{HashingModule, Module};
use bitflags::bitflags;

bitflags! {
    /// Abstract representation of the config registers flags.
    struct HmacCFG: u32 {
        const HMAC_ENABLED = 1 << 0;
        const SHA_ENABLED = 1 << 1;
        /// If set the input is interpreted in little endian, otherwise big endian
        const ENDIAN_SWAPPED = 1 << 2;
        /// If set the output is in big endian, otherwise little endian
        const DIGEST_ENDIAN_SWAPPED = 1 << 3;
    }

    /// Abstract representation of the command registers flags.
    struct HmacCMD: u32 {
        const HASH_START = 1 << 0;
        const HASH_PROCESS = 1 << 1;
    }

    /// Abstract representation of the status registers flags.
    struct HmacSTATUS: u32 {
        const FIFO_EMPTY = 1 << 0;
        const FIFO_FULL = 1 << 1;
    }

    /// Abstract representation of the interrupt state registers flags.
    struct HmacINTRSTATE: u32 {
        const HMAC_DONE = 1 << 0;
        const FIFO_EMPTY = 1 << 1;
        const HMAC_ERR = 1 << 2;
    }
}

/// Offset of the interrupt state register
const HMAC_INTR_STATE_OFFSET: usize = 0x0;
/// Offset of the configuration register
const HMAC_CFG_OFFSET: usize = 0x10;
/// Offset of the command register
const HMAC_CMD_OFFSET: usize = 0x14;
/// Offset of the status register
const HMAC_STATUS_OFFSET: usize = 0x18;
/// Offset of the digest register
///
/// Digest can be used like an [u32; 8] residing at this offset
const HMAC_DIGEST_OFFSET: usize = 0x44;
/// Offset of the message register
const HMAC_MSG_OFFSET: usize = 0x800;

/// HMAC driver implementation as described by:
/// https://docs.opentitan.org/hw/ip/hmac/doc/
pub struct OpentitanHMAC {
    initialized: bool,
    base_address: *mut u8,
}

impl OpentitanHMAC {
    /// Creates a new OpentitanHMAC driver
    ///
    /// # Arguments
    ///
    /// * `base_address` - A pointer to the MMIO address of the hmac device
    ///
    /// # Safety:
    ///  - a valid hmac device must be at the base_address
    ///  - no other hmac must use the same base_address
    pub const unsafe fn new(base_address: *mut u8) -> OpentitanHMAC {
        OpentitanHMAC {
            initialized: true,
            base_address,
        }
    }

    /// Returns pointer to interrupt state register
    #[inline]
    unsafe fn _interrupt_state_reg(&self) -> *mut u32 {
        self.base_address.add(HMAC_INTR_STATE_OFFSET) as *mut u32
    }

    /// Returns pointer to configuration register
    #[inline]
    unsafe fn _config_reg(&self) -> *mut u32 {
        self.base_address.add(HMAC_CFG_OFFSET) as *mut u32
    }

    /// Returns pointer to command register
    #[inline]
    unsafe fn _command_reg(&self) -> *mut u32 {
        self.base_address.add(HMAC_CMD_OFFSET) as *mut u32
    }

    /// Returns pointer to status register
    #[inline]
    unsafe fn _status_reg(&self) -> *mut u32 {
        self.base_address.add(HMAC_STATUS_OFFSET) as *mut u32
    }

    /// Returns pointer to digest register
    #[inline]
    unsafe fn _digest(&self) -> *mut [u32; 8] {
        self.base_address.add(HMAC_DIGEST_OFFSET) as *mut [u32; 8]
    }

    /// Returns pointer to message register
    #[inline]
    unsafe fn _msg_reg(&self) -> *mut u32 {
        self.base_address.add(HMAC_MSG_OFFSET) as *mut u32
    }
}

impl Module for OpentitanHMAC {
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn initialized(&self) -> bool {
        self.initialized
    }
}

impl HashingModule for OpentitanHMAC {
    fn init_hashing(&self) {
        unsafe {
            self._config_reg()
                .write_volatile(HmacCFG::SHA_ENABLED.bits())
        }
    }

    fn write_input(&self, data: &[u32]) {
        unsafe {
            self._command_reg()
                .write_volatile(HmacCMD::HASH_START.bits());

            for value in data {
                while HmacSTATUS::from_bits_unchecked(self._status_reg().read_volatile())
                    .contains(HmacSTATUS::FIFO_FULL)
                {
                    core::hint::spin_loop()
                }

                self._msg_reg().write_volatile(*value);
            }
        }
    }

    fn wait_for_completion(&self) {
        unsafe {
            self._command_reg()
                .write_volatile(HmacCMD::HASH_PROCESS.bits());

            while !HmacINTRSTATE::from_bits_unchecked(self._interrupt_state_reg().read_volatile())
                .contains(HmacINTRSTATE::HMAC_DONE)
            {
                core::hint::spin_loop()
            }

            self._interrupt_state_reg()
                .write_volatile(HmacINTRSTATE::HMAC_DONE.bits());
        }
    }

    fn read_digest(&self, buffer: &mut [u32; 8]) {
        unsafe { buffer.copy_from_slice(&self._digest().read_volatile()) }
    }
}
