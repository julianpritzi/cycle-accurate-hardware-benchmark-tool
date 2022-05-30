#![allow(dead_code)]

use crate::modules::{Module, RNGModule};
use bitflags::bitflags;

bitflags! {
    /// Abstract representation of the interrupt state register.
    struct CsrngINTRState: u32 {
        const CS_CMD_REQ_DONE = 1 << 0;
        const CS_ENTROPY_REQ = 1 << 1;
        const CS_HW_INST_EXC = 1 << 2;
        const CS_FATAL_ERR = 1 << 3;
    }

    /// Abstract representation of the command header flags.
    struct CsrngCMDHeader: u32 {
        const FLAG0 = 1 << 8;
        const FLAG1 = 1 << 9;
        const FLAG2 = 1 << 10;
        const FLAG3 = 1 << 11;
    }

    /// Abstract representation of the register write enabled register flags.
    struct CsrngREGWEN: u32 {
        const REGWEN = 1 << 0;
    }

    /// Abstract representation of the command status register flags.
    struct CsrngCMDStatus: u32 {
        const CMD_RDY = 1 << 0;
        const CMD_STS = 1 << 1;
    }

    /// Abstract representation of the generated bits valid register flags.
    struct CsrngGENBITSValid: u32 {
        const GENBITS_VLD = 1 << 0;
        const GENBITS_FIPS = 1 << 1;
    }

    /// Abstract representation of the command header flags.
    struct CsrngHWStatus: u32 {
        const HW0_ERR = 1 << 0;
        const HW1_ERR = 1 << 1;
        const HW2_ERR = 1 << 2;
        const HW3_ERR = 1 << 3;
        const HW4_ERR = 1 << 4;
        const HW5_ERR = 1 << 5;
        const HW6_ERR = 1 << 6;
        const HW7_ERR = 1 << 7;
        const HW8_ERR = 1 << 8;
        const HW9_ERR = 1 << 9;
        const HW10_ERR = 1 << 10;
        const HW11_ERR = 1 << 11;
        const HW12_ERR = 1 << 12;
        const HW13_ERR = 1 << 13;
        const HW14_ERR = 1 << 14;
    }

    /// Abstract representation of the error code register flags.
    struct CsrngERRCode: u32 {
        const SFIFO_CMD_ERR = 1 << 0;
        const SFIFO_GENBITS_ERR = 1 << 1;
        const SFIFO_CMDREQ_ERR = 1 << 2;
        const SFIFO_RCSTAGE_ERR = 1 << 3;
        const SFIFO_KEYVRC_ERR = 1 << 4;
        const SFIFO_UPDREQ_ERR = 1 << 5;
        const SFIFO_BENCREQ_ERR = 1 << 6;
        const SFIFO_BENCACK_ERR = 1 << 7;
        const SFIFO_PDATA_ERR = 1 << 8;
        const SFIFO_FINAL_ERR = 1 << 9;
        const SFIFO_GBENCACK_ERR = 1 << 10;
        const SFIFO_GRCSTAGE_ERR = 1 << 11;
        const SFIFO_GGENREQ_ERR = 1 << 12;
        const SFIFO_GADSTAGE_ERR = 1 << 13;
        const SFIFO_GGENBITS_ERR = 1 << 14;
        const SFIFO_BLKENC_ERR = 1 << 15;
        const CMD_STAGE_SM_ERR = 1 << 20;
        const MAIN_SM_ERR = 1 << 21;
        const DRBG_GEN_SM_ERR = 1 << 22;
        const DRBG_UPDBE_SM_ERR = 1 << 23;
        const DRBG_UPDOB_SM_ERR = 1 << 24;
        const AES_CIPHER_SM_ERR = 1 << 25;
        const CMD_GEN_CNT_ERR = 1 << 26;
        const FIFO_WRITE_ERR = 1 << 28;
        const FIFO_READ_ERR = 1 << 29;
        const FIFO_STATE_ERR = 1 << 30;
    }
}

/// Offset of the interrupt state register
const CSRNG_INTR_STATE_OFFSET: usize = 0x0;
/// Offset of the write enabled register
const CSRNG_REGWEN_OFFSET: usize = 0x10;
/// Offset of the control register
const CSRNG_CTRL_OFFSET: usize = 0x14;
/// Offset of the command request register
const CSRNG_CMD_REQ_OFFSET: usize = 0x18;
/// Offset of the software command status register
const CSRNG_SW_CMD_STS_OFFSET: usize = 0x1c;
/// Offset of the generated bits valid status register
const CSRNG_GENBITS_VLD_OFFSET: usize = 0x20;
/// Offset of the generated bits register
const CSRNG_GENBITS_OFFSET: usize = 0x24;
/// Offset of the interrupt state register
const CSRNG_HW_EXEC_STS_OFFSET: usize = 0x30;
/// Offset of the error code register
const CSRNG_ERR_CODE_OFFSET: usize = 0x38;

/// Multi bit value representing true
/// Used when a true value has to be represented with 4 bits
const K_MULTI_BIT_BOOL4_TRUE: u32 = 0xA;
/// Multi bit value representing false
/// Used when a false value has to be represented with 4 bits
const K_MULTI_BIT_BOOL4_FALSE: u32 = 0x5;

#[derive(Copy, Clone)]
enum CsrngCMD {
    Instantiate = 0x1,
    Reseed = 0x2,
    Generate = 0x3,
    Update = 0x4,
    Uninstantiate = 0x5,
}

/// CSRNG driver implementation as described by:
/// https://docs.opentitan.org/hw/ip/csrng/doc/
///
/// TODO: Check on actual hardware if the following errors persist:
/// - hwip always generates 0 as random bits
/// - hwip hangs when requesting seed from entropy source,\
///   potentially because none is present?
pub struct OpentitanCSRNG {
    initialized: bool,
    base_address: *mut u8,
}

impl OpentitanCSRNG {
    /// Creates a new OpentitanCSRNG driver
    ///
    /// # Arguments
    ///
    /// * `base_address` - A pointer to the MMIO address of the csrng device
    ///
    /// # Safety:
    ///  - a valid csrng device must be at the base_address
    ///  - no other csrng module must use the same base_address
    pub const unsafe fn new(base_address: *mut u8) -> OpentitanCSRNG {
        OpentitanCSRNG {
            initialized: false,
            base_address,
        }
    }

    /// Returns pointer to interrupt state register
    #[inline]
    unsafe fn _interrupt_state_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_INTR_STATE_OFFSET) as *mut u32
    }

    /// Returns pointer to register write enabled register
    #[inline]
    unsafe fn _regwen_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_REGWEN_OFFSET) as *mut u32
    }

    /// Returns pointer to control register
    #[inline]
    unsafe fn _control_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_CTRL_OFFSET) as *mut u32
    }

    /// Returns pointer to command request register
    #[inline]
    unsafe fn _command_request_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_CMD_REQ_OFFSET) as *mut u32
    }

    /// Returns pointer to software command status register
    #[inline]
    unsafe fn _command_status_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_SW_CMD_STS_OFFSET) as *mut u32
    }

    /// Returns pointer to generated bits valid register
    #[inline]
    unsafe fn _generated_bits_valid_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_GENBITS_VLD_OFFSET) as *mut u32
    }

    /// Returns pointer to generated bits register
    #[inline]
    unsafe fn _generated_bits_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_GENBITS_OFFSET) as *mut u32
    }

    /// Returns pointer to generated bits register
    #[inline]
    unsafe fn _hardware_exception_status_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_HW_EXEC_STS_OFFSET) as *mut u32
    }

    /// Returns pointer to error code register
    #[inline]
    unsafe fn _error_code_reg(&self) -> *mut u32 {
        self.base_address.add(CSRNG_ERR_CODE_OFFSET) as *mut u32
    }

    /// Sends request data via the command request register
    #[inline]
    unsafe fn send_req_data(&self, data: u32) {
        while !CsrngCMDStatus::from_bits_unchecked(self._command_status_reg().read_volatile())
            .contains(CsrngCMDStatus::CMD_RDY)
        {
            core::hint::spin_loop();
        }
        self._command_request_reg().write_volatile(data);
    }
}

impl Module for OpentitanCSRNG {
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        self._control_reg().write_volatile(
            K_MULTI_BIT_BOOL4_TRUE | (K_MULTI_BIT_BOOL4_TRUE << 4) | (K_MULTI_BIT_BOOL4_TRUE << 8),
        );
        self._hardware_exception_status_reg().write_volatile(0);

        Ok(())
    }

    fn initialized(&self) -> bool {
        self.initialized
    }
}

impl RNGModule for OpentitanCSRNG {
    fn init_rng(&self, seed: Option<alloc::vec::Vec<u32>>) {
        unsafe {
            let header = generate_header(CsrngCMD::Uninstantiate, 0, 0, 0);
            self.send_req_data(header);

            if let Some(seed) = seed {
                let seed_len = seed.len();
                let seed_len = if seed_len < 12 { seed_len } else { 12 };

                let header = generate_header(
                    CsrngCMD::Instantiate,
                    seed_len as u32,
                    CsrngCMDHeader::FLAG0.bits(),
                    0,
                );
                self.send_req_data(header);

                for value in &seed[0..seed_len] {
                    self.send_req_data(*value);
                }
            } else {
                let header = generate_header(CsrngCMD::Instantiate, 0, 0, 0);
                self.send_req_data(header);
            }
        }
    }

    fn generate(&self) -> u128 {
        unsafe {
            let header = generate_header(CsrngCMD::Generate, 0, 0, 1);
            self.send_req_data(header);

            while !CsrngGENBITSValid::from_bits_unchecked(
                self._generated_bits_valid_reg().read_volatile(),
            )
            .contains(CsrngGENBITSValid::GENBITS_VLD)
            {
                core::hint::spin_loop()
            }

            (self._generated_bits_reg().read_volatile() as u128) << (0 * 32)
                | (self._generated_bits_reg().read_volatile() as u128) << (1 * 32)
                | (self._generated_bits_reg().read_volatile() as u128) << (2 * 32)
                | (self._generated_bits_reg().read_volatile() as u128) << (3 * 32)
        }
    }
}

/// Generates an application command header according to the documentation
///
/// # Arguments
///
/// * `acmd` - The application command to execute
/// * `clen` - The command length, has to be between 0 and 12
/// * `flags` - Valid CsrngCMDHeader flags
/// * `glen` - The generate length, has to be between 0 and 4096
///
/// # Safety:
///  - argument restrictions have to be upheld
unsafe fn generate_header(acmd: CsrngCMD, clen: u32, flags: u32, glen: u32) -> u32 {
    acmd as u32 | (clen & 0b1111) << 4 | flags | (glen & 0b1111_1111_1111) << 12
}
