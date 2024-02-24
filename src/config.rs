use std::collections::HashMap;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use super::utils::{get_from_env, get_from_env_or_exit};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub link: LinkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            link: LinkConfig::default(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LinkConfig {
    pub source: String,
    pub destinations: Vec<String>,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            source: "".to_string(),
            destinations: vec![],
        }
    }
}

impl LinkConfig {
    pub fn expand_destinations(&self) -> Vec<PathBuf> {
        let mut destinations = Vec::new();
        for destination in self.destinations.clone() {
            destinations.push(PathBuf::from(if destination.starts_with("$") {
                get_from_env_or_exit(&destination[1..])
            } else {
                destination
            }));
        }
        destinations
    }

    pub fn expand_source(&self) -> PathBuf {
        PathBuf::from(if self.source.starts_with("$") {
            get_from_env_or_exit(&self.source[1..])
        } else {
            self.source.clone()
        })
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LinkDestinationConfig {
    pub systems: HashMap<String, System>,
}

impl Default for LinkDestinationConfig {
    fn default() -> Self {
        Self {
            systems: HashMap::new(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct System {
    pub destination: Option<String>,
    pub destinations: Option<Vec<String>>,
    pub dumper: String,
    pub extension: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub extra_path: Option<String>,
}

impl Default for System {
    fn default() -> Self {
        Self {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        }
    }
}

impl LinkDestinationConfig {
    pub fn get_system_names(&self) -> Vec<String> {
        Vec::from_iter(self.systems.keys().map(|k| k.to_string()))
    }
}

// TODO: Before this logic was moved here, the code had access to both `system` and the config,
// making the `unwrap_or` part possible. I want to figure out a better way to handle populating
// instances of the struct so that 1) `system` doesn't need to be passed in as an argument and 2) I
// don't have to resort to requiring `destination` and `extension` in each config entry.
impl System {
    pub fn get_destinations(&self, system: &String) -> Vec<String> {
        let destinations = if self.destinations.is_some() {
            self.destinations.clone().unwrap()
        } else {
            vec![self.destination.clone().unwrap_or(system.to_string())]
        };

        destinations
    }

    pub fn get_extensions(&self, system: &String) -> Vec<String> {
        let extensions = if self.extensions.is_some() {
            self.extensions.clone().unwrap()
        } else {
            vec![self.extension.clone().unwrap_or(system.to_string())]
        };

        extensions
    }
}

pub fn load_config() -> Result<Config, String> {
    let config: Config = match get_from_env("RETRO_CONFIG") {
        Ok(path) => confy::load_path(PathBuf::from(path)).unwrap(),
        Err(_) => match confy::load("retro", "retro") {
            Ok(config) => config,
            Err(e) => {
                return Err(e.to_string());
            }
        },
    };

    Ok(config)
}

pub fn load_config_recursively<T: serde::Serialize + serde::de::DeserializeOwned + Default>(
    root: &Path,
) -> Result<T, String> {
    let mut path: PathBuf = root.into();
    if path == PathBuf::from(".") {
        path = current_dir().unwrap();
    }
    let file = Path::new("retro.toml");

    loop {
        path.push(file);

        if path.is_file() {
            break Ok(confy::load_path(path).unwrap());
        }

        if !(path.pop() && path.pop()) {
            break Err("No retro.toml file found".to_string());
        }
    }
}

pub fn load_link_destination_config(
    config_file: Option<PathBuf>,
) -> Result<LinkDestinationConfig, String> {
    let config: LinkDestinationConfig =
        match confy::load_path(config_file.unwrap_or(PathBuf::from("retro.toml"))) {
            Ok(config) => config,
            Err(e) => {
                return Err(e.to_string());
            }
        };

    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::env::set_var;

    use super::*;

    #[test]
    fn expand_destinations_with_enviroment_variable() {
        set_var("TEST_EXPAND_DESTINATIONS_WITH_ENVIRONMENT_VARIABLE", "a");
        let config = LinkConfig {
            source: "$TEST_EXPAND_DESTINATIONS_WITH_ENVIRONMENT_VARIABLE".to_string(),
            destinations: vec![],
        };
        let source = config.expand_source();
        assert_eq!(source, PathBuf::from("a"));
    }

    #[test]
    fn expand_destinations_without_environment_variables() {
        let config = LinkConfig {
            source: "".to_string(),
            destinations: vec!["a".to_string(), "b".to_string()],
        };
        let destinations = config.expand_destinations();
        assert_eq!(destinations, vec![PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[test]
    fn expand_source_with_enviroment_variable() {
        set_var("TEST_EXPAND_SOURCE_WITH_ENVIROMENT_VARIABLE_1", "a");
        set_var("TEST_EXPAND_SOURCE_WITH_ENVIROMENT_VARIABLE_2", "b");
        let config = LinkConfig {
            source: "".to_string(),
            destinations: vec![
                "$TEST_EXPAND_SOURCE_WITH_ENVIROMENT_VARIABLE_1".to_string(),
                "$TEST_EXPAND_SOURCE_WITH_ENVIROMENT_VARIABLE_2".to_string(),
            ],
        };
        let destinations = config.expand_destinations();
        assert_eq!(destinations, vec![PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[test]
    fn expand_source_without_environment_variable() {
        let config = LinkConfig {
            source: "a".to_string(),
            destinations: vec![],
        };
        let source = config.expand_source();
        assert_eq!(source, PathBuf::from("a"));
    }

    #[test]
    fn link_destination_config_get_system_names() {
        let system1 = System {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        };
        let system2 = System {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        };
        let config = LinkDestinationConfig {
            systems: HashMap::from([
                ("system1".to_string(), system1),
                ("system2".to_string(), system2),
            ]),
        };
        let systems = config.get_system_names();
        assert_eq!(systems.len(), 2);
        assert!(systems.contains(&"system1".to_string()));
        assert!(systems.contains(&"system2".to_string()));
    }

    #[test]
    fn system_get_destinations_uses_destinations_first() {
        let destinations = &["b".to_string(), "c".to_string()];
        let system = System {
            destination: Some("a".to_string()),
            destinations: Some(destinations.to_vec()),
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        };
        assert_eq!(system.get_destinations(&"".to_string()), destinations);
    }

    #[test]
    fn system_get_destinations_uses_destination_second() {
        let system = System {
            destination: Some("a".to_string()),
            destinations: None,
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        };
        assert_eq!(system.get_destinations(&"".to_string()), &["a".to_string()]);
    }

    #[test]
    fn system_get_destinations_uses_system_last() {
        let system = System {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        };
        assert_eq!(system.get_destinations(&"abc".to_string()), &["abc"]);
    }

    #[test]
    fn system_get_extensions_uses_extensions_first() {
        let extensions = &["b".to_string(), "c".to_string()];
        let system = System {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: Some("a".to_string()),
            extensions: Some(extensions.to_vec()),
            extra_path: None,
        };
        assert_eq!(system.get_extensions(&"".to_string()), extensions);
    }

    #[test]
    fn system_get_extensions_uses_extension_second() {
        let system = System {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: Some("a".to_string()),
            extensions: None,
            extra_path: None,
        };
        assert_eq!(system.get_extensions(&"".to_string()), &["a".to_string()]);
    }

    #[test]
    fn system_get_extensions_uses_system_last() {
        let system = System {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        };
        assert_eq!(system.get_extensions(&"abc".to_string()), &["abc"]);
    }
}
