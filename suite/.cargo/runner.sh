#!/bin/bash

# Utility Script for running the benchmarking suite

# Variables for styling output:
FNT_BOLD=$(tput bold 2>/dev/null)
FNT_NORMAL=$(tput sgr0 2>/dev/null)
FNT_RED=$(tput setaf 1 2>/dev/null)
FNT_GREEN=$(tput setaf 2 2>/dev/null)

# Variables
USE_SIMULATOR="none"
SERIAL="stdio"
KERNEL=$1

shift
while getopts "s:vq" arg
do
    case ${arg} in
        s) # sets a custom serial argument if possible
            SERIAL=$OPTARG
            ;;
        v) # specify to use verilator
            USE_SIMULATOR="verilator"
            ;;
        q) # specify to use qemu 
            USE_SIMULATOR="qemu"
            ;;
        ?)
            echo "${FNT_BOLD}${FNT_RED}runner-error${FNT_NORMAL}: Invalid arguments"
            exit 1
            ;;
    esac
done

case "$USE_SIMULATOR" in
    "verilator")
        echo "       ${FNT_BOLD}${FNT_GREEN}using${FNT_NORMAL} verilator"
        if [[ -z "$VERILATOR_SIM" ]]; then
            echo "${FNT_BOLD}${FNT_RED}runner-error${FNT_NORMAL}: set up the VERILATOR_SIM environment variable to point to the chip simulator binary"
            USE_SIMULATOR="none"
        fi
        if [[ -z "$VERILATOR_ROM" ]]; then
            echo "${FNT_BOLD}${FNT_RED}runner-error${FNT_NORMAL}: set up the VERILATOR_ROM environment variable to point to the file that should be loaded into rom"
            USE_SIMULATOR="none"
        fi
        if [[ -z "$VERILATOR_OTP" ]]; then
            echo "${FNT_BOLD}${FNT_RED}runner-error${FNT_NORMAL}: set up the VERILATOR_OTP environment variable to point to the file that should be loaded into otp"
            USE_SIMULATOR="none"
        fi

        if [[ $USE_SIMULATOR == "verilator" ]]; then
            cp $KERNEL $KERNEL.elf
            $VERILATOR_SIM --meminit=rom,$VERILATOR_ROM --meminit=flash,$KERNEL.elf --meminit=otp,$VERILATOR_OTP
        else
            exit 2
        fi
        ;;
    "qemu")
        echo "       ${FNT_BOLD}${FNT_GREEN}using${FNT_NORMAL} qemu"
        qemu-system-riscv32 \
            -M virt \
            -cpu rv32 \
            -smp 1 \
            -m 32M \
            -display none \
            -bios none \
            -serial $SERIAL \
            -kernel $KERNEL
        ;;
    *)
        echo "use 'cargo run-<arch>' or 'cargo test-<arch>' instead, supported archs:"
        echo "  qemu"
        echo "  verilator"
        exit 2
        ;;
esac
