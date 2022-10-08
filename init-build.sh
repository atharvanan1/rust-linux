#! /bin/bash

export MAKEFLAGS="-j16"
export LLVM="1"

make allnoconfig qemu-busybox-min.config rust.config
make
make menuconfig
make rustvm
