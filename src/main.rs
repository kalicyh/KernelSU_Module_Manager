use clap::{Parser, Subcommand, builder::Styles};
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
    command: Commands,
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
    /// 列出已安装模块
    List,
    /// 安装模块
    Install {
        /// 模块名称
        module: String,
    },
    /// 初始化模块
    Init,
    /// 移除模块
    Remove {
        /// 模块名称
        module: String,
    },
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

    match cli.command {
        Commands::Build => commands::build::execute(),
        Commands::Init => commands::init::execute(),
        Commands::Install { module } => commands::install::execute(module),
        Commands::List => commands::list::execute(),
        Commands::Remove { module } => commands::remove::execute(module),
        Commands::Sign { file } => commands::sign::execute_sign_file(file),
        Commands::Key { key_command } => commands::sign::execute_key_command(key_command),
        Commands::Version => commands::version::execute(),
    }
}
