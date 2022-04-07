use super::Platform;

#[path = "../modules/uart16550.rs"]
mod uart16550;

static mut UART0: uart16550::Uart16550 =
    unsafe { uart16550::Uart16550::new(0x1000_0000 as *mut u8) };

pub struct VirtPlatform;

impl Platform for VirtPlatform {
    fn get_communication_module(
        &self,
    ) -> Option<&'static mut dyn crate::modules::CommunicationModule> {
        unsafe { Some(&mut UART0) }
    }

    fn suspend(&self, code: u32) -> ! {
        // use the sifive_test device
        let addr = 0x100000 as *mut u32;

        let value = if code == 0 {
            0x5555
        } else {
            (code << 16) | 0x3333
        };

        unsafe { addr.write_volatile(value) };

        loop {}
    }
}