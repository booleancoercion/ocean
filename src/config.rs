use crate::Error;

use serde_derive::{Deserialize, Serialize};

use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    pub package: Package,
}

#[derive(Deserialize, Serialize)]
pub struct Package {
    pub name: String,
}

pub struct Config {
    pub manifest: Manifest,
    pub root: PathBuf,
}

pub struct ConfigHost {
    pub config: Result<Config, Error>,
}

impl ConfigHost {
    /// Potentially gets the config, and prints a relevant error message in case that's not possible.
    pub fn get_config(self) -> Result<Config, Error> {
        self.config
    }

    fn generate_config() -> Result<Config, Error> {
        let root = Self::get_project_root()?;

        match root {
            Some(root) => {
                let contents = fs::read_to_string(&root.join("Ocean.toml"))?;
                let config = Config {
                    manifest: toml::from_str(&contents)?,
                    root,
                };
                Ok(config)
            }
            None => Err(Error::Other("not inside a project".into())),
        }
    }

    fn get_project_root() -> Result<Option<PathBuf>, io::Error> {
        let mut current_path = env::current_dir()?;

        loop {
            if Self::dir_contains_manifest_file(&current_path)? {
                break Ok(Some(current_path));
            }
            if !current_path.pop() {
                break Ok(None);
            }
        }
    }

    fn dir_contains_manifest_file<P: AsRef<Path>>(path: P) -> Result<bool, io::Error> {
        for item in fs::read_dir(path)? {
            let item = item?;
            if item.file_type()?.is_file() || item.file_name() == "Ocean.toml" {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl Default for ConfigHost {
    fn default() -> Self {
        Self {
            config: Self::generate_config(),
        }
    }
}
