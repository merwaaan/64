//! Dummy test to validate the recording mechanism and various helpers.

#![no_std]
#![no_main]

test_suite_rom::run_test!(Dummy);

impl Test for Dummy {
    type Params = bool;

    fn cases() -> Vec<Self::Params> {
        Vec::from([true, false])
    }

    fn case_name(params: &Self::Params) -> String {
        format!("Dummy case: {}", params)
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<()> {
        app.push_comment("A helpful comment")?;
        app.push_value(if *params { u32::MAX } else { 0 })?;
        app.push_pc()?;

        let some_ram_data = (0..1000).map(|i| i as u32).collect::<Vec<_>>();

        app.push_memory_region(
            some_ram_data.as_ptr() as u32,
            some_ram_data.len() as u32 * 4,
        )?;

        for i in 100..110 {
            app.push_memory(unsafe { some_ram_data.as_ptr().add(i) as u32 })?;
        }

        Ok(())
    }
}
