#!/usr/bin/env bash

set -e

cargo run --release -- --log-from $1 --log-to $2 "$3" || true

../project64/Bin/x64/Release/Project64.exe --log_from $1 --log_to $2 "$3"

# cargo run -- -b 0x800000c8 sm.n64
# ./diff.sh 0 100000

# ./diff.sh 6000000 8000000 Wave\ Race\ 64\ -\ Kawasaki\ Jet\ Ski\ \(Europe\)\ \(En\,De\).z64 