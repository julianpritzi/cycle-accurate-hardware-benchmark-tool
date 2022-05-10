use core::arch::global_asm;

use crate::{modules::ModuleRef, println};

use super::Platform;

#[path = "../../modules/opentitan_hmac.rs"]
mod opentitan_hmac;
#[path = "../../modules/opentitan_uart.rs"]
mod opentitan_uart;

// Opentitan requires a manifest and custom interrupt vector,
// these are realized in ibex_start.S and included here.
global_asm!(include_str!("ibex_start.S"));

// Note: clk_hz & baud_rate according to sw/device/lib/arch/device_sim_verilator.c
static mut UART0: opentitan_uart::OpentitanUart =
    unsafe { opentitan_uart::OpentitanUart::new(0x4000_0000 as *mut u8, 7200, 125_000) };
static mut HMAC: opentitan_hmac::OpentitanHMAC =
    unsafe { opentitan_hmac::OpentitanHMAC::new(0x4111_0000 as *mut u8) };

/// EarlGrey platform according to the Opentitan specification:
///
/// https://docs.opentitan.org/hw/top_earlgrey/doc/
pub struct EarlGreyPlatform;

impl Platform for EarlGreyPlatform {
    unsafe fn get_communication_module(
        &self,
    ) -> &'static mut dyn crate::modules::CommunicationModule {
        // Safety:
        // there possibly exist multiple mutable references to UART0
        // but the responsibility to ensure correctness is delegated
        // to the caller of this function
        &mut UART0
    }

    fn suspend(&self, _code: u32) -> ! {
        // If this is a successful suspension, try printing it to the user
        if _code == 0 {
            println!("Successfully finished executing, going to sleep!")
        }

        loop {
            unsafe {
                riscv::asm::wfi();
            }
        }
    }

    fn get_sha256_module(&self) -> Option<ModuleRef<dyn crate::modules::SHA256Module>> {
        unsafe { Some(ModuleRef::new(&mut HMAC)) }
    }
}
