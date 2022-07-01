use std::io;
use std::io::*;

pub fn get_line_from_input(prompt: &str) -> String {

    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error");
    input.truncate(input.trim_end_matches(&['\r', '\n']).len());
    input

}