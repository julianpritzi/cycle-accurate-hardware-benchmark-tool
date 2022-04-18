# Cycle accurate benchmarking of security critical operations in hardware

The benchmarking tool is separated into two rust projects:

- `suite/` - the benchmarking suite running on the FPGA and performing the actual benchmarks
- `cli/` - a command line interface tool that connects with the suite and controls it

## Benchmarking Suite

### Usage

The benchmarking suite can be built using `cargo build`.

To run or test the suite with the qemu emulator use `cargo run` or `cargo test` respectively.

### Structure

The suite is written to run on a single `riscv32imc` [Ibex Core](https://github.com/lowRISC/ibex).

The code of the benchmarking suite uses two main abstractions:

- `suite/src/modules/`: Modules that communicate with HWIP (ex. UART, future opentitan HWIPs)
- `suite/src/platform/`: Combines the modules to create different platforms to compile for (ex. qemu virt board, FPGA board)