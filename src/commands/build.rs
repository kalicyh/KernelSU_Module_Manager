use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::process::Command;
use regex::Regex;

pub fn execute() {
    println!("{} {}", "🔨", "构建模块...".cyan());

    // 检查是否存在 module.prop 文件
    let module_prop_path = Path::new("module.prop");
    if !module_prop_path.exists() {
        println!("{} 未找到 module.prop 文件，请确保在模块目录中运行此命令", "❌");
        return;
    }

    // 读取并解析 module.prop
    let module_prop_content = fs::read_to_string(module_prop_path)
        .expect("无法读取 module.prop 文件");

    let mut module_info = HashMap::new();
    for line in module_prop_content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            module_info.insert(key.to_string(), value.to_string());
        }
    }

    // 获取必要的信息
    let id = module_info.get("id").unwrap_or(&"unknown".to_string()).clone();
    let version = module_info.get("version").unwrap_or(&"0.1.0".to_string()).clone();
    let version_code = module_info.get("versionCode").unwrap_or(&"1".to_string()).clone();
    let update_json_url = module_info.get("updateJson").unwrap_or(&"https://github.com/unknown/repo/releases/latest/download/update.json".to_string()).clone();

    // 从 updateJson URL 解析 GitHub 用户名和仓库名
    let github_regex = Regex::new(r"github\.com[\/:]([^\/]+)\/([^\/]+)").unwrap();
    let (username, repo) = if let Some(captures) = github_regex.captures(&update_json_url) {
        let username = captures.get(1).map_or("unknown", |m| m.as_str());
        let repo = captures.get(2).map_or("repo", |m| m.as_str()).trim_end_matches(".git");
        (username.to_string(), repo.to_string())
    } else {
        ("unknown".to_string(), "repo".to_string())
    };

    // 获取 git 短提交哈希
    let short_commit = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        })
        .unwrap_or_else(|| "unknown".to_string());

    // 创建 .ksmm 目录
    let ksmm_dir = Path::new(".ksmm");
    fs::create_dir_all(ksmm_dir).expect("无法创建 .ksmm 目录");

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
    fs::write(&update_json_path, update_json).expect("无法写入 update.json");

    println!("{} 创建 .ksmm/update.json", "[+]".green());
    println!("{} 模块构建完成!", "✅");
}