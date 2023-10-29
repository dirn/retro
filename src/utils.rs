use std::env::var;
use std::process::{exit, Command};

pub fn capture_output<'a>(command: &'a mut Command, expected_message: &'a str) -> String {
    let output = command.output().expect(expected_message);
    let result = match String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    result.trim().to_string()
}

pub fn env_or_exit(name: &str) -> String {
    return match var(name) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{name:?}: {error}");
            exit(1);
        }
    };
}
