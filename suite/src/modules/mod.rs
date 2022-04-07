/// Generic module trait, implemented by all modules
pub trait Module {
    /// Initialize module
    ///
    /// # Safety:
    /// - only call once
    unsafe fn init(&mut self) -> Result<(), &'static str>;

    fn initialized(&self) -> bool;
}

/// Module for communicating with the Benchmarking-CLI
pub trait CommunicationModule: core::fmt::Write + Module {}
