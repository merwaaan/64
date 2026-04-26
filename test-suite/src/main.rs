mod tests;

#[cfg(not(any(feature = "collect", feature = "compare")))]
compile_error!("must enable either feature \"collect\" or \"compare\"");

#[cfg(all(feature = "collect", feature = "compare"))]
compile_error!("features \"collect\" and \"compare\" are mutually exclusive");

fn main() {
    let result = tests::ai::RegistersMirroring::run();

    #[cfg(feature = "collect")]
    collect(result);

    #[cfg(feature = "compare")]
    compare(result);
}

#[cfg(feature = "collect")]
fn collect(result: TestResult) {
    // send to USB
}

#[cfg(feature = "compare")]
fn compare(result: TestResult) {
    // compare with collected
}

#[derive(Debug)]
enum State {
    Pc(u32),
    Memory { address: u32, value: u32 },
}

#[derive(Default, Debug)]
struct TestResult {
    states: Vec<State>,
}

trait Test {
    type Params: Sized;

    fn run() -> TestResult;
}
