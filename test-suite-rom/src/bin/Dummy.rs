//! Does nothing, just a dummy test to validate the recording mechanism

#![no_std]
#![no_main]

test_suite_rom::run_test! {
    TestWithParams Dummy {
        type Params = bool;

        fn cases() -> Vec<Self::Params> {
            vec![true, false]
        }

        fn case_name(params: &Self::Params) -> String {
            format!("Dummy case: {}", params)
        }

        fn run(params: &Self::Params, result: &mut TestCaseResult) {
            result.push_comment(&format!("Dummy test result with {}", params));
            result.push_value(if *params { 1 } else { 0 });
            result.push_pc();

            // TODO write some data to make this deterministic?

            for i in 0..10 {
                 result.push_memory(i * 4);
             }
        }
    }
}
