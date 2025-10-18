use clap::{Command, Parser, Subcommand};

mod cli_config;
mod commands;
mod error;
mod utils;

use commands::*;
use error::Result;

/// airssys-wasm - Component lifecycle management CLI for AirsSys WASM framework
#[derive(Parser)]
#[command(name = "airssys-wasm")]
#[command(version, about, long_about = None)]
#[command(author = "AirsStack Contributors")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (text, json, yaml)
    #[arg(short, long, global = true, default_value = "text")]
    output: String,
}

impl Cli {
    pub fn command() -> Command {
        <Self as clap::CommandFactory>::command()
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new Ed25519 keypair for component signing
    Keygen(keygen::KeygenArgs),

    /// Initialize a new WASM component project
    Init(init::InitArgs),

    /// Build a WASM component from source
    Build(build::BuildArgs),

    /// Sign a WASM component with your private key
    Sign(sign::SignArgs),

    /// Install a component from Git repository, local file, or remote URL
    Install(install::InstallArgs),

    /// Update an installed component to a newer version
    Update(update::UpdateArgs),

    /// Uninstall a component (requires signature authorization)
    Uninstall(uninstall::UninstallArgs),

    /// List all installed components
    List(list::ListArgs),

    /// Show detailed information about a component
    Info(info::InfoArgs),

    /// View or stream component logs
    Logs(logs::LogsArgs),

    /// Check health and status of components
    Status(status::StatusArgs),

    /// Verify component signature and integrity
    Verify(verify::VerifyArgs),

    /// Manage CLI configuration
    Config(config::ConfigArgs),

    /// Generate shell completions
    Completions(completions::CompletionsArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    // Execute the appropriate command
    match cli.command {
        Commands::Keygen(args) => keygen::execute(args).await,
        Commands::Init(args) => init::execute(args).await,
        Commands::Build(args) => build::execute(args).await,
        Commands::Sign(args) => sign::execute(args).await,
        Commands::Install(args) => install::execute(args).await,
        Commands::Update(args) => update::execute(args).await,
        Commands::Uninstall(args) => uninstall::execute(args).await,
        Commands::List(args) => list::execute(args).await,
        Commands::Info(args) => info::execute(args).await,
        Commands::Logs(args) => logs::execute(args).await,
        Commands::Status(args) => status::execute(args).await,
        Commands::Verify(args) => verify::execute(args).await,
        Commands::Config(args) => config::execute(args).await,
        Commands::Completions(args) => completions::execute(args).await,
    }
}
