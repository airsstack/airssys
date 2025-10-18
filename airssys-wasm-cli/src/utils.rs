/// Utility functions for CLI operations
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Print success message
pub fn success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print error message
#[allow(dead_code)] // Will be used in error handling when commands are fully implemented
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print warning message
pub fn warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

/// Print info message
pub fn info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Create a progress bar with spinner
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a progress bar for downloads
#[allow(dead_code)] // Will be used for download progress in install/update commands
pub fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

/// Format bytes to human-readable size
#[allow(dead_code)] // Will be used for displaying component sizes and download progress
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Confirm action with user
pub fn confirm(message: &str) -> bool {
    use dialoguer::Confirm;
    Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .unwrap_or(false)
}
