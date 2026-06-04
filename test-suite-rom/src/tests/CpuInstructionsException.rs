//! SYSCALL
//! TODO others

use n64_specs::cpu::instructions::Syscall;

use crate::{
    app::App,
    exceptions::{ExceptionTracker, install_exception_handler},
    no_params,
    program::Program,
    register_test,
    test::{Test, TestError},
};

register_test!(CpuInstructionSyscall);

impl Test for CpuInstructionSyscall {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        let exception_tracker = install_exception_handler(ExceptionTracker::new());

        Program::new().push(Syscall::default().into()).run();

        app.bool("Exception occurred", exception_tracker.occurred)?;
        app.bool("Syscall exception occurred", exception_tracker.syscall)?;

        // TODO delay slot case?

        Ok(())
    }
}
