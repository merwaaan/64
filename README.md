# 64

# Test suite

A Nintendo 64 test suite that records behaviors on real hardware and bundles the collected data into self-contained replay ROMs, for emulator development and research.

## Rationale

Existing test suites like [n64-systemtest](https://github.com/lemmy-64/n64-systemtest) and [N64 Bare Metal](https://github.com/PeterLemon/N64) are invaluable tools for emulator developers. However, writing tests for those implies some form of emulation:

1. Implementing a test requires _emulating_ part of the system to formulate assertions
2. The test is ran on real hardware to verify its implementation
3. The test is ran on emulators to check accuracy against hardware

This project takes a complementary data-driven approach where data is recorded _en masse_ on real hardware:

- Each test can be compiled in two modes: **record** or **replay**
- **Record mode**: the test runs on real hardware and emits various measurements (referred as _steps_). A step can be the value of a specific register, the contents of a memory region, or a condition like an exception triggering. These steps are streamed to a PC server as the test runs. A record-mode ROM does not fail or succeed, it does not assert behaviors, it just records steps.
- **Replay mode**: once a test's steps have been recorded, they are embedded into a companion replay-mode ROM. When run (typically in an emulator), this ROM follows the same code path and compares its own runtime steps agains the recorded ones.

> [!NOTE]  
> This project currently requires the [SummerCart64](https://summercart64.dev/) flashcart to record tests. Replaying tests in emulators does not require any hardware.

## For emulator developers

Replay-mode ROMS are available on the Releases page TODO link.

Load any `[TestName].z64` ROM into your emulator to see whether it matches hardware and where any divergence occurs.

### Graphic display

Test results are written directly to the framebuffer (320x240 pixels, 16-bit colors) using the CPU only (no RSP/RDP emulation required).

### IS-Viewer interface

Test results are also printed to an IS-VIEWER-compatible debug interface (TODO link), which is helpful if display has not been emulated yet or if users prefer string output for automation.

To print messages, the test suite writes text data in the PI region, from `0x13FF_0020` up to `0x13FF_0220`, and then writes the text length as a u32 to `0x13FF_0014` to signal a flush.

To support this protocol in your emulator:

- intercept writes to the staging area
- read `length` bytes from the buffered data whenever `0x13FF_0014` is written to

## For test developers/N64 researchers

For implementing new tests or building custom test sets, use `test-suite-server`, which orchestrates ROM compilation, test recording, data embedding in the replay-mode ROMs, and test validation.

> [!TIP]  
> For all the following commands, omitting `--filter` runs the operation for all available tests.

### Build a record-mode ROM

`cargo run -p test-suite-server build record --filter TestName`

This command produces a `[TestName].record.z64` ROM, for execution on real hardware.

> [!TIP]  
> Even without SummerCart64, record-mode ROMs can still be ran on emulators for debugging.

### Record on real hardware and collect data

`cargo run -p test-suite-server record --filter TestName`

This command expects the record-mode ROM to have been built beforehand.

It uploads the record-mode ROM to the SummerCart64 using `sc64deployer`, collects the results, and dumps them to `[TestName].json`.

### Build a replay-mode ROM

`cargo run -p test-suite-server build replay --filter TestName`

This command expects the test to have been recorded beforehand on hardware and dumped as JSON.

It produces a `[TestName].z64` ROM that you can then load in your own emulator.

### Run the full pipeline in one step

`cargo run -p test-suite-server all --filter TestName`

This builds the record-mode ROM, runs it on hardware, collects data, and embeds the data in the replay-mode ROM.

# Credits

- The N64 documentation and preservation community, [N64brew](https://n64brew.dev/wiki/Main_Page), [ultra64](https://ultra64.ca/)
- [n64-systemtest](https://github.com/lemmy-64/n64-systemtest) and [N64 Bare Metal](https://github.com/PeterLemon/N64) for inspiring this project
- [Libdragon][https://github.com/DragonMinded/libdragon] for their open-source IPL3, included in test ROMs
- [nust64](https://github.com/rust-n64/nust64) for their Rust-to-N64 build process that this project started from
