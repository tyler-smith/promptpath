use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const CONFIG_PATH: &str = ".config/promptpath/config.toml";
const CODE_ROOT: &str = "~/code";
const UNKNOWN: &str = "unknown";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Deserialize, Default, Debug)]
struct Config {
    #[serde(default)]
    projects: Vec<ProjectMapping>,
}

#[derive(Deserialize, Debug)]
struct ProjectMapping {
    path: String,
    alias: String,
}

#[derive(Debug)]
struct AppContext {
    home: PathBuf,
    project_mappings: HashMap<PathBuf, (String, String)>,
}

impl AppContext {
    fn new() -> Self {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .expect("HOME environment variable must be set");

        // Load project mappings from the config file
        let config = Self::load_config(&home).unwrap_or_default();
        let project_mappings = config
            .projects
            .into_iter()
            .map(|mapping| {
                let key = expand_home_alias(&home, &mapping.path);
                (key, (mapping.path, mapping.alias))
            })
            .collect();

        Self {
            home,
            project_mappings,
        }
    }

    fn load_config(home: &Path) -> Result<Config, ConfigError> {
        let config_path = home.join(CONFIG_PATH);
        let contents = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&contents)?)
    }
}

// Get the nickname for a given path
fn get_nickname(ctx: &AppContext, path: PathBuf) -> String {
    // Special cases:
    //   If we're in the home directory, return ~
    //   If we're in the root directory, return /
    if path == ctx.home {
        return "~".to_string();
    }
    if path == PathBuf::from("/") {
        return "/".to_string();
    }

    let nickname = collapse_home_alias(&ctx.home, &path);
    let nickname = collapse_project_alias(ctx, &path, nickname);
    let nickname = collapse_code_alias(nickname);
    strip_trailing_slashes(nickname)
}

// Expands a path that starts with ~/ to an absolute path
fn expand_home_alias(home: &Path, path: &str) -> PathBuf {
    if !path.starts_with("~/") {
        return PathBuf::from(path);
    }

    let mut normalized = home.to_path_buf();
    normalized.push(&path[2..]);
    normalized
}

// Collapses a path that starts with the home directory to ~/...
fn collapse_home_alias(home: &Path, path: &Path) -> String {
    if !path.starts_with(home) {
        return path.to_string_lossy().into_owned();
    }

    let mut result = String::with_capacity(path.as_os_str().len());
    result.push('~');
    result.push('/');
    result.push_str(path.strip_prefix(home).unwrap().to_str().unwrap());
    result
}

// Finds the project that matches the path, or None if no match is found.
// If multiple matches are found we take the one with the longest prefix.
fn collapse_project_alias(ctx: &AppContext, path: &Path, path_str: String) -> String {
    let longest_match = ctx
        .project_mappings
        .iter()
        .filter(|(key, _)| path.starts_with(key))
        .max_by_key(|(key, _)| key.as_os_str().len())
        .map(|(_, (path, alias))| (path, alias));

    match longest_match {
        Some((project_path, alias)) => {
            let shortened = path_str.replacen(project_path, alias, 1);
            let shortened = shortened.trim_start_matches('/');
            shortened.to_string()
        }
        None => path_str,
    }
}

// Collapses a path that starts with the code directory to ~/code/...
fn collapse_code_alias(path: String) -> String {
    if path == CODE_ROOT {
        return path[2..].to_string();
    }

    if !path.starts_with(CODE_ROOT) {
        return path;
    }

    // Attempt to replace the code root with the alias. If no change happens we didn't find a match
    // and we should return the original path. If the new path does not start with a / then return
    // the original path.
    let new_path = path.replacen(CODE_ROOT, "", 1);
    if new_path.eq(&path) {
        return path;
    }
    if !new_path.starts_with('/') {
        return path;
    }
    strip_leading_slashes(new_path)
}

// Strips all leading slashes from a path, except for the root directory
fn strip_leading_slashes(path: String) -> String {
    if path.eq("/") {
        return path;
    }
    path.trim_start_matches('/').to_owned()
}

// Strips all trailing slashes from a path, except for the root directory
fn strip_trailing_slashes(path: String) -> String {
    if path.eq("/") {
        return path;
    }
    path.trim_end_matches('/').to_owned()
}

// Get the nickname for the current working directory
fn get_cwd_nickname(ctx: &AppContext) -> String {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(_) => return UNKNOWN.to_string(),
    };
    get_nickname(ctx, cwd)
}

fn main() {
    let ctx = AppContext::new();
    println!("{}", get_cwd_nickname(&ctx));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct PathTest {
        input: String,
        expected: String,
        description: &'static str,
    }

    fn setup_test_context() -> AppContext {
        let home = PathBuf::from("/Users/tcrypt");
        let project_mappings = {
            let mut m = HashMap::new();
            m.insert(
                PathBuf::from("/Users/tcrypt/code/github.com/tyler-smith/promptpath"),
                (
                    String::from("~/code/github.com/tyler-smith/promptpath"),
                    String::from("promptpath"),
                ),
            );
            m
        };

        AppContext {
            home,
            project_mappings,
        }
    }

    #[test]
    fn test_paths() {
        let test_cases = vec![
            PathTest {
                input: "/".into(),
                expected: "/".into(),
                description: "Root directory",
            },
            PathTest {
                input: "/Users".into(),
                expected: "/Users".into(),
                description: "Users directory",
            },
            PathTest {
                input: "/Users/tcrypt".into(),
                expected: "~".into(),
                description: "Home directory",
            },
            PathTest {
                input: "/Users/tcrypt/data".into(),
                expected: "~/data".into(),
                description: "Directory in home",
            },
            PathTest {
                input: "/Users/tcrypt/data/images".into(),
                expected: "~/data/images".into(),
                description: "Nested directory in home",
            },
            PathTest {
                input: "/Users/tcrypt/code".into(),
                expected: "code".into(),
                description: "Code root directory",
            },
            PathTest {
                input: "/Users/tcrypt/code/github.com".into(),
                expected: "github.com".into(),
                description: "Github directory",
            },
            PathTest {
                input: "/Users/tcrypt/code/github.com/go-bip39".into(),
                expected: "github.com/go-bip39".into(),
                description: "Regular project directory",
            },
            PathTest {
                input: "/Users/tcrypt/code/github.com/tyler-smith/promptpath".into(),
                expected: "promptpath".into(),
                description: "Mapped project directory",
            },
            PathTest {
                input: "/usr/bin".into(),
                expected: "/usr/bin".into(),
                description: "System directory",
            },
            PathTest {
                input: "/Users/tcrypt/code/github.com/tyler-smith/promptpath/src".into(),
                expected: "promptpath/src".into(),
                description: "Subdirectory of mapped project",
            },
            PathTest {
                input: "/Users/other_user".into(),
                expected: "/Users/other_user".into(),
                description: "Other user's home directory",
            },
        ];

        let ctx = setup_test_context();
        for test in test_cases {
            // Test path
            let path = PathBuf::from(&test.input);
            let result = get_nickname(&ctx, path.clone());
            assert_eq!(
                result, test.expected,
                "Failed test '{}': expected '{}', got '{}'",
                test.description, test.expected, result
            );

            // Test path with a trailing slash
            if path != PathBuf::from("/") {
                let path = path.join("");
                let result = get_nickname(&ctx, path.clone());
                assert_eq!(
                    result, test.expected,
                    "Failed test '{}': expected '{}', got '{}'",
                    test.description, test.expected, result
                );
            }
        }
    }
}
