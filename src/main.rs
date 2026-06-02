use clap::Parser;
use markdown_project_manager::cli::Cli;
use markdown_project_manager::commands;
use markdown_project_manager::store::Paths;

fn main() {
    let cli = Cli::parse();
    let paths = Paths::resolve(cli.dir.as_deref());
    if let Err(e) = commands::run(cli.command, &paths) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
