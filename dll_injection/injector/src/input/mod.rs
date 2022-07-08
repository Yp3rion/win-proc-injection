use std::io;
use std::io::*;

pub fn get_user_input(prompt: &str) -> Result<String> {

    loop {

        print!("{}", prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {

            Err(io_err) => {

                match io_err.kind() {

                    ErrorKind::InvalidData => {
                        eprintln!("[ERR] {}", io_err);
                    },
                    _ => return Err(io_err),

                }

            },
            Ok(_) => (),

        }
        input.truncate(input.trim_end_matches(&['\r', '\n']).len());
        return Ok(input);

    }

}