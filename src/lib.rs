use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("ocean: error: {}", &format!($($arg)*));
    };
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("config parse error ({0})")]
    ConfigParse(#[from] toml::de::Error),

    #[error("IO error ({0})")]
    Io(#[from] io::Error),

    #[error("error ({0})")]
    Other(String),
}

#[derive(Deserialize, Serialize)]
struct Config {
    package: Package,
}

#[derive(Deserialize, Serialize)]
struct Package {
    name: String,
}

fn create_default_project_in_directory(mut path: PathBuf, name: String) -> Result<(), Error> {
    path.push("Ocean.toml");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|err| {
            error!("Unable to create the `Ocean.toml` file.");
            err
        })?;

    let config = Config {
        package: Package { name },
    };
    writeln!(&mut file, "{}", toml::to_string_pretty(&config).unwrap()).map_err(|err| {
        error!("Unable to write to the `Ocean.toml` file.");
        err
    })?;

    path.pop();
    path.push("src/headers");
    fs::create_dir_all(&path)?;

    path.pop();
    path.push("main.c");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    writeln!(&mut file, "int main(void) {{\n    return 0;\n}}")?;

    Ok(())
}

pub fn new(project_name: String) -> Result<(), Error> {
    let mut path = PathBuf::from("./");
    path.push(&project_name);
    fs::create_dir(&path).map_err(|err| {
        error!("Could not create a new directory `{}`.", path.display());
        err
    })?;

    create_default_project_in_directory(path, project_name)
}

pub fn init() -> Result<(), Error> {
    let project_name = env::current_dir()
        .map_err(|err| {
            error!("Couldn't access the current directory.");
            err
        })?
        .file_name()
        .ok_or_else(|| Error::Other("current directory has no name".into()))?
        .to_str()
        .ok_or_else(|| Error::Other("current dir name isn't valid utf-8".into()))?
        .to_owned();

    create_default_project_in_directory(PathBuf::from("./"), project_name)
}

fn dir_contains_config_file<P: AsRef<Path>>(path: P) -> Result<bool, io::Error> {
    for item in fs::read_dir(path)? {
        let item = item?;
        if item.file_type()?.is_file() || item.file_name() == "Ocean.toml" {
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_project_root() -> Result<Option<PathBuf>, io::Error> {
    let mut current_path = env::current_dir()?;

    loop {
        if dir_contains_config_file(&current_path)? {
            break Ok(Some(current_path));
        }
        if !current_path.pop() {
            break Ok(None);
        }
    }
}

fn get_project_details() -> Result<(Config, PathBuf), Error> {
    let root = get_project_root()?;

    match root {
        Some(root) => {
            let contents = fs::read_to_string(&root)?;
            Ok((toml::from_str(&contents)?, root))
        }
        None => Err(Error::Other("not inside a project".into())),
    }
}

pub fn run(args: Vec<OsString>, verbose: bool) -> Result<(), Error> {
    let (config, root) = get_project_details()?;

    todo!()
}

pub fn build(verbose: bool) -> Result<(), Error> {
    todo!()
}

pub fn clean() -> Result<(), Error> {
    todo!()
}
