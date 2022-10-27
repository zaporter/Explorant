#!/bin/sh

ORIGINAL_DIRECTORY=$(pwd)
rm -rf test-executables/build/*
build_executable()
{
    PROGRAM_NAME=$1
    cd test-executables/$PROGRAM_NAME
    cargo build 
    cp target/debug/$PROGRAM_NAME ../build
    cd $ORIGINAL_DIRECTORY

}
build_executable date_viewer
build_executable many_threads
