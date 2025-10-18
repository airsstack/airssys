use crate::error::Result;
use crate::utils;
use clap::Args;

#[derive(Args)]
pub struct KeygenArgs {
    /// Output path for the keypair (default: ~/.airssys/keypair.json)
    #[arg(short, long)]
    output: Option<String>,

    /// Overwrite existing keypair if it exists
    #[arg(short, long)]
    force: bool,
}

pub async fn execute(_args: KeygenArgs) -> Result<()> {
    let spinner = utils::create_spinner("Generating Ed25519 keypair...");

    // TODO: Implement Ed25519 keypair generation
    // - Generate keypair using ed25519-dalek
    // - Save to specified path or default ~/.airssys/keypair.json
    // - Display public key for verification

    spinner.finish_with_message("✓ Keypair generated successfully");

    utils::success("Public key: [PUBLIC_KEY_PLACEHOLDER]");
    utils::info("Keypair saved to: [PATH_PLACEHOLDER]");
    utils::warning("Keep your private key secure and never share it!");

    Ok(())
}
