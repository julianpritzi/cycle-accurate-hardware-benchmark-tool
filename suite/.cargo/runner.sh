#!/bin/bash

# Utility Script for emulating the benchmarking suite on qemu

SERIAL="stdio"
MEMORY="64K"
KERNEL=$1

shift
while getopts "s:m:" arg
do
    case ${arg} in
        s) # sets a custom serial argument for qemu 
            SERIAL=$OPTARG
            ;;
        s) # sets a custom memory size for qemu 
            MEMORY=$OPTARG
            ;;
        ?)
            echo "Invalid arguments"
            exit 1
            ;;
    esac
done

qemu-system-riscv32 \
    -M virt \
    -cpu rv32 \
    -smp 1 \
    -m $MEMORY \
    -display none \
    -bios none \
    -serial $SERIAL \
    -kernel $KERNEL