use clap::Parser;
use markdown_project_manager::cli::{Cli, Command};
use markdown_project_manager::store::Paths;
use markdown_project_manager::{commands, install};

fn main() {
    let cli = Cli::parse();
    // `install` targets agent hosts, not a follow-ups log — handle it before
    // resolving Paths so it works from anywhere (incl. dirs with no log).
    let result = match cli.command {
        Command::Install { host } => install::run(host.as_deref()),
        cmd => commands::run(cmd, &Paths::resolve(cli.dir.as_deref())),
    };
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
