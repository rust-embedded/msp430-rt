#!/bin/bash

set -euxo pipefail

# cflags taken from cc 1.0.22

crate=msp430-rt

# remove existing blobs because otherwise this will append object files to the old blobs
rm -f bin/*.a

msp430-elf-as -mcpu=msp430 asm.s -o bin/$crate.o
ar crs bin/msp430-none-elf.a bin/$crate.o

rm bin/$crate.o
