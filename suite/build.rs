use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Customizes the build process of the suite to use the appropriate memory file for linking.
///
/// For more information see:
/// - https://doc.rust-lang.org/cargo/reference/build-scripts.html
/// - https://docs.rs/riscv-rt/latest/riscv_rt/
fn main() {
    let out_dir = env::var("OUT_DIR").expect("No out dir");
    let dest_path = Path::new(&out_dir);
    let mut f = File::create(&dest_path.join("memory.x")).expect("Could not create file");

    #[cfg(feature = "platform_qemu_virt")]
    let memory_information = include_bytes!("memory/qemu_virt.x");
    #[cfg(feature = "platform_verilator_earlgrey")]
    let memory_information = include_bytes!("memory/verilator_earlgrey.x");

    f.write_all(memory_information)
        .expect("Could not write file");

    println!("cargo:rustc-link-search={}", dest_path.display());

    println!("cargo:rerun-if-changed=memory/qemu_virt.x");
    println!("cargo:rerun-if-changed=memory/verilator_earlgrey.x");
    println!("cargo:rerun-if-changed=build.rs");
}
