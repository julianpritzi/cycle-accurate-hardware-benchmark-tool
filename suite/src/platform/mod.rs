use crate::modules::CommunicationModule;

use self::module_types::*;

#[cfg(any(
    feature = "platform_verilator_earlgrey",
    feature = "platform_nexysvideo_earlgrey"
))]
mod earlgrey;
#[cfg(feature = "platform_qemu_virt")]
mod virt;

#[cfg(feature = "platform_qemu_virt")]
pub mod module_types {
    use crate::modules::empty::EmptyModule;

    type SHA2Module = EmptyModule;
    type SHA3Module = EmptyModule;
    type AESModule = EmptyModule;
    type RNGModule = EmptyModule;
}

#[cfg(any(
    feature = "platform_verilator_earlgrey",
    feature = "platform_nexysvideo_earlgrey"
))]
pub mod module_types {
    use super::earlgrey;

    pub type SHA2Module = earlgrey::opentitan_hmac::OpentitanHMAC;
    pub type SHA3Module = earlgrey::opentitan_kmac::OpentitanKMAC;
    pub type AESModule = earlgrey::opentitan_aes::OpentitanAES;
    pub type RNGModule = earlgrey::opentitan_csrng::OpentitanCSRNG;
}

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

    /// Returns the platforms sha2 module if one is present.
    ///
    /// Returned type if present is guaranteed to implement the HashingModule trait
    fn get_sha2_module(&self) -> Option<&'static mut SHA2Module> {
        None
    }

    /// Returns the platforms sha3 module if one is present.
    ///
    /// Returned type if present is guaranteed to implement the HashingModule trait
    fn get_sha3_module(&self) -> Option<&'static mut SHA3Module> {
        None
    }

    /// Returns the platforms aes module if one is present.
    ///
    /// Returned type if present is guaranteed to implement the AESModule trait
    fn get_aes_module(&self) -> Option<&'static mut AESModule> {
        None
    }

    /// Returns the opentitan rng module if one is present.
    ///
    /// Returned type if present is guaranteed to implement the RNGModule trait
    fn get_rng_module(&self) -> Option<&'static mut RNGModule> {
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
