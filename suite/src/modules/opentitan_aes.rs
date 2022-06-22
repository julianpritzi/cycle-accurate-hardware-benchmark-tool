#![allow(dead_code)]

use crate::modules::{AESKeyLength, AESMode, AESModule, AESOperation, Module};
use bitflags::bitflags;

bitflags! {
    /// Abstract representation of the control registers flags.
    struct AesCTRL: u32 {
        /// Set if the AES module should decrypt, if not set it will encrypt
        const DECRYPT = 1 << 0;
        const MANUAL_OPERATION = 1 << 10;
        const FORCE_ZERO_MASKS = 1 << 11;
    }

    /// Abstract representation of the trigger registers flags.
    struct AesTRIGGER: u32 {
        /// Start encryption of next block in manual mode
        const START = 1 << 0;
        /// Clear all input registers with random data
        const KEY_IV_DATA_IN_CLEAR = 1 << 1;
        /// Clear all output registers with random data
        const DATA_OUT_CLEAR = 1 << 2;
        /// Reseed the internal PRNG used for masking
        const PRNG_RESEED = 1 << 3;
    }

    /// Abstract representation of the status registers flags.
    struct AesSTATUS: u32 {
        /// Set when the aes unit is ready for operation
        const IDLE = 1 << 0;
        /// Set when the aes unit is waiting for output to be read before continuing operation
        const STALL = 1 << 1;
        const OUTPUT_LOST = 1 << 2;
        /// Set when the aes unit has valid output data
        const OUTPUT_VALID = 1 << 3;
        /// Set when the aes unit can receive new input data
        const INPUT_READY = 1 << 4;
        const ALERT_RECOV_CTRL_UPDATE_ERR = 1 << 5;
        const ALERT_FATAL_FAULT = 1 << 6;
    }
}

/// Offset of the first key share register
///
/// Can be used like a [u32; 8] residing at this offset
const AES_KEY_SHARE0_OFFSET: usize = 0x4;
/// Offset of the second key share register
///
/// Can be used like a [u32; 8] residing at this offset
const AES_KEY_SHARE1_OFFSET: usize = 0x24;
/// Offset of the initialization vector register
///
/// Can be used like a u128 residing at this offset
const AES_KEY_IV_OFFSET: usize = 0x44;
/// Offset of the input data register
///
/// Can be used like a u128 residing at this offset
const AES_DATA_IN_OFFSET: usize = 0x54;
/// Offset of the output data register
///
/// Can be used like a u128 residing at this offset
const AES_DATA_OUT_OFFSET: usize = 0x64;
/// Offset of the control register. \
/// **Important: This register is shadowed so it has to be written to twice fo the change to take affect**
const AES_CTRL_SHADOWED_OFFSET: usize = 0x74;
/// Contains offsets & masks for values inside the control register
mod ctrl_reg {
    pub const OPERATION_OFFSET: u32 = 0x0;
    pub const OPERATION_MASK: u32 = 0b1;
    pub const MODE_OFFSET: u32 = 0x1;
    pub const MODE_MASK: u32 = 0b111111;
    pub const KEY_LEN_OFFSET: u32 = 0x7;
    pub const KEY_LEN_MASK: u32 = 0b111;
}
/// Offset of the status register.
const AES_TRIGGER_OFFSET: usize = 0x78;
/// Offset of the status register.
const AES_STATUS_OFFSET: usize = 0x7c;

/// AES driver implementation as described by:
/// https://docs.opentitan.org/hw/ip/aes/doc/
///
/// All registers are little endian.
pub struct OpentitanAES {
    initialized: bool,
    base_address: *mut u8,
}

impl OpentitanAES {
    /// Creates a new OpentitanAES driver
    ///
    /// # Arguments
    ///
    /// * `base_address` - A pointer to the MMIO address of the aes device
    ///
    /// # Safety:
    ///  - a valid aes device must be at the base_address
    ///  - no other aes must use the same base_address
    pub const unsafe fn new(base_address: *mut u8) -> OpentitanAES {
        OpentitanAES {
            initialized: false,
            base_address,
        }
    }

    /// Returns pointer to control register \
    /// **Important: This register is shadowed so it has to be written to twice fo the change to take affect**
    #[inline]
    unsafe fn _control_reg(&self) -> *mut u32 {
        self.base_address.add(AES_CTRL_SHADOWED_OFFSET) as *mut u32
    }

    /// Returns pointer to trigger register
    #[inline]
    unsafe fn _trigger_reg(&self) -> *mut u32 {
        self.base_address.add(AES_TRIGGER_OFFSET) as *mut u32
    }

    /// Returns pointer to status register
    #[inline]
    unsafe fn _status_reg(&self) -> *mut u32 {
        self.base_address.add(AES_STATUS_OFFSET) as *mut u32
    }

    /// Returns pointer to the first key share registers offset by offset u32, 0 <= offset < 8
    #[inline]
    unsafe fn _key_share_0(&self, offset: usize) -> *mut u32 {
        self.base_address.add(AES_KEY_SHARE0_OFFSET + offset * 4) as *mut u32
    }

    /// Returns pointer to the second key share registers offset by offset u32, 0 <= offset < 8
    #[inline]
    unsafe fn _key_share_1(&self, offset: usize) -> *mut u32 {
        self.base_address.add(AES_KEY_SHARE1_OFFSET + offset * 4) as *mut u32
    }

    /// Returns pointer to the initialization vector registers
    #[inline]
    unsafe fn _iv(&self) -> *mut u128 {
        self.base_address.add(AES_KEY_IV_OFFSET) as *mut u128
    }

    /// Returns pointer to the data input registers
    #[inline]
    unsafe fn _input(&self) -> *mut u128 {
        self.base_address.add(AES_DATA_IN_OFFSET) as *mut u128
    }

    /// Returns pointer to the data output registers
    #[inline]
    unsafe fn _output(&self) -> *mut u128 {
        self.base_address.add(AES_DATA_OUT_OFFSET) as *mut u128
    }

    /// Busy waits until some status is set
    #[inline]
    unsafe fn _wait_for(&self, status: AesSTATUS) {
        while !AesSTATUS::from_bits_unchecked(self._status_reg().read_volatile()).contains(status) {
            core::hint::spin_loop();
        }
    }

    /// Writes to the control register
    unsafe fn write_ctrl(&self, ctrl: u32) {
        self._control_reg().write_volatile(ctrl);
        self._control_reg().write_volatile(ctrl);
    }
}

impl Module for OpentitanAES {
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        // Wait for the initial clearing of registers to finish
        self._wait_for(AesSTATUS::IDLE);
        self.initialized = true;

        Ok(())
    }

    fn initialized(&self) -> bool {
        self.initialized
    }
}

impl AESModule for OpentitanAES {
    fn init_aes(
        &self,
        key_len: AESKeyLength,
        operation: AESOperation,
        mode: AESMode,
        key_share0: &[u32; 8],
        key_share1: &[u32; 8],
    ) {
        unsafe {
            // Wait for the AES unit to become ready
            self._wait_for(AesSTATUS::IDLE);

            let (serialized_mode, iv) = _serialize_mode(mode);
            let ctrl_val: u32 =
                _serialize_key_len(key_len) | _serialize_operation(operation) | serialized_mode;

            self.write_ctrl(ctrl_val);

            for i in 0..8 {
                self._key_share_0(i).write_volatile(key_share0[i]);
                self._key_share_1(i).write_volatile(key_share1[i]);
            }

            self._wait_for(AesSTATUS::IDLE);

            if let Some(iv) = iv {
                self._iv().write_volatile(iv);
            }
        }
    }

    fn execute(&self, input: &[u128], output: &mut [u128]) {
        unsafe {
            let input_len = input.len();

            self._input().write_volatile(input[0]);
            while self._status_reg().read_volatile() & AesSTATUS::INPUT_READY.bits() == 0 {}
            self._input().write_volatile(input[1]);

            for blk_count in 2..input_len {
                while self._status_reg().read_volatile() & AesSTATUS::OUTPUT_VALID.bits() == 0 {}
                output[blk_count - 2] = self._output().read_volatile();

                self._input().write(input[blk_count]);
            }

            while self._status_reg().read_volatile() & AesSTATUS::OUTPUT_VALID.bits() == 0 {}
            output[input_len - 2] = self._output().read_volatile();
            while self._status_reg().read_volatile() & AesSTATUS::OUTPUT_VALID.bits() == 0 {}
            output[input_len - 1] = self._output().read_volatile();
        }
    }

    fn execute_inplace(&self, data: &mut [u128]) {
        unsafe {
            for blk_count in 0..(data.len() + 2) {
                if blk_count == 1 {
                    self._wait_for(AesSTATUS::INPUT_READY);
                }

                if blk_count > 1 {
                    self._wait_for(AesSTATUS::OUTPUT_VALID);

                    data[blk_count - 2] = self._output().read();
                }

                if blk_count < data.len() {
                    self._input().write_volatile(data[blk_count]);
                }
            }
        }
    }

    fn deinitialize(&self) {
        unsafe {
            let ctrl_val: u32 = AesCTRL::MANUAL_OPERATION.bits();
            let ctrl_reg = self._control_reg();
            ctrl_reg.write_volatile(ctrl_val);
            ctrl_reg.write_volatile(ctrl_val);

            self._trigger_reg().write_volatile(
                (AesTRIGGER::KEY_IV_DATA_IN_CLEAR | AesTRIGGER::DATA_OUT_CLEAR).bits(),
            );

            self._wait_for(AesSTATUS::IDLE);
        }
    }
}

/// Serializes the key length according to to the opentitan docs, so it can be directly written into the control register
#[inline]
fn _serialize_key_len(val: AESKeyLength) -> u32 {
    let val = match val {
        AESKeyLength::Aes128 => 0x1,
        AESKeyLength::Aes192 => 0x2,
        AESKeyLength::Aes256 => 0x4,
    };

    (val & ctrl_reg::KEY_LEN_MASK) << ctrl_reg::KEY_LEN_OFFSET
}

/// Serializes the operation according to to the opentitan docs, so it can be directly written into the control register
#[inline]
fn _serialize_operation(val: AESOperation) -> u32 {
    let val = match val {
        AESOperation::Encrypt => 0x0,
        AESOperation::Decrypt => 0x1,
    };

    (val & ctrl_reg::OPERATION_MASK) << ctrl_reg::OPERATION_OFFSET
}

/// Serializes the operation according to to the opentitan docs,
/// so the first value can be directly written into the control register.
/// The second value corresponds to an IV if present
#[inline]
fn _serialize_mode(val: AESMode) -> (u32, Option<u128>) {
    let mut ret_iv = None;
    let val = match val {
        AESMode::ECB => 0x01,
        AESMode::CBC { iv } => {
            ret_iv = Some(iv);
            0x02
        }
        AESMode::CFB => 0x04,
        AESMode::OFB => 0x08,
        AESMode::CTR { iv } => {
            ret_iv = Some(iv);
            0x10
        }
    };

    ((val & ctrl_reg::MODE_MASK) << ctrl_reg::MODE_OFFSET, ret_iv)
}
