use clap::Parser;
use ocean::ConfigHost;
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

    let chost = ConfigHost::default();

    let result = match opts.subcommand {
        SubCommand::New { project_name } => ocean::new(project_name),
        SubCommand::Init => ocean::init(),
        SubCommand::Run { arguments, verbose } => ocean::run(arguments, verbose, chost),
        SubCommand::Build { verbose } => ocean::build(verbose, chost),
        SubCommand::Clean => ocean::clean(chost),
    };

    match result {
        Ok(()) => (),
        Err(why) => ocean::error!("{}", why),
    }
}
