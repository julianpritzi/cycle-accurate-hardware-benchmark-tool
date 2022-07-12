use crate::modules::CommunicationModule;

mod earlgrey;
#[cfg(feature = "platform_qemu_virt")]
mod virt;

/// Returns the platform the suite was compiled for.
pub fn current() -> impl Platform {
    #[cfg(feature = "platform_qemu_virt")]
    {
        virt::VirtPlatform
    }
    #[cfg(any(
        feature = "platform_verilator_earlgrey",
        feature = "platform_nexysvideo_earlgrey"
    ))]
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

    /// Returns the opentitan SHA256 module if one is present.
    fn get_sha256_module(&self) -> Option<&'static mut earlgrey::opentitan_hmac::OpentitanHMAC> {
        None
    }

    /// Returns the opentitan aes module if one is present.
    fn get_aes_module(&self) -> Option<&'static mut earlgrey::opentitan_aes::OpentitanAES> {
        None
    }

    /// Returns the opentitan rng module if one is present.
    fn get_rng_module(&self) -> Option<&'static mut earlgrey::opentitan_csrng::OpentitanCSRNG> {
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
