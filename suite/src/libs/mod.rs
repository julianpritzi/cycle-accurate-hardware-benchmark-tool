#[cfg(any(
    feature = "platform_verilator_earlgrey",
    feature = "platform_nexysvideo_earlgrey"
))]
pub mod ecdsa;
#[cfg(any(
    feature = "platform_verilator_earlgrey",
    feature = "platform_nexysvideo_earlgrey"
))]
pub mod otbn;
