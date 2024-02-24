use std::env::{current_dir, var, VarError};
use std::path::{Path, PathBuf};
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

pub fn find_file_recursively(root: &Path, name: &str) -> Option<PathBuf> {
    let mut path: PathBuf = root.into();
    if path == PathBuf::from(".") {
        path = current_dir().unwrap();
    }
    let file = Path::new(name);

    loop {
        path.push(file);

        if path.is_file() {
            break Some(path);
        }

        if !(path.pop() && path.pop()) {
            break None;
        }
    }
}

pub fn get_from_env(name: &str) -> Result<String, VarError> {
    match var(name) {
        Ok(value) => Ok(value),
        Err(e) => Err(e),
    }
}

pub fn get_from_env_or_exit(name: &str) -> String {
    match var(name) {
        Ok(value) => value,
        Err(error) => {
            error!("{name:?}: {error}");
            exit(1);
        }
    }
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

#[cfg(test)]
mod test {
    use std::fs::{create_dir, File};

    use tempdir::TempDir;
    use test_context::{test_context, TestContext};

    use super::*;

    struct Context {
        root: TempDir,
    }

    impl TestContext for Context {
        fn setup() -> Self {
            Self {
                root: TempDir::new("tmp").unwrap(),
            }
        }

        fn teardown(self) {
            println!("teardown");
            self.root.close().unwrap();
        }
    }

    #[test_context(Context)]
    #[test]
    fn find_file_recursively_does_not_find_file_in_child(ctx: &mut Context) {
        let _ = create_dir(ctx.root.path().join("child")).unwrap();
        let file_path = ctx.root.path().join("child").join("test");
        let _ = File::create(&file_path).unwrap();

        let result = find_file_recursively(ctx.root.path(), "test");
        assert_eq!(None, result);
    }

    #[test_context(Context)]
    #[test]
    fn find_file_recursively_finds_file_in_grandparent(ctx: &mut Context) {
        let file_path = ctx.root.path().join("test");
        let _ = File::create(&file_path).unwrap();
        let grandchild_path = ctx.root.path().join("child").join("grandchild");

        let result = find_file_recursively(&grandchild_path, "test");
        assert_eq!(file_path, result.unwrap());
    }

    #[test_context(Context)]
    #[test]
    fn find_file_recursively_finds_file_in_parent(ctx: &mut Context) {
        let file_path = ctx.root.path().join("test");
        let _ = File::create(&file_path).unwrap();
        let child_path = ctx.root.path().join("child");

        let result = find_file_recursively(&child_path, "test");
        assert_eq!(file_path, result.unwrap());
    }

    #[test_context(Context)]
    #[test]
    fn find_file_recursively_finds_file_in_root(ctx: &mut Context) {
        let file_path = ctx.root.path().join("test");
        let _ = File::create(&file_path).unwrap();

        let result = find_file_recursively(ctx.root.path(), "test");
        assert_eq!(file_path, result.unwrap());
    }

    #[test]
    fn longest_common_prefix_returns_no_prefix_with_no_strings() {
        assert_eq!(longest_common_prefix(&[]), "");
    }

    #[test]
    fn longest_common_prefix_returns_full_string_with_one_string() {
        assert_eq!(longest_common_prefix(&["abc".to_string()]), "abc");
    }

    #[test]
    fn longest_common_prefix_returns_full_string_with_one_string_twice() {
        assert_eq!(
            longest_common_prefix(&["abc".to_string(), "abc".to_string()]),
            "abc"
        );
    }

    #[test]
    fn longest_common_prefix_returns_shorter_string_with_one_string_that_is_a_substring_of_the_other(
    ) {
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
    fn longest_common_prefix_returns_no_prefix_with_no_common_prefix() {
        assert_eq!(
            longest_common_prefix(&["abc".to_string(), "def".to_string()]),
            ""
        );
    }

    #[test]
    fn longest_common_prefix_returns_prefix_with_some_strings() {
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
