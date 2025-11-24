use std::env::{current_dir, var, VarError};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use log::error;

pub fn capture_output<'a>(
    command: &'a mut Command,
    expected_message: &'a str,
) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|e| format!("{}: {}", expected_message, e))?;

    let mut result = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to decode UTF-8 in command output: {}", e))?;

    // Trim trailing whitespace in-place to avoid allocation
    result.truncate(result.trim_end().len());
    Ok(result)
}

pub fn find_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files_found = Vec::new();
    let entries = root
        .read_dir()
        .map_err(|e| format!("Failed to read directory {}: {}", root.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "Failed to read directory entry in {}: {}",
                root.display(),
                e
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            files_found.append(&mut find_files(&path)?);
        } else {
            files_found.push(path);
        }
    }

    Ok(files_found)
}

pub fn find_files_with_extension(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut files_found = Vec::new();
    for file in find_files(root)? {
        if let Some(extension) = file.extension() {
            if let Some(extension) = extension.to_str() {
                if extensions.iter().any(|ext| *ext == extension) {
                    files_found.push(file);
                }
            }
        }
    }

    Ok(files_found)
}

pub fn find_file_recursively(root: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    let mut path: PathBuf = root.into();
    if path == PathBuf::from(".") {
        path = current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    }
    let file = Path::new(name);

    loop {
        path.push(file);

        if path.is_file() {
            break Ok(Some(path));
        }

        if !(path.pop() && path.pop()) {
            break Ok(None);
        }
    }
}

pub fn get_from_env(name: &str) -> Result<String, VarError> {
    var(name)
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

pub fn require_command(command: &str) -> Result<Command, String> {
    if let Ok(output) = Command::new("which").arg(command).output() {
        if output.status.success() {
            return Ok(Command::new(command));
        }
    }

    Err(format!("Failed to find command '{}' in PATH", command))
}

pub fn stream_output(command: &mut Command, expected_message: &str) -> Result<(), String> {
    let mut child = command
        .spawn()
        .map_err(|e| format!("{}: {}", expected_message, e))?;

    let exit_status = child
        .wait()
        .map_err(|e| format!("Failed to wait for command: {}", e))?;

    if !exit_status.success() {
        let code = exit_status.code().unwrap_or(1);
        return Err(format!("Failed to execute command: exit code {}", code));
    }

    Ok(())
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

        let result = find_file_recursively(ctx.root.path(), "test").unwrap();
        assert_eq!(None, result);
    }

    #[test_context(Context)]
    #[test]
    fn find_file_recursively_finds_file_in_grandparent(ctx: &mut Context) {
        let file_path = ctx.root.path().join("test");
        let _ = File::create(&file_path).unwrap();
        let grandchild_path = ctx.root.path().join("child").join("grandchild");

        let result = find_file_recursively(&grandchild_path, "test").unwrap();
        assert_eq!(file_path, result.unwrap());
    }

    #[test_context(Context)]
    #[test]
    fn find_file_recursively_finds_file_in_parent(ctx: &mut Context) {
        let file_path = ctx.root.path().join("test");
        let _ = File::create(&file_path).unwrap();
        let child_path = ctx.root.path().join("child");

        let result = find_file_recursively(&child_path, "test").unwrap();
        assert_eq!(file_path, result.unwrap());
    }

    #[test_context(Context)]
    #[test]
    fn find_file_recursively_finds_file_in_root(ctx: &mut Context) {
        let file_path = ctx.root.path().join("test");
        let _ = File::create(&file_path).unwrap();

        let result = find_file_recursively(ctx.root.path(), "test").unwrap();
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
