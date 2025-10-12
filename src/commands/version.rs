use owo_colors::OwoColorize;
use std::process::Command;

pub fn execute() {
    let version = env!("CARGO_PKG_VERSION");

    // Try to get git SHA
    let git_sha = match Command::new("git")
        .args(&["rev-parse", "--short=7", "HEAD"])
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    };

    println!("{} {} {}", "ℹ️".blue(), "KernelSU Module Manager".green().bold(), format!("{} ({})", version, git_sha).yellow());
}