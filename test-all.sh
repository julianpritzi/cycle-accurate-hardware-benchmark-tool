#!/bin/bash

# Runs the benchmarking suite tests
# Then runs qemu emulating the benchmark suite in the background exposing the communication module to a pty
# The runs the cli-tool tests using the pty of qemu

set -e

bash -c "cd suite && cargo test"

coproc suite_emulator (cd suite && cargo emulate)

while read -r line; do
    TTY_PATH=$(echo $line | sed -r "s/char device redirected to (.*) \(.*/\1/")
    
    (export BENCHMARK_TEST_TTY=$TTY_PATH && cd cli && cargo test)
    pkill qemu-system-ris
done <&"${suite_emulator[0]}"
