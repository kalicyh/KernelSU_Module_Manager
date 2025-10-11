use owo_colors::OwoColorize;
use std::fs;
use std::process::Command;
use std::env;
use std::path::Path;
use clap::Subcommand;

// 嵌入 zakosign 二进制文件
static ZAKOSIGN_BINARY: &[u8] = include_bytes!("../bin/macos/arm64/zakosign");

#[derive(Subcommand)]
pub enum KeyCommands {
    /// 创建新的密钥对
    New {
        /// 密钥文件名
        name: String,
    },
}

fn get_zakosign_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // 创建临时目录存放 zakosign 二进制文件
    let temp_dir = env::temp_dir();
    let zakosign_path = temp_dir.join("zakosign");

    // 如果文件不存在，则写入嵌入的二进制文件
    if !zakosign_path.exists() {
        fs::write(&zakosign_path, ZAKOSIGN_BINARY)?;

        // 设置执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&zakosign_path, fs::Permissions::from_mode(0o755))?;
        }
    }

    Ok(zakosign_path)
}

pub fn execute_sign_file(file: String) {
    println!("{} {}", "📋", "对文件进行签名".cyan());

    // 检查文件是否存在
    let input_path = Path::new(&file);
    if !input_path.exists() {
        println!("{} 文件 '{}' 不存在", "❌", file);
        return;
    }

    // 扫描 .ksmm/key 目录中的密钥文件
    let ksmm_dir = Path::new(".ksmm");
    let key_dir = ksmm_dir.join("key");

    if !key_dir.exists() {
        println!("{} 未找到密钥目录，请先使用 'ksmm key new <name>' 创建密钥", "❌");
        println!("{} 或者手动将 ED25519 类型的 .pem 文件放置在 .ksmm/key/ 目录中", "💡".blue());
        return;
    }

    // 查找密钥文件
    let key_files = match fs::read_dir(&key_dir) {
        Ok(entries) => {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().map_or(false, |ext| ext == "pem"))
                .collect::<Vec<_>>()
        }
        Err(e) => {
            println!("{} 读取密钥目录失败: {}", "❌", e);
            return;
        }
    };

    if key_files.is_empty() {
        println!("{} 未找到任何 .pem 密钥文件，请先使用 'ksmm key new <name>' 创建密钥", "❌");
        return;
    }

    // 使用第一个找到的密钥文件
    let key_path = &key_files[0];
    println!("{} 使用密钥: {}", "🔑", key_path.display());

    // 获取 zakosign 路径
    let zakosign_path = match get_zakosign_path() {
        Ok(path) => path,
        Err(e) => {
            println!("{} 获取 zakosign 失败: {}", "❌", e);
            return;
        }
    };

    // 生成输出文件名
    let output_file = if file.ends_with(".zip") {
        file.replace(".zip", "_signed.zip")
    } else {
        format!("{}_signed", file)
    };

    // 执行签名命令
    let output = match Command::new(&zakosign_path)
        .args(&["sign", "--key", key_path.to_str().unwrap(), "--output", &output_file, "-f", &file])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            println!("{} 执行签名命令失败: {}", "❌", e);
            return;
        }
    };

    if output.status.success() {
        let signed_file = format!("{}_signed.zip", file.trim_end_matches(".zip"));
        println!("{} 文件签名成功", "✅");
        println!("{} 输入文件: {}", "📁", file);
        println!("{} 输出文件: {}", "📁", signed_file);
    } else {
        println!("{} 签名失败", "❌");
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            println!("错误信息: {}", stderr);
        }
    }
}

pub fn execute_key_command(key_command: KeyCommands) {
    match key_command {
        KeyCommands::New { name } => create_new_key(name),
    }
}

fn create_new_key(name: String) {
    println!("{} {}", "🔑", "创建新的签名密钥".cyan());

    // 自动添加 .pem 后缀（如果没有的话）
    let key_name = if name.ends_with(".pem") {
        name
    } else {
        format!("{}.pem", name)
    };

    // 创建 .ksmm/key 目录
    let ksmm_dir = Path::new(".ksmm");
    let key_dir = ksmm_dir.join("key");
    if let Err(e) = fs::create_dir_all(&key_dir) {
        println!("{} 创建密钥目录失败: {}", "❌", e);
        return;
    }

    let key_path = key_dir.join(&key_name);
    if key_path.exists() {
        println!("{} 密钥文件 '{}' 已存在", "⚠️".yellow(), key_path.display());
        return;
    }

    // 获取 zakosign 路径
    let zakosign_path = match get_zakosign_path() {
        Ok(path) => path,
        Err(e) => {
            println!("{} 获取 zakosign 失败: {}", "❌", e);
            return;
        }
    };

    // 执行密钥创建命令 - zakosign 会直接输出到指定文件
    let output = match Command::new(&zakosign_path)
        .args(&["key", "new", &key_path.to_string_lossy()])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            println!("{} 执行密钥创建命令失败: {}", "❌", e);
            return;
        }
    };

    if output.status.success() {
        println!("{} 密钥已创建: {}", "✅", key_path.display());
        println!("{} 私钥文件: {}", "🔒", key_path.display());
    } else {
        println!("{} 密钥创建失败", "❌");
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            println!("错误信息: {}", stderr);
        }
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            println!("标准输出: {}", stdout);
        }
    }
}
