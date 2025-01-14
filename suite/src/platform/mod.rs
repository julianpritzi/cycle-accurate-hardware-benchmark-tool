use crate::modules::{AESModule, CommunicationModule, ModuleRef, RNGModule, SHA256Module};

#[cfg(feature = "platform_verilator_earlgrey")]
mod earlgrey;
#[cfg(feature = "platform_qemu_virt")]
mod virt;

/// Returns the platform the suite was compiled for.
pub fn current() -> impl Platform {
    #[cfg(feature = "platform_qemu_virt")]
    {
        virt::VirtPlatform
    }
    #[cfg(feature = "platform_verilator_earlgrey")]
    {
        earlgrey::EarlGreyPlatform
    }
}

/// A platform represents the underlying layer on which the suite runs.
///
/// A platform bundles the functionality it supports by including module implementations
/// and providing it with necessary information, like addresses of MMIOs.
pub trait Platform {
    /// Returns an implementation of the CommunicationModule
    /// that should be used by the suite to communicate with the CLI
    ///
    /// # Safety
    ///  - this module should only be used through the macros provided by `runtime.rs`
    ///  - calling the function more than once might invalidate previous references
    unsafe fn get_communication_module(&self) -> &'static mut dyn CommunicationModule;

    /// Returns the platforms SHA256 module if one is present.
    fn get_sha256_module(&self) -> Option<ModuleRef<dyn SHA256Module>> {
        None
    }

    /// Returns the platforms aes module if one is present.
    fn get_aes_module(&self) -> Option<ModuleRef<dyn AESModule>> {
        None
    }

    /// Returns the platforms aes module if one is present.
    fn get_rng_module(&self) -> Option<ModuleRef<dyn RNGModule>> {
        None
    }

    /// Signals the platform that the suite finished executing.
    /// What should happen when this function is called is defined by the platform.
    ///
    /// # Arguments
    ///
    /// * `code` - An exit code, where 0 represents success and any other value is interpreted as an error code
    fn suspend(&self, code: u32) -> !;
}
