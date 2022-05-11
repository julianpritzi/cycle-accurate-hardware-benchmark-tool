# Cycle accurate benchmarking of security critical operations in hardware

The top level folder structure is as follows:

- `benchmarks/` - collection of benchmark description files that are used to perform benchmarks
- `cli/` - a command line interface tool that connects with the suite and controls it
- `common/` - a crate used by both the cli & suite that defines the communication between them
- `suite/` - the benchmarking suite running on the FPGA and performing the actual benchmarks

## Reproducibility

To run all benchmarks present in the `benchmarks/` folder the `reproduce.py` script can be used.
The script only requires python and nix to be installed. All further dependencies are downloaded if needed.

Simply run the following command:
```console
python3 reproduce.py
```

## Benchmarking Suite

### Structure

The suite is written to run on a single `riscv32imc` [Ibex Core](https://github.com/lowRISC/ibex).

The code of the benchmarking suite uses two main abstractions:

- `suite/src/modules/`: Modules that communicate with HWIP (ex. UART, future opentitan HWIPs)
- `suite/src/platform/`: Combines the modules to create different platforms to compile for (ex. qemu virt board, FPGA board)

### Manual Usage

The benchmarking suite can be built using `cargo build`.

**Running/Testing using the Qemu emulator:**

Simply use `cargo run-qemu` or `cargo test-qemu`. \
The serial-output of the uart will be printed to stdout by default. Using `-s pty` as an argument a pty is used instead.

**Running/Testing using Opentitan & Verilator:**
1. Use the Opentitan project to build the earlgrey chip, the test_rom and the otp_img, for the verilator target.
2. Set the environment variables `VERILATOR_SIM`, `VERILATOR_ROM`, `VERILATOR_OTP` to the resulting artifacts.
   ```console
   export VERILATOR_SIM=/path/to/opentitan/build-bin/hw/top_earlgrey/Vchip_earlgrey_verilator
   export VERILATOR_ROM=/path/to/opentitan/build-bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.scr.39.vmem
   export VERILATOR_OTP=/path/to/opentitan/build-bin/sw/device/otp_img/otp_img_sim_verilator.vmem
   ```
3. Use `cargo run-verilator` or `cargo test-verilator`. \
   Note that the verilator test does not stop execution, the result can only be determined by reading from the pty. 

## Benchmarking CLI

This repository currently contains an early version of the benchmarking CLI.

This version of the CLI will read the input_file line by line.
Each line is then parsed as a Message that should be sent directly to the Suite.
Every message read from the suite is output as is.
This mode of operation is referred to as 'raw mode' and may be used in the future for manual testing.
