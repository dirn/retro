use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub systems: HashMap<String, System>,
}

#[derive(Debug, serde::Deserialize)]
pub struct System {
    pub destination: Option<String>,
    pub destinations: Option<Vec<String>>,
    pub dumper: String,
    pub extension: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub extra_path: Option<String>,
}

impl Config {
    pub fn get_system_names(&self) -> Vec<String> {
        Vec::from_iter(self.systems.keys().map(|k| k.to_string()))
    }
}

// TODO: Before this logic was moved here, the code had access to both `system` and the config,
// making the `unwrap_or` part possible. I want to figure out a better way to handle populating
// instances of the struct so that 1) `system` doesn't need to be passed in as an argument and 2) I
// don't have to resort to requiring `destination` and `extension` in each config entry.
impl System {
    pub fn get_destinations(&self, system: String) -> Vec<String> {
        let destinations = if self.destinations.is_some() {
            self.destinations.clone().unwrap()
        } else {
            vec![self.destination.clone().unwrap_or(system.to_uppercase())]
        };

        destinations
    }

    pub fn get_extensions(&self, system: String) -> Vec<String> {
        let extensions = if self.extensions.is_some() {
            self.extensions.clone().unwrap()
        } else {
            vec![self.extension.clone().unwrap_or(system)]
        };

        extensions
    }
}

pub fn load_config(config_file: Option<&Path>) -> Result<Config, String> {
    let data = match read_to_string(config_file.unwrap_or(Path::new("retro.toml"))) {
        Ok(contents) => contents,
        Err(_) => {
            return Err(format!("Could not find config file at {config_file:?}"));
        }
    };

    let config: Config = match toml::from_str(&data) {
        Ok(config) => config,
        Err(_) => {
            return Err(format!("Could not parse config file at {config_file:?}"));
        }
    };

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_get_system_names() {
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
        let config = Config {
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
        let destinations = vec!["b".to_string(), "c".to_string()];
        let system = System {
            destination: Some("a".to_string()),
            destinations: Some(destinations.clone()),
            dumper: "".to_string(),
            extension: None,
            extensions: None,
            extra_path: None,
        };
        assert_eq!(system.get_destinations("".to_string()), destinations);
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
        assert_eq!(
            system.get_destinations("".to_string()),
            vec!["a".to_string()]
        );
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
        assert_eq!(system.get_destinations("abc".to_string()), vec!["ABC"]);
    }

    #[test]
    fn system_get_extensions_uses_extensions_first() {
        let extensions = vec!["b".to_string(), "c".to_string()];
        let system = System {
            destination: None,
            destinations: None,
            dumper: "".to_string(),
            extension: Some("a".to_string()),
            extensions: Some(extensions.clone()),
            extra_path: None,
        };
        assert_eq!(system.get_extensions("".to_string()), extensions);
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
        assert_eq!(system.get_extensions("".to_string()), vec!["a".to_string()]);
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
        assert_eq!(system.get_extensions("abc".to_string()), vec!["abc"]);
    }
}
