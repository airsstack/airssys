use crate::error::Result;
use clap::{Args, ValueEnum};
use clap_complete::{generate, Shell};

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    shell: ShellType,
}

#[derive(ValueEnum, Clone, Copy)]
enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl From<ShellType> for Shell {
    fn from(shell: ShellType) -> Self {
        match shell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
            ShellType::Elvish => Shell::Elvish,
        }
    }
}

pub async fn execute(args: CompletionsArgs) -> Result<()> {
    let mut cmd = crate::Cli::command();
    let shell: Shell = args.shell.into();

    generate(shell, &mut cmd, "airssys-wasm", &mut std::io::stdout());

    Ok(())
}
