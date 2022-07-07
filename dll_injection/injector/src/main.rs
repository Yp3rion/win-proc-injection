mod injector;
mod input;
mod type_conversions;
mod errors;

use errors::InjectionError;
use injector::Injector;
use input::get_line_from_input;

fn main() {

    let target_process = get_line_from_input("Target process name: ");
    let lib_path = get_line_from_input("Path of library to inject: ");

    let mut injector = Injector::new(target_process, lib_path); 

    unsafe {

        loop {

            println!("\nINJECTION ATTEMPT");

            match injector.inject() {

                Err(inject_err) => {

                    eprintln!("[ERR] {}", inject_err);

                    match inject_err {
                        InjectionError::ProcNotFound(_) => {

                            println!("Trying to start process from executable...");
                            match injector.start_target_process() {
                                Err(fail_start_process) => {
                                    eprintln!("[ERR] {}", fail_start_process);
                                    injector.set_target_process(get_line_from_input("Enter a new target process: "));
                                },
                                Ok(()) => (),
                            }
                            continue;

                        },
                        InjectionError::LibNotFound(_) => {

                            injector.set_lib_path(get_line_from_input("Enter the path to an existing lib: "));
                            continue;

                        },
                        InjectionError::ProcError(_) => {

                            injector.set_target_process(get_line_from_input("Enter a new target process: "));

                        },
                        _ => break,
                    }

                },
                Ok(()) => {
                    println!("Injection successful! Check your C:\\Users\\Public folder...");
                    break;
                },

            }

        }

        println!("Terminating the process...");

    }

}
