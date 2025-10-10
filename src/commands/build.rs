use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::process::Command;
use regex::Regex;
use chrono::{Datelike, Timelike, Utc};

fn refresh_version_code(module_prop_path: &Path) -> Result<i32, Box<dyn std::error::Error>> {
    // 读取并解析 module.prop，保持原始顺序
    let module_prop_content = fs::read_to_string(module_prop_path)?;

    let mut module_info = HashMap::new();

    // 刷新 versionCode
    let now = Utc::now();
    let new_version_code = (now.year() * 1000000 + now.month() as i32 * 10000 + now.day() as i32 * 100 + now.hour() as i32) as i32;

    let mut new_module_prop_content = String::new();
    let mut version_code_updated = false;

    for line in module_prop_content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.to_string();
            let value = value.to_string();

            if key == "versionCode" && !version_code_updated {
                new_module_prop_content.push_str(&format!("versionCode={}\n", new_version_code));
                module_info.insert(key, new_version_code.to_string());
                version_code_updated = true;
            } else {
                new_module_prop_content.push_str(&format!("{}={}\n", key, value));
                module_info.insert(key, value);
            }
        } else {
            new_module_prop_content.push_str(&format!("{}\n", line));
        }
    }

    // 如果没有找到 versionCode 字段，添加一个
    if !version_code_updated {
        new_module_prop_content.push_str(&format!("versionCode={}\n", new_version_code));
        module_info.insert("versionCode".to_string(), new_version_code.to_string());
    }

    fs::write(module_prop_path, new_module_prop_content)?;
    println!("{} 刷新 versionCode: {}", "[+]".green(), new_version_code);

    Ok(new_version_code)
}

fn parse_github_info(update_json_url: &str) -> (String, String) {
    // 从 updateJson URL 解析 GitHub 用户名和仓库名
    let github_regex = Regex::new(r"github\.com[\/:]([^\/]+)\/([^\/]+)").unwrap();
    if let Some(captures) = github_regex.captures(update_json_url) {
        let username = captures.get(1).map_or("unknown", |m| m.as_str());
        let repo = captures.get(2).map_or("repo", |m| m.as_str()).trim_end_matches(".git");
        (username.to_string(), repo.to_string())
    } else {
        ("unknown".to_string(), "repo".to_string())
    }
}

fn get_git_commit_hash() -> String {
    Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn generate_update_json(module_info: &HashMap<String, String>, short_commit: &str, ksmm_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let id = module_info.get("id").map(|s| s.as_str()).unwrap_or("unknown");
    let version = module_info.get("version").map(|s| s.as_str()).unwrap_or("0.1.0");
    let version_code = module_info.get("versionCode").map(|s| s.as_str()).unwrap_or("1");
    let update_json_url = module_info.get("updateJson").map(|s| s.as_str()).unwrap_or("https://github.com/unknown/repo/releases/latest/download/update.json");

    let (username, repo) = parse_github_info(update_json_url);

    // 生成 update.json
    let update_json = format!(
        r#"{{
  "changelog": "https://raw.githubusercontent.com/{}/{}/main/{}/CHANGELOG.md",
  "version": "v{}-{}",
  "versionCode": {},
  "zipUrl": "https://github.com/{}/{}/releases/latest/download/{}-{}.zip"
}}"#,
        username, repo, id, version, short_commit, version_code, username, repo, id, version_code
    );

    let update_json_path = ksmm_dir.join("update.json");
    fs::write(&update_json_path, update_json)?;

    Ok(())
}

pub fn execute() {
    println!("{} {}", "🔨", "构建模块...".cyan());

    // 检查是否存在 module.prop 文件
    let module_prop_path = Path::new("module.prop");
    if !module_prop_path.exists() {
        println!("{} 未找到 module.prop 文件，请确保在模块目录中运行此命令", "❌");
        return;
    }

    // 刷新 versionCode
    if let Err(e) = refresh_version_code(&module_prop_path) {
        println!("{} 刷新 versionCode 失败: {}", "❌", e);
        return;
    }

    // 重新读取更新后的 module.prop
    let module_prop_content = match fs::read_to_string(&module_prop_path) {
        Ok(content) => content,
        Err(e) => {
            println!("{} 重新读取 module.prop 失败: {}", "❌", e);
            return;
        }
    };

    let mut module_info = HashMap::new();
    for line in module_prop_content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            module_info.insert(key.to_string(), value.to_string());
        }
    }

    // 获取 git 短提交哈希
    let short_commit = get_git_commit_hash();

    // 创建 .ksmm 目录
    let ksmm_dir = Path::new(".ksmm");
    if let Err(e) = fs::create_dir_all(ksmm_dir) {
        println!("{} 创建 .ksmm 目录失败: {}", "❌", e);
        return;
    }

    // 生成 update.json
    if let Err(e) = generate_update_json(&module_info, &short_commit, &ksmm_dir) {
        println!("{} 生成 update.json 失败: {}", "❌", e);
        return;
    }

    println!("{} 创建 .ksmm/update.json", "[+]".green());
    println!("{} 模块构建完成!", "✅");
}