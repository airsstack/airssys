use crate::error::Result;
use crate::utils;
use clap::Args;

#[derive(Args)]
pub struct UninstallArgs {
    /// Component name to uninstall
    name: String,

    /// Path to keypair for authorization
    #[arg(short, long)]
    keypair: Option<String>,

    /// Force uninstall without confirmation
    #[arg(short, long)]
    force: bool,
}

pub async fn execute(args: UninstallArgs) -> Result<()> {
    if !args.force && !utils::confirm(&format!("Uninstall component '{}'?", args.name)) {
        utils::info("Uninstall cancelled");
        return Ok(());
    }

    let spinner = utils::create_spinner(&format!("Uninstalling component '{}'...", args.name));

    // TODO: Implement component uninstall logic
    // - Verify signature authorization
    // - Remove component files
    // - Update registry

    spinner.finish_with_message("✓ Component uninstalled successfully");
    utils::success(&format!("Uninstalled {}", args.name));

    Ok(())
}
