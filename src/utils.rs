use std::env::var;
use std::path::PathBuf;
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

pub fn find_files(root: PathBuf, extensions: Vec<String>) -> Vec<PathBuf> {
    let mut files_found = Vec::new();
    for file in root.read_dir().unwrap() {
        let path = file.unwrap().path();
        if let Some(extension) = path.extension() {
            if let Some(extension) = extension.to_str() {
                if extensions.contains(&extension.to_string()) {
                    files_found.push(path);
                }
            }
        }
    }

    files_found
}
