mod config;
pub mod print;

pub use config::{Config, ConfigHost};
pub use print::{error, info, OnError};

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
        .on_err(|| error!("could not create the `Ocean.toml` file."))?;

    let manifest = Manifest {
        package: Package { name },
    };
    writeln!(&mut file, "{}", toml::to_string_pretty(&manifest).unwrap())
        .on_err(|| error!("could not write to the `Ocean.toml` file."))?;

    path.pop();
    path.push("src/headers");
    fs::create_dir_all(&path).on_err(|| error!("could not create source directories."))?;

    path.pop();
    path.push("main.c");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .on_err(|| error!("could not create file src/main.c"))?;
    writeln!(&mut file, "int main(void) {{\n    return 0;\n}}")
        .on_err(|| error!("could not write to src/main.c"))?;

    Ok(())
}

pub fn new(project_name: String) -> Result<(), Error> {
    let mut path = PathBuf::from("./");
    path.push(&project_name);
    fs::create_dir(&path)
        .on_err(|| error!("could not create a new directory `{}`.", path.display()))?;

    create_default_project_in_directory(path, project_name)
}

pub fn init() -> Result<(), Error> {
    let project_name = env::current_dir()
        .on_err(|| error!("could not access the current directory."))?
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
        info!("executing program `{}`\n", config.manifest.package.name);

        let mut path = config.root.join("artifacts");
        if cfg!(windows) {
            path.push(&format!("{}.exe", config.manifest.package.name));
        } else {
            path.push(&config.manifest.package.name);
        }
        let status = Command::new(path).args(&args).status()?;
        if !status.success() {
            error!("process didn't exit successfully");
            if let Some(code) = status.code() {
                info!("exit code: {}", code);
            }
        }

        Ok(())
    } else {
        error!("cannot execute program. aborting...");
        Err(Error::Other(
            "compilation failure; cannot execute program".into(),
        ))
    }
}

pub fn build(verbose: bool, chost: ConfigHost) -> Result<(), Error> {
    let config = chost.get_config()?;
    build_with_config(verbose, &config).map(|_| ())
}

fn build_with_config(_verbose: bool, config: &Config) -> Result<ExitStatus, Error> {
    let Config { manifest, root } = config;

    let art_dir = root.join("artifacts");
    if !art_dir.exists() {
        fs::create_dir(&art_dir).on_err(|| error!("could not create artifact directory."))?;
    }

    let src = root.join("src");
    let headers = src.join("headers");

    let files =
        get_source_files_in_dir(&src).on_err(|| error!("could not gather source file names"))?;

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

    let status = command.status().unwrap();
    if status.success() {
        info!("compilation successful.")
    } else {
        error!("compilation failed.")
    }

    Ok(status)
}

pub fn clean(chost: ConfigHost) -> Result<(), Error> {
    let config = chost.get_config()?;
    let art = config.root.join("artifacts");

    if art.exists() {
        fs::remove_dir_all(&art).on_err(|| error!("could not remove artifact directory."))?;
    }

    Ok(())
}
