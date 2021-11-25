use clap::Parser;
use std::ffi::OsString;

/// A universal C compiler shortcut for quick and dirty development, inspired by Cargo.
#[derive(Parser)]
#[clap(
    version = "0.1",
    author = "boolean_coercion <booleancoercion@gmail.com>"
)]
struct Opts {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Create a new project inside a new directory, with the specified name.
    New { project_name: String },
    /// Initialize a new project inside the current directory.
    Init,
    /// Build and execute the current project.
    Run {
        #[clap(short)]
        verbose: bool,
        #[clap(raw = true)]
        arguments: Vec<OsString>,
    },
    /// Build the current project.
    Build {
        #[clap(short)]
        verbose: bool,
    },
    /// Cleanup build artifacts.
    Clean,
}

fn main() {
    let opts = Opts::parse();

    let result = match opts.subcommand {
        SubCommand::New { project_name } => ocean::new(project_name),
        SubCommand::Init => ocean::init(),
        SubCommand::Run { arguments, verbose } => {
            let config = ocean::get_project_details().unwrap();
            ocean::run(arguments, verbose, &config)
        }
        SubCommand::Build { verbose } => {
            let config = ocean::get_project_details().unwrap();
            ocean::build(verbose, &config).map(|_| ())
        }
        SubCommand::Clean => ocean::clean(),
    };

    match result {
        Ok(()) => (),
        Err(why) => ocean::error!("{}", why),
    }
}
