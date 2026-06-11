#!/bin/bash

mkdir -p _release

cargo run -p test_suite_server clean

REPEAT=3

# Master ROM

#cargo run -p test_suite_server all --merge all --repeat $REPEAT

# Grouped ROMs

# cargo run -p test_suite_server all --match Ai --merge ai --repeat $REPEAT
# cargo run -p test_suite_server all --match Cop0 --merge cop0 --repeat $REPEAT
# cargo run -p test_suite_server all --match CpuInstr --merge cpu --repeat $REPEAT
# cargo run -p test_suite_server all --match Mi --merge mi --repeat $REPEAT
# cargo run -p test_suite_server all --match Rsp --merge rsp --repeat $REPEAT
# cargo run -p test_suite_server all --match Vi --merge vi --repeat $REPEAT

# temp test
cargo run -p test_suite_server all --match Logical --merge logical --repeat $REPEAT

# Individual ROMs

#cargo run -p test_suite_server all --repeat $REPEAT
