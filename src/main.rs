use clap::{Parser, Subcommand, builder::Styles, CommandFactory};
use std::env;

mod commands;

#[derive(Parser)]
#[command(
    name = "ksmm",
    about = "KernelSU Module Manager",
    styles = get_styles()
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show version information
    #[arg(short = 'V', long)]
    version: bool,
}

fn get_styles() -> Styles {
    Styles::styled()
        .header(clap::builder::styling::AnsiColor::Green.on_default() | clap::builder::styling::Effects::BOLD)
        .usage(clap::builder::styling::AnsiColor::Cyan.on_default())
        .literal(clap::builder::styling::AnsiColor::Yellow.on_default())
        .placeholder(clap::builder::styling::AnsiColor::Magenta.on_default())
        .error(clap::builder::styling::AnsiColor::Red.on_default() | clap::builder::styling::Effects::BOLD)
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化模块
    Init,
    /// 构建模块
    Build,
    /// 签名文件
    Sign {
        /// 要签名的文件
        file: String,
    },
    /// 密钥管理
    Key {
        #[command(subcommand)]
        key_command: commands::sign::KeyCommands,
    },
    /// 显示版本信息
    Version,
}

fn main() {
    unsafe {
        env::set_var("FORCE_COLOR", "1");
    }

    let cli = Cli::parse();

    // Handle version flag
    if cli.version {
        commands::version::execute();
        return;
    }

    // Handle commands
    match cli.command {
        Some(Commands::Build) => commands::build::execute(),
        Some(Commands::Init) => commands::init::execute(),
        Some(Commands::Sign { file }) => commands::sign::execute_sign_file(file),
        Some(Commands::Key { key_command }) => commands::sign::execute_key_command(key_command),
        Some(Commands::Version) => commands::version::execute(),
        None => {
            // No command provided, show help
            let _ = Cli::command().print_help();
        }
    }
}
