#![no_std]
#![no_main]

use alloc::{string::String, vec::Vec};

// Does nothing, just a dummy test to validate the recording mechanism

test_suite_rom::define_test! {
    Dummy {
        type Params = bool;

        fn cases() -> Vec<Self::Params> {
            alloc::vec![true, false]
        }

        fn case_name(params: &Self::Params) -> Option<String> {
            Some(alloc::format!("Dummy case: {}", params))
        }

        fn run_case(params: &Self::Params, result: &mut TestCaseResult) {

            result.push_comment(alloc::format!("Dummy test result with {}", params).as_str());
            result.push_value(if *params { 1 } else { 0 });
            result.push_pc();

            // TODO write some data to make this deterministic

            for i in 0..10 {
                result.push_memory(i * 4);
            }
        }
    }
}
