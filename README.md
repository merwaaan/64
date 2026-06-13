# N64 Replay Test

A Nintendo 64 test suite that records behaviors on real hardware and bundles the collected data into self-contained replay ROMs, for emulator development and research.

💾 **Replay-mode ROMS are available on the [Releases page](http://todo.link).**

## Rationale

Existing test suites like [n64-systemtest](https://github.com/lemmy-64/n64-systemtest) and [N64 Bare Metal](https://github.com/PeterLemon/N64) are invaluable tools for emulator developers. However, writing tests for those implies some form of emulation:

1. Implementing a test requires _emulating_ part of the system to formulate assertions.
2. The test must be ran on real hardware to verify its implementation.
3. The test can then be ran on emulators to check accuracy against hardware.

This project takes a complementary data-driven approach where data is recorded _en masse_ on real hardware:

- Each test can be compiled in two modes: **record** or **replay**.
- **Record mode**
  - The test runs on real hardware and emits various measurements (referred as _steps_).
  - A step can be the value of a specific register, the contents of a memory region, or a condition like an exception triggering.
  - These steps are streamed to a PC server as the test runs.
  - A record-mode ROM does not fail or succeed, it does not assert behaviors, it just records steps.
- **Replay mode**
  - Once a test's steps have been recorded, they are embedded into a companion replay-mode ROM.
  - When run (typically in an emulator), this ROM follows the same code path and compares its own runtime steps against those recorded on hardware.

> [!NOTE]  
> This project currently requires a [SummerCart64](https://summercart64.dev/) flashcart to **record** tests.
>
> Replaying tests in emulators does not require any hardware.

## For emulator developers

Load any `[test name].z64` ROM into your emulator to see whether it matches hardware and where any divergence occurs.

### Graphic display

Test results are written directly to the framebuffer (320x240 pixels, 16-bit colors) using the CPU only (no RSP/RDP emulation required).

### IS-Viewer interface

Test results are also printed to an IS-VIEWER-like debug interface (TODO link), which is helpful if display has not been emulated yet or if users prefer string output for automation.

To print messages, the test suite writes text data in the PI region, from `0x13FF_0020` up to `0x13FF_0220`, and then writes the text length as a u32 to `0x13FF_0014` to signal a flush.

To support this protocol in your emulator:

- Intercept writes from `0x13FF_0020` to `0x13FF_0220`.
- Intercept writes to `0x13FF_0014` to detect flushes and read `n` bytes from that buffered data, where `n`is the written value.

## For test developers/N64 researchers

Tests are stored under `test-suite-rom/tests`.

Each module can contain one or several tests. Tests are identified as `module::name`, eg. `AiDma::AiDmaQueue`.

A test is a struct that implements the `Test` trait and it must be registered with the `register_test!()` macro.

For implementing new tests or building custom test sets, use `test-suite-server`, which orchestrates ROM compilation, test recording, data embedding in the replay-mode ROMs, and test validation.

`> cargo run -p test-suite-server --help`

### List all the available tests

`> cargo run -p test-suite-server list`

### Run the full build pipeline in one step

`> cargo run -p test-suite-server all`

- Builds the record-mode ROM.
- Runs it on hardware and collects data.
- Buils the replay-mode ROM with the recorded data.
- Runs it back on hardware for validation.

By default, **each** available test is built as a separate ROM.

**Options**

- `[exact name]`: select the test named `[exact name]`, can be the test name only or `ModuleName::TestName`.
- `--match [keyword]`: selects tests that contain `keyword` in their `ModuleName::TestName`, multiple `--match` can be specified.
- `--merge [rom name]`: produces a single `[rom name].z64` ROM running the selected tests in sequence.

### Build a record-mode ROM

`> cargo run -p test-suite-server build --mode record`

Produces `[test name].record.z64` ROMs.

The same match and merge options as `all` are available.

> [!TIP]  
> Even without SummerCart64, record-mode ROMs can still be ran on emulators for debugging.

### Record on real hardware and collect data

`> cargo run -p test-suite-server record [test name]`

This command expects the record-mode ROM to have been built beforehand.

It uploads the record-mode ROM to the SummerCart64 using `sc64deployer`, collects the results, and dumps them to `[test name].json`.

### Build a replay-mode ROM

`> cargo run -p test-suite-server build --mode replay`

This command expects test data to have been recorded and dumped to `[test name].json` beforehand.

It produces a `[TestName].z64` ROM that you can then load in your own emulator.

The same match and merge options as `all` are available.

# Credits

- The N64 documentation and preservation community, [N64brew](https://n64brew.dev/wiki/Main_Page), [ultra64](https://ultra64.ca/)
- [n64-systemtest](https://github.com/lemmy-64/n64-systemtest) and [N64 Bare Metal](https://github.com/PeterLemon/N64) for inspiring this project
- [Libdragon](https://github.com/DragonMinded/libdragon) for their open-source IPL3, included in test ROMs
- [nust64](https://github.com/rust-n64/nust64) for their Rust-to-N64 build process that this project started from
