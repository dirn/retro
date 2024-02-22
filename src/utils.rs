use std::env::var;
use std::path::PathBuf;
use std::process::{exit, Command};

use log::error;

pub fn capture_output<'a>(command: &'a mut Command, expected_message: &'a str) -> String {
    let output = command.output().expect(expected_message);
    let result = match String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    result.trim().to_string()
}

pub fn env_or_exit(name: &str) -> String {
    match var(name) {
        Ok(value) => value,
        Err(error) => {
            error!("{name:?}: {error}");
            exit(1);
        }
    }
}

pub fn find_files(root: PathBuf, extensions: &[String]) -> Vec<PathBuf> {
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

// Adapted from https://users.rust-lang.org/t/is-this-code-idiomatic/51798/2.
pub fn longest_common_prefix(vals: &[String]) -> &str {
    if vals.is_empty() {
        return "";
    }

    let common = &vals[0];

    for (i, c) in common.chars().enumerate() {
        for val in vals {
            if val.chars().nth(i) != Some(c) {
                return &common[..i];
            }
        }
    }

    common
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_longest_common_prefix_with_no_strings() {
        assert_eq!(longest_common_prefix(&[]), "");
    }

    #[test]
    fn test_longest_common_prefix_with_one_string() {
        assert_eq!(longest_common_prefix(&["abc".to_string()]), "abc");
    }

    #[test]
    fn test_longest_common_prefix_with_one_string_twice() {
        assert_eq!(
            longest_common_prefix(&["abc".to_string(), "abc".to_string()]),
            "abc"
        );
    }

    #[test]
    fn test_longest_common_prefix_with_one_string_that_is_a_substring_of_the_other() {
        assert_eq!(
            longest_common_prefix(&["abc".to_string(), "abcdef".to_string()]),
            "abc"
        );
        assert_eq!(
            longest_common_prefix(&["abcdef".to_string(), "abc".to_string()]),
            "abc"
        );
    }

    #[test]
    fn test_longest_common_prefix_with_no_common_prefix() {
        assert_eq!(
            longest_common_prefix(&["abc".to_string(), "def".to_string()]),
            ""
        );
    }

    #[test]
    fn test_longest_common_prefix_with_some_strings() {
        assert_eq!(
            longest_common_prefix(&[
                "abcdef".to_string(),
                "abcdeg".to_string(),
                "abcdhi".to_string(),
                "abcjkl".to_string(),
            ]),
            "abc"
        );
    }
}

pub fn require_command(command: &str) -> Command {
    if let Ok(output) = Command::new("which").arg(command).output() {
        if output.status.success() {
            return Command::new(command);
        }
    }

    panic!("{command} not found");
}

pub fn stream_output(command: &mut Command, expected_message: &str) {
    let mut child = command.spawn().expect(expected_message);
    let exit_status = child.wait().expect(expected_message);
    if !exit_status.success() {
        exit(exit_status.code().unwrap());
    }
}
