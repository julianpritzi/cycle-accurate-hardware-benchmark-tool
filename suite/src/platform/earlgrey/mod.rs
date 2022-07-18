use core::arch::global_asm;

use crate::println;

use super::Platform;

#[path = "../../modules/opentitan_aes.rs"]
pub mod opentitan_aes;
#[path = "../../modules/opentitan_csrng.rs"]
pub mod opentitan_csrng;
#[path = "../../modules/opentitan_hmac.rs"]
pub mod opentitan_hmac;
#[path = "../../modules/opentitan_kmac.rs"]
pub mod opentitan_kmac;
#[path = "../../modules/opentitan_uart.rs"]
pub mod opentitan_uart;

// Opentitan requires a manifest and custom interrupt vector,
// these are realized in ibex_start_XXX.S and included here.
#[cfg(feature = "platform_verilator_earlgrey")]
global_asm!(include_str!("ibex_start_verilator.S"));
#[cfg(feature = "platform_nexysvideo_earlgrey")]
global_asm!(include_str!("ibex_start_nexysvideo.S"));

#[cfg(feature = "platform_verilator_earlgrey")]
// Note: clk_hz & baud_rate according to sw/device/lib/arch/device_sim_verilator.c
static mut UART0: opentitan_uart::OpentitanUart =
    unsafe { opentitan_uart::OpentitanUart::new(0x4000_0000 as *mut u8, 7200, 125_000) };
#[cfg(feature = "platform_nexysvideo_earlgrey")]
// Note: clk_hz & baud_rate according to sw/device/lib/arch/device_fpga_nexysvideo.c
static mut UART0: opentitan_uart::OpentitanUart =
    unsafe { opentitan_uart::OpentitanUart::new(0x4000_0000 as *mut u8, 115200, 25 * 100 * 1000) };
static mut HMAC: opentitan_hmac::OpentitanHMAC =
    unsafe { opentitan_hmac::OpentitanHMAC::new(0x4111_0000 as *mut u8) };
static mut KMAC: opentitan_kmac::OpentitanKMAC =
    unsafe { opentitan_kmac::OpentitanKMAC::new(0x41120000 as *mut u8) };
static mut AES: opentitan_aes::OpentitanAES =
    unsafe { opentitan_aes::OpentitanAES::new(0x4110_0000 as *mut u8) };
static mut CSRNG: opentitan_csrng::OpentitanCSRNG =
    unsafe { opentitan_csrng::OpentitanCSRNG::new(0x41150000 as *mut u8) };

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

    fn get_sha256_module(&self) -> Option<&'static mut opentitan_hmac::OpentitanHMAC> {
        unsafe { Some(&mut HMAC) }
    }

    fn get_aes_module(&self) -> Option<&'static mut opentitan_aes::OpentitanAES> {
        unsafe { Some(&mut AES) }
    }

    fn get_rng_module(&self) -> Option<&'static mut opentitan_csrng::OpentitanCSRNG> {
        unsafe { Some(&mut CSRNG) }
    }

    fn get_sha3_module(&self) -> Option<&'static mut self::opentitan_kmac::OpentitanKMAC> {
        unsafe { Some(&mut KMAC) }
    }
}
