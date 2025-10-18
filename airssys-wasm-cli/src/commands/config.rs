use crate::error::Result;
use crate::utils;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Reset configuration to defaults
    Reset,
}

pub async fn execute(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommands::Show => {
            let config = crate::cli_config::CliConfig::load()?;
            println!("{}", toml::to_string_pretty(&config).unwrap());
        }
        ConfigCommands::Set { key, value } => {
            // TODO: Implement config set logic
            utils::success(&format!("Set {key} = {value}"));
        }
        ConfigCommands::Get { key } => {
            // TODO: Implement config get logic
            utils::info(&format!("Value for {key}: [not implemented]"));
        }
        ConfigCommands::Reset => {
            if utils::confirm("Reset configuration to defaults?") {
                let config = crate::cli_config::CliConfig::default();
                config.save()?;
                utils::success("Configuration reset to defaults");
            }
        }
    }

    Ok(())
}
