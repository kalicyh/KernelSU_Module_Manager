use owo_colors::OwoColorize;

pub fn execute() {
    println!("{} {} {}", "ℹ️".blue(), "KernelSU Module Manager".green().bold(), format!("v{}", env!("CARGO_PKG_VERSION")).yellow());
}