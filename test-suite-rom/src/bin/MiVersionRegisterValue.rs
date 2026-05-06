#![no_std]
#![no_main]

test_suite_rom::define_test! {
    MiVersionRegisterValue {
        type Params = ();

        fn run_case(_params: &Self::Params, result: &mut TestCaseResult) {
            let value = unsafe {
                reg_mut_ptr(specs::mi::Version::ADDRESS).read_volatile()
            };

            result.push_value(value);
        }
    }
}
