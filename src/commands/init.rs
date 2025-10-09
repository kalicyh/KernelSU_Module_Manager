use dialoguer::{Input, Confirm};
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::{Datelike, Utc};
use regex::Regex;

fn get_git_info() -> (String, Option<String>) {
    // 首先尝试获取git远程仓库URL
    println!("{} 尝试获取git仓库信息...", "🔍".blue());
    let remote_output = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .output();

    let mut update_json = String::new();
    let mut username = None;

    if let Ok(output) = remote_output {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("{} 找到git远程仓库: {}", "✅".green(), url);
            
            // 解析GitHub URL
            // 支持的格式：
            // https://github.com/username/repo.git
            // git@github.com:username/repo.git
            // https://github.com/username/repo
            let github_regex = Regex::new(r"github\.com[\/:]([^\/]+)\/([^\/\.]+)").unwrap();
            
            if let Some(captures) = github_regex.captures(&url) {
                if let (Some(user), Some(repo)) = (captures.get(1), captures.get(2)) {
                    let user = user.as_str();
                    let repo = repo.as_str().trim_end_matches(".git");
                    println!("{} 解析出用户名: {}, 仓库: {}", "✅".green(), user, repo);
                    update_json = format!("https://github.com/{}/{}/releases/latest/download/update.json", user, repo);
                    username = Some(user.to_string());
                }
            } else {
                println!("{} 无法从URL解析GitHub信息", "⚠️".yellow());
            }
        } else {
            println!("{} git remote get-url origin 命令失败", "❌".red());
        }
    } else {
        println!("{} 无法执行git remote get-url origin 命令", "❌".red());
    }

    // 如果无法从远程URL获取，尝试获取git用户名
    if username.is_none() {
        println!("{} 尝试获取git用户名...", "🔍".blue());
        let user_output = Command::new("git")
            .args(&["config", "user.name"])
            .output();

        if let Ok(output) = user_output {
            if output.status.success() {
                let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !user.is_empty() {
                    println!("{} 找到git用户名: {}", "✅".green(), user);
                    update_json = format!("https://github.com/{}/ksmm/releases/latest/download/update.json", user);
                    username = Some(user);
                } else {
                    println!("{} git用户名为空", "⚠️".yellow());
                }
            } else {
                println!("{} git config user.name 命令失败", "❌".red());
            }
        } else {
            println!("{} 无法执行git config user.name 命令", "❌".red());
        }
    }

    // 如果都获取不到，使用默认的ksmm
    if update_json.is_empty() {
        println!("{} 使用默认用户名: ksmm", "ℹ️".blue());
        println!("{} 建议在module.prop文件中手动修改updateJson和作者信息", "💡".cyan());
        update_json = "https://github.com/ksmm/ksmm/releases/latest/download/update.json".to_string();
    }

    (update_json, username)
}

pub fn execute() {
    println!("{} {}", "🚀".green(), "初始化 KernelSU 模块...".cyan());

    // 输入创建地址
    let path: String = Input::new()
        .with_prompt("请输入创建地址 (默认当前目录: .)")
        .default(".".to_string())
        .interact_text()
        .unwrap();

    let base_path = Path::new(&path);

    // 确定项目名称
    let mut project_name = if path == "." {
        // 使用当前目录名称
        match std::env::current_dir() {
            Ok(current_dir) => {
                current_dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            }
            Err(_) => "unknown".to_string(),
        }
    } else {
        base_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    };

    // 验证项目名称/id格式
    let id_regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$").unwrap();
    if !id_regex.is_match(&project_name) {
        println!("{} 当前目录名称 '{}' 不符合模块ID格式要求。", "⚠️".yellow(), project_name);
        project_name = Input::new()
            .with_prompt("请输入项目名称 (必须以字母开头，只能包含字母、数字、点、下划线和连字符)")
            .interact_text()
            .unwrap();
        
        // 再次验证用户输入
        if !id_regex.is_match(&project_name) {
            println!("{} 模块ID格式无效。必须以字母开头，只能包含字母、数字、点、下划线和连字符。", "❌".red());
            return;
        }
    }

    // 创建 system 目录
    let system_path = base_path.join("system");
    fs::create_dir_all(&system_path).expect("无法创建 system 目录");

    // 使用项目名称作为id和name
    let id = project_name.clone();
    let name = project_name;

    // 获取git仓库信息
    let (update_json, git_username) = get_git_info();

    // 确定作者
    let author = if let Some(username) = git_username {
        println!("{} 使用git用户名作为作者: {}", "✅".green(), username);
        println!("{} 如果作者信息不符合要求，请在module.prop文件中手动修改", "ℹ️".blue());
        username
    } else {
        println!("{} 无法获取git用户信息，使用默认作者: ksmm", "ℹ️".blue());
        "ksmm".to_string()
    };

    // 默认版本信息
    let version = "0.1.0".to_string();
    let now = Utc::now();
    let version_code_int = (now.year() * 10000 + now.month() as i32 * 100 + now.day() as i32) as i32;

    // 自动生成描述
    let description = format!("一个用ksmm创建的{}模块", name);

    // 创建 module.prop
    let module_prop_content = format!(
        "id={}\nname={}\nversion={}\nversionCode={}\nauthor={}\ndescription={}\nupdateJson={}\n",
        id, name, version, version_code_int, author, description, update_json
    );
    fs::write(base_path.join("module.prop"), module_prop_content).expect("无法写入 module.prop");

    // 创建脚本文件
    fs::write(base_path.join("post-fs-data.sh"), "#!/system/bin/sh\n# 在文件系统挂载后执行\n").expect("无法写入 post-fs-data.sh");
    fs::write(base_path.join("service.sh"), "#!/system/bin/sh\n# 服务脚本\n").expect("无法写入 service.sh");
    fs::write(base_path.join("customize.sh"), "#!/system/bin/sh\n# 自定义脚本\n").expect("无法写入 customize.sh");

    // 询问是否需要执行按钮
    let need_action = Confirm::new()
        .with_prompt("是否需要执行按钮?")
        .default(true)
        .interact()
        .unwrap();

    if need_action {
        fs::write(base_path.join("action.sh"), "#!/system/bin/sh\n# 执行按钮脚本\n").expect("无法写入 action.sh");
    }

    // 询问是否需要 webui
    let need_webui = Confirm::new()
        .with_prompt("是否需要 WebUI?")
        .default(true)
        .interact()
        .unwrap();

    if need_webui {
        let webroot_path = base_path.join("webroot");
        fs::create_dir_all(&webroot_path).expect("无法创建 webroot 目录");

        let index_html = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>KernelSU Module WebUI</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 50px; }
        h1 { color: #333; }
    </style>
</head>
<body>
    <h1>欢迎使用 KernelSU 模块</h1>
    <p>这是一个简单的 WebUI 示例。</p>
</body>
</html>"#;
        fs::write(webroot_path.join("index.html"), index_html).expect("无法写入 index.html");
    }

    println!("{} {}", "✅".green(), "模块初始化完成!".cyan());
}