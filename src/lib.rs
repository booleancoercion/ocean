mod config;
pub use config::{Config, ConfigHost};
use config::{Manifest, Package};

use cc::Build;
use thiserror::Error;

use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

const TARGET: &str = env!("TARGET");

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("ocean: error: {}", &format!($($arg)*));
    };
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("manifest parse error ({0})")]
    ManifestParse(#[from] toml::de::Error),

    #[error("IO error ({0})")]
    Io(#[from] io::Error),

    #[error("error ({0})")]
    Other(String),
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

    let manifest = Manifest {
        package: Package { name },
    };
    writeln!(&mut file, "{}", toml::to_string_pretty(&manifest).unwrap()).map_err(|err| {
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

fn get_source_files_in_dir(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = vec![];
    for item in dir.read_dir()? {
        let item = item?;
        if item.file_type()?.is_file() {
            files.push(item.path());
        }
    }

    Ok(files)
}

pub fn run(args: Vec<OsString>, verbose: bool, chost: ConfigHost) -> Result<(), Error> {
    let config = chost.get_config()?;
    let status = build_with_config(verbose, &config)?;
    if status.success() {
        println!(
            "ocean: info: compilation successful. executing program `{}`:\n",
            config.manifest.package.name
        );

        let mut path = config.root.join("artifacts");
        if cfg!(windows) {
            path.push(&format!("{}.exe", config.manifest.package.name));
        } else {
            path.push(&config.manifest.package.name);
        }
        let _status = Command::new(path).args(&args).status()?;
    }

    Ok(())
}

pub fn build(verbose: bool, chost: ConfigHost) -> Result<(), Error> {
    let config = chost.get_config()?;
    build_with_config(verbose, &config).map(|_| ())
}

fn build_with_config(_verbose: bool, config: &Config) -> Result<ExitStatus, Error> {
    let Config { manifest, root } = config;

    let art_dir = root.join("artifacts");
    if !art_dir.exists() {
        fs::create_dir(&art_dir)?;
    }

    let src = root.join("src");
    let headers = src.join("headers");

    let files = get_source_files_in_dir(&src)?;

    let tool = Build::new()
        .opt_level(1)
        .target(TARGET)
        .host(TARGET)
        .include(&headers)
        .shared_flag(false)
        .static_flag(false)
        .cargo_metadata(false)
        .get_compiler();

    let mut command = tool.to_command();
    if tool.is_like_msvc() {
        command.args(&["/Fe:", &format!("{}.exe", manifest.package.name)]);
    } else {
        command.args(&["-o", &manifest.package.name]);
    }
    command.args(&files);
    command.current_dir(art_dir);

    Ok(command.status()?)
}

pub fn clean(chost: ConfigHost) -> Result<(), Error> {
    let config = chost.get_config()?;
    let art = config.root.join("artifacts");

    if art.exists() {
        fs::remove_dir_all(&art)?;
    }

    Ok(())
}
