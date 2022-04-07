use crate::modules::CommunicationModule;

#[cfg(feature = "platform_virt")]
mod virt;

pub fn current() -> impl Platform {
    #[cfg(feature = "platform_virt")]
    virt::VirtPlatform
}

pub trait Platform {
    fn get_communication_module(&self) -> Option<&'static mut dyn CommunicationModule>;

    fn suspend(&self, code: u32) -> !;
}
