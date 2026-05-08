#![no_std]
#![no_main]

test_suite_rom::run_test! {
    TestNoParams MiVersionRegisterValue {
        fn run(result: &mut TestCaseResult) {
            let value = unsafe {
                reg_mut_ptr(specs::mi::Version::ADDRESS).read_volatile()
            };

            result.push_value(value);
        }
    }
}
