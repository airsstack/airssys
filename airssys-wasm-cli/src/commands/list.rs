use crate::error::Result;
use crate::utils;
use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    /// Show detailed information
    #[arg(short, long)]
    detailed: bool,

    /// Filter by status (running, stopped, all)
    #[arg(short, long)]
    status: Option<String>,
}

pub async fn execute(_args: ListArgs) -> Result<()> {
    // TODO: Implement list command
    // - Query component registry
    // - Format and display component list

    utils::info("Installed components:");
    println!("\nNo components installed yet.");
    println!("Install a component with 'airssys-wasm install <source>'");

    Ok(())
}
