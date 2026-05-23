/// Builds the test ROM for test `TEST_MODULE::TEST_NAME`.
fn main() {
    println!("cargo:rerun-if-env-changed=TEST_NAME");
    println!("cargo:rerun-if-env-changed=TEST_MODULE");

    let test_module = std::env::var("TEST_MODULE").unwrap_or("Dummy".to_string()); //TODO.expect("TEST_MODULE environment variable is not set");
    let test_name = std::env::var("TEST_NAME").unwrap_or("Dummy".to_string()); //TODO.expect("TEST_NAME environment variable is not set");

    // Generate code that exports the test as `CurrentTest`

    std::fs::write(
        format!("{}/current_test.rs", std::env::var("OUT_DIR").unwrap()),
        format!("pub use {test_module}::{test_name} as CurrentTest;"),
    )
    .expect("failed to generate test module");
}
