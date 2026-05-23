/// Builds the test ROM for test `TEST_NAME`.
fn main() {
    let test = std::env::var("TEST_NAME").unwrap_or("Dummy".to_string()); //.expect("TEST_NAME environment variable is not set");

    // Generate code that exports the test as `CurrentTest`

    std::fs::write(
        format!("{}/current_test.rs", std::env::var("OUT_DIR").unwrap()),
        format!("pub use {test}::{test} as CurrentTest;"),
    )
    .expect("failed to generate test module");
}
