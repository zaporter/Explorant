#!/usr/bin/env bash
set -eux
gcc \
  -L "$(pwd)/lib" \
  -I "$(pwd)/include" \
  -Wl,--rpath="$(pwd)/lib" \
  -Wl,--dynamic-linker="$(pwd)/lib/ld-linux-x86-64.so.2" \
  -std=c11 \
  -o "$1.out" \
  -v \
  -g3 \
  --static \
  "$1.c" \
  -pthread \
;
#ldd ./test_glibc.out
"./$1.out"
