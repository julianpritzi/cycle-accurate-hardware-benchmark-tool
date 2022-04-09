use crate::modules::CommunicationModule;

#[cfg(feature = "platform_virt")]
mod virt;

pub fn current() -> impl Platform {
    #[cfg(feature = "platform_virt")]
    virt::VirtPlatform
}

pub trait Platform {
    /// # Safety
    ///  - this module should only be used through the macros provided by `runtime.rs`
    ///  - calling the function more than once might invalidate previous references
    unsafe fn get_communication_module(&self) -> &'static mut dyn CommunicationModule;

    fn suspend(&self, code: u32) -> !;
}
