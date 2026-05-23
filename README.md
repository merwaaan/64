# 64

# Test suite

A Nintendo 64 test suite that replays behaviors recorded on real hardware.

## Using the test suite as an emulator developer

The test ROMs are available in the Releases page.

You may be interested in running the `[TestName]\_replay_.z64` ROMs on your emulator.

### Graphic display

The tests print their results on the screen.

The framebuffer is filled with the CPU only, so no need for RSP/RDP emulation.

### IS-Viewer interface

The tests

## Using the test suite as a test developer/N64 researcher

The `test-suite-server` crate orchestrates ROM compilation and recording.

To build a record-mode ROM:

TODO make default?
`cargo run -p test-suite-server build record [XXX]`

To record the execution of a record-mode ROM:

`cargo run -p test-suite-server [XXX]`

To build a replay-mode ROM (requires the steps to have been recorded):

`cargo run -p test-suite-server [XXX]`

To do all of that in a single step:

`cargo run -p test-suite-server all [XXX]`

## Rationale

Existing test suites like xxx and xxx are invaluable help for emulator developers to enhance the fidelity of their projects.

Such test suites are typically structured like so:

- the test developer writes a test, referring to documentation to extrapolate the expected outcome
- the test is ran on real hardware to validate its implementation
- the test is ran on emulators to test their soundness compared to real hardware

This project takes an alternative approach where hardware data is recorded _en masse_:

- Each test can be compiled in two modes: _record_ or _replay_
- The record-mode ROM runs the test and emits various measurements, referred as _steps_. Steps are sent to a server-like process running on a PC. A record-mode ROM does not fail or succeed, it just records.
- Once the data has been recorded, it's embedded into the replay-mode ROM. This one will run the same test and compare its steps agains the recorded ones.

memory

# Credits

- The people behind the n64brew wiki,
- n64 system test + lemon influenced this project
- libdragon for their open-source IPL3 that is part of our test ROMs
- nust64 for their Rust to N64 build process that we
