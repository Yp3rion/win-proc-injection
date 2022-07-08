mod injector;
mod input;
mod type_conversions;
mod errors;

use errors::InjectionError;
use injector::Injector;
use input::get_user_input;
use std::error::Error;
use std::process::ExitCode;

// Ask the user for the name of the process to inject and a path to the library that must be injected,
// then attempt to perform the injection (may perform multiple attempt depending on whether errors are
// encountered or not)
fn try_main() -> Result<(), Box<dyn Error>> {

    let target_process = get_user_input("Target process name: ")?;
    let lib_path = get_user_input("Path of library to inject: ")?;

    let mut injector = Injector::new(target_process, lib_path); 

    unsafe {

        loop {

            println!("\nINJECTION ATTEMPT");

            match injector.inject() {

                Err(inject_err) => {

                    match inject_err {
                        InjectionError::ProcNotFound(_) => {

                            eprintln!("[ERR] {}", inject_err);
                            println!("Trying to start process from executable...");
                            match injector.start_target_process() {
                                Err(fail_start_process) => {
                                    eprintln!("[ERR] {}", fail_start_process);
                                    injector.set_target_process(get_user_input("Enter a new target process: ")?);
                                },
                                Ok(()) => (),
                            }

                        },
                        InjectionError::LibNotFound(_) => {

                            eprintln!("[ERR] {}", inject_err);
                            injector.set_lib_path(get_user_input("Enter the path to an existing lib: ")?);

                        },
                        InjectionError::ProcError(_) => {

                            eprintln!("[ERR] {}", inject_err);
                            injector.set_target_process(get_user_input("Enter a new target process: ")?);

                        },
                        _ => return Err(inject_err.into()),
                    }

                },
                Ok(()) => {
                    println!("Injection successful! Check your C:\\Users\\Public folder...");
                    break;
                },

            }

        }

        Ok(())

    }

}

// This pattern simplifies error handling by delegating the application logic to the try_main function, which returns a
// Result. Depending on whether try_main completed successfully or not, main will terminate with a different exit code.
fn main() -> std::process::ExitCode {
    let mut exit_code = ExitCode::SUCCESS;

    if let Err(critical_err) = try_main() {
        eprintln!("[ERR] {}", critical_err);
        exit_code = ExitCode::FAILURE;
    }

    println!("Terminating the process...");

    exit_code
}