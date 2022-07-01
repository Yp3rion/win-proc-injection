mod injector;
mod input;
mod type_conversions;

use injector::Injector;
use input::get_line_from_input;

fn main() {

    let target_process = get_line_from_input("Process name: ");
    let lib_path = get_line_from_input("Path to library: ");

    let injector = Injector::new(target_process, lib_path); 

    unsafe {

        injector.inject();

    }

}
