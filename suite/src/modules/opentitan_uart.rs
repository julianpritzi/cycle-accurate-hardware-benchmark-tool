#![allow(dead_code)]

use core::fmt::Write;

use crate::modules::{ByteRead, Module};
use bitflags::bitflags;

bitflags! {
    /// Abstract representation of the control registers flags.
    struct UartCTRL: u32 {
        const TX_ENABLED = 1 << 0;
        const RX_ENABLED = 1 << 1;
    }

    /// Abstract representation of the status registers flags.
    struct UartSTATUS: u32 {
        const TX_FULL = 1 << 0;
        const RX_FULL = 1 << 1;
        const TX_EMPTY = 1 << 2;
        const TX_IDLE = 1 << 3;
        const RX_IDLE = 1 << 4;
        const RX_EMPTY = 1 << 5;
    }

    /// Abstract representation of the fifo control registers flags.
    struct FifoCTRL: u32 {
        const RX_RESET = 1 << 0;
        const TX_RESET = 1 << 1;
    }
}

/// Offset of the control register
const UART_CTRL_OFFSET: usize = 0x10;
/// Offset of the status register
const UART_STATUS_OFFSET: usize = 0x14;
/// Offset of the read data register
const UART_RDATA_OFFSET: usize = 0x18;
/// Offset of the write data register
const UART_WDATA_OFFSET: usize = 0x1c;
/// Offset of the fifo control register
const UART_FIFO_CTRL_OFFSET: usize = 0x20;
/// Offset of the fifo status register
const UART_FIFO_STATUS_OFFSET: usize = 0x24;

/// Offset of the NCO value inside the UartCTRL register
const UART_NCO_OFFSET: u32 = 16;
/// Mask of the NCO value
const UART_NCO_MASK: u64 = 0xffff_ffff_ffff_ffff;

/// Offset of the RX value inside the FifoStatus register
const UART_RX_LVL_OFFSET: u32 = 16;
const UART_MAX_RX_LVL: u8 = 32;
/// Offset of the RX value inside the FifoStatus register
const UART_TX_LVL_OFFSET: u32 = 0;
const UART_MAX_TX_LVL: u8 = 32;
const UART_LVL_MASK: u32 = 0xff_ffff;

/// Uart driver implementation as described by:
/// https://docs.opentitan.org/hw/ip/uart/doc/
pub struct OpentitanUart {
    initialized: bool,
    baud_rate: u64,
    clk_hz: u64,
    base_address: *mut u8,
}

impl OpentitanUart {
    /// Creates a new OpentitanUart driver
    ///
    /// # Arguments
    ///
    /// * `base_address` - A pointer to the MMIO address of the uart device
    /// * `baud_rate` - The baud rate the uart should be configured with
    /// * `clk_hz` - Clock speed of the peripheral clock, used to setup the uart device
    ///
    /// # Safety:
    ///  - a valid uart device must be at the base_address
    ///  - no other uart must use the same base_address
    pub const unsafe fn new(base_address: *mut u8, baud_rate: u64, clk_hz: u64) -> OpentitanUart {
        OpentitanUart {
            initialized: false,
            baud_rate,
            clk_hz,
            base_address,
        }
    }

    /// Returns pointer to control register
    unsafe fn _control_reg(&self) -> *mut u32 {
        self.base_address.add(UART_CTRL_OFFSET) as *mut u32
    }

    /// Returns pointer to status register
    unsafe fn _status_reg(&self) -> *mut u32 {
        self.base_address.add(UART_STATUS_OFFSET) as *mut u32
    }

    /// Returns pointer to read data register
    unsafe fn _read_reg(&self) -> *mut u8 {
        self.base_address.add(UART_RDATA_OFFSET) as *mut u8
    }

    /// Returns pointer to write data register
    unsafe fn _write_reg(&self) -> *mut u8 {
        self.base_address.add(UART_WDATA_OFFSET) as *mut u8
    }

    /// Returns pointer to fifo control register
    unsafe fn _fifo_control_reg(&self) -> *mut u32 {
        self.base_address.add(UART_FIFO_CTRL_OFFSET) as *mut u32
    }

    /// Returns pointer to fifo status register
    unsafe fn _fifo_status_reg(&self) -> *mut u32 {
        self.base_address.add(UART_FIFO_STATUS_OFFSET) as *mut u32
    }

    /// Uses the fifo control register to signal to the HWIP to reset the fifos
    unsafe fn reset_fifos(&self) {
        self._fifo_control_reg()
            .write_volatile((FifoCTRL::RX_RESET | FifoCTRL::TX_RESET).bits())
    }

    /// Returns the bytes currently in the sending queue
    unsafe fn get_tx_lvl(&self) -> u8 {
        ((self._fifo_status_reg().read_volatile() >> UART_TX_LVL_OFFSET) & UART_LVL_MASK) as u8
    }

    /// Returns the bytes currently in the receiving queue
    unsafe fn get_rx_lvl(&self) -> u8 {
        ((self._fifo_status_reg().read_volatile() >> UART_RX_LVL_OFFSET) & UART_LVL_MASK) as u8
    }

    /// Tries to send a byte, fails if the send queue is full
    ///
    /// # Arguments
    ///
    /// * `val` - the byte that should be sent
    unsafe fn put(&self, val: u8) -> Result<(), ()> {
        if self.get_tx_lvl() < UART_MAX_TX_LVL {
            self._write_reg().write_volatile(val);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Tries to read a byte, returns none if the receiving queue is empty
    unsafe fn get(&self) -> Option<u8> {
        if self.get_rx_lvl() > 0 {
            Some(self._read_reg().read_volatile())
        } else {
            None
        }
    }
}

impl Module for OpentitanUart {
    unsafe fn init(&mut self) -> Result<(), &'static str> {
        if self.initialized {
            return Ok(());
        }

        let nco = ((self.baud_rate << 20) / self.clk_hz) & UART_NCO_MASK;

        // Set BAUD and enable RX & TX
        self._control_reg().write_volatile(
            (nco as u32) << UART_NCO_OFFSET | (UartCTRL::RX_ENABLED | UartCTRL::TX_ENABLED).bits(),
        );

        self.reset_fifos();

        self.initialized = true;

        Ok(())
    }

    fn initialized(&self) -> bool {
        self.initialized
    }
}

impl Write for OpentitanUart {
    fn write_str(&mut self, data: &str) -> core::fmt::Result {
        if !self.initialized {
            Err(core::fmt::Error)
        } else {
            unsafe {
                for c in data.as_bytes() {
                    while self.put(*c).is_err() {
                        core::hint::spin_loop();
                    }
                }
            }

            Ok(())
        }
    }
}

impl ByteRead for OpentitanUart {
    fn read_byte(&self) -> Option<u8> {
        unsafe { self.get() }
    }
}
