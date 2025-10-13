use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;
use regex::Regex;
use chrono::{Datelike, Timelike, Utc};
use std::io;
use zip::write::FileOptions;

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

fn generate_update_json(module_info: &HashMap<String, String>, short_commit: &str, release_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

    let update_json_path = release_dir.join("update.json");
    fs::write(&update_json_path, update_json)?;

    Ok(())
}

fn read_build_config(file_path: &Path) -> (Vec<String>, Vec<String>) {
    let mut ignore_patterns = Vec::new();
    let mut include_patterns = Vec::new();

    if !file_path.exists() {
        return (ignore_patterns, include_patterns);
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if line.starts_with('!') {
                    // 强制包括模式
                    include_patterns.push(line[1..].to_string());
                } else {
                    // 忽略模式
                    ignore_patterns.push(line.to_string());
                }
            }
        }
        Err(_) => {}
    }

    (ignore_patterns, include_patterns)
}

fn matches_pattern(file_path: &str, pattern: &str) -> bool {
    // 简单模式匹配实现
    // 支持 * 通配符和目录匹配（以 / 结尾）
    if pattern.ends_with('/') {
        // 目录匹配
        return file_path.starts_with(pattern) || file_path.contains(&format!("/{}", pattern.trim_end_matches('/')));
    } else if pattern.contains('*') {
        // 通配符匹配
        let regex_pattern = pattern.replace('.', r"\.").replace('*', ".*");
        if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
            return regex.is_match(file_path);
        }
    } else {
        // 精确匹配
        return file_path == pattern || file_path.ends_with(&format!("/{}", pattern));
    }
    false
}

fn copy_files_to_build(build_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 确保构建目录存在
    fs::create_dir_all(build_dir)?;

    // 读取 .gitignore
    let gitignore_patterns = read_ignore_file(Path::new(".gitignore"));

    // 读取 .ksmm/build.conf
    let (ksmm_ignore_patterns, include_patterns) = read_build_config(Path::new(".ksmm/build.conf"));

    // 合并忽略模式：.ksmm/build.conf 优先级更高
    let mut all_ignore_patterns = gitignore_patterns;
    all_ignore_patterns.extend(ksmm_ignore_patterns);

    // 收集所有要处理的文件和目录
    let mut operations = Vec::new();
    collect_operations(Path::new("."), build_dir, &all_ignore_patterns, &include_patterns, &mut operations)?;

    // 排序操作：先包括，再忽略；先目录，再文件
    operations.sort_by(|a, b| {
        // 首先按类型排序：包括 > 忽略
        let a_is_include = matches!(a.operation_type, OperationType::Include(_));
        let b_is_include = matches!(b.operation_type, OperationType::Include(_));

        if a_is_include != b_is_include {
            return b_is_include.cmp(&a_is_include); // 包括优先
        }

        // 然后按操作类型排序：创建目录 > 复制文件 > 忽略
        let a_priority = match &a.operation_type {
            OperationType::CreateDir => 0,
            OperationType::CopyFile => 1,
            OperationType::Include(_) => 2,
            OperationType::Ignore(_) => 3,
        };
        let b_priority = match &b.operation_type {
            OperationType::CreateDir => 0,
            OperationType::CopyFile => 1,
            OperationType::Include(_) => 2,
            OperationType::Ignore(_) => 3,
        };

        a_priority.cmp(&b_priority)
    });

    // 执行操作并输出日志
    for op in operations {
        match op.operation_type {
            OperationType::CreateDir => {
                fs::create_dir_all(&op.dst)?;
                println!("{} 创建目录: {}", "[DEBUG]".cyan(), op.dst.display());
            }
            OperationType::CopyFile => {
                fs::copy(&op.src, &op.dst)?;
                println!("{} 复制文件: {} -> {}", "[DEBUG]".green(), op.src.display(), op.dst.display());
            }
            OperationType::Include(pattern) => {
                println!("{} 文件 '{}' 匹配包括模式 '{}', 包括", "[DEBUG]".yellow(), op.src.display(), pattern);
            }
            OperationType::Ignore(pattern) => {
                println!("{} 文件 '{}' 匹配忽略模式 '{}', 忽略", "[DEBUG]".red(), op.src.display(), pattern);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum OperationType {
    CreateDir,
    CopyFile,
    Include(String),
    Ignore(String),
}

#[derive(Debug, Clone)]
struct FileOperation {
    src: PathBuf,
    dst: PathBuf,
    operation_type: OperationType,
}

fn collect_operations(
    src: &Path,
    dst: &Path,
    ignore_patterns: &[String],
    include_patterns: &[String],
    operations: &mut Vec<FileOperation>,
) -> io::Result<()> {
    if !src.exists() {
        return Ok(());
    }

    // 检查是否是强制包括的文件
    if let Some(include_pattern) = is_force_include(src, include_patterns) {
        operations.push(FileOperation {
            src: src.to_path_buf(),
            dst: dst.to_path_buf(),
            operation_type: OperationType::Include(include_pattern),
        });
    }

    // 检查是否应该忽略此文件/目录
    if let Some(ignore_pattern) = should_ignore_file(src, ignore_patterns, include_patterns) {
        operations.push(FileOperation {
            src: src.to_path_buf(),
            dst: dst.to_path_buf(),
            operation_type: OperationType::Ignore(ignore_pattern),
        });
        return Ok(());
    }

    if src.is_dir() {
        // 收集创建目录的操作
        operations.push(FileOperation {
            src: src.to_path_buf(),
            dst: dst.to_path_buf(),
            operation_type: OperationType::CreateDir,
        });

        // 递归处理目录内容
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            // 跳过 .ksmm 目录本身
            if file_name == ".ksmm" {
                continue;
            }

            collect_operations(&src_path, &dst_path, ignore_patterns, include_patterns, operations)?;
        }
    } else {
        // 收集复制文件的操作
        operations.push(FileOperation {
            src: src.to_path_buf(),
            dst: dst.to_path_buf(),
            operation_type: OperationType::CopyFile,
        });
    }

    Ok(())
}

fn should_ignore_file(file_path: &Path, ignore_patterns: &[String], include_patterns: &[String]) -> Option<String> {
    let file_str = file_path.to_string_lossy();

    // 首先检查是否在包括列表中（最高优先级）
    if is_force_include(file_path, include_patterns).is_some() {
        return None;
    }

    // 然后检查是否在忽略列表中
    for pattern in ignore_patterns {
        if matches_pattern(&file_str, pattern) {
            return Some(pattern.clone());
        }
    }

    None
}

fn is_force_include(file_path: &Path, include_patterns: &[String]) -> Option<String> {
    let file_str = file_path.to_string_lossy();

    for pattern in include_patterns {
        if matches_pattern(&file_str, pattern) {
            return Some(pattern.clone());
        }
    }

    None
}

fn read_ignore_file(file_path: &Path) -> Vec<String> {
    if !file_path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(file_path) {
        Ok(content) => content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn package_build_to_zip(build_dir: &Path, module_info: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let id = module_info.get("id").unwrap_or(&"unknown".to_string()).clone();
    let _version = module_info.get("version").unwrap_or(&"0.1.0".to_string()).clone();
    let version_code = module_info.get("versionCode").unwrap_or(&"1".to_string()).clone();

    let release_dir = Path::new(".ksmm/release");

    // 生成ZIP文件名
    let zip_filename = format!("{}-{}.zip", id, version_code);
    let zip_path = release_dir.join(&zip_filename);

    // 创建ZIP文件
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(zip_file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 递归添加文件到ZIP
    add_dir_to_zip(&mut zip, build_dir, build_dir, options)?;

    zip.finish()?;
    println!("{} 创建 .ksmm/release/{}", "[+]".green(), zip_filename);

    Ok(())
}

fn add_dir_to_zip<W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    base_path: &Path,
    current_path: &Path,
    options: FileOptions,
) -> zip::result::ZipResult<()> {
    if current_path.is_dir() {
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.strip_prefix(base_path).unwrap().to_string_lossy();

            if path.is_dir() {
                zip.add_directory(name, options)?;
                add_dir_to_zip(zip, base_path, &path, options)?;
            } else {
                zip.start_file(name, options)?;
                let mut f = fs::File::open(&path)?;
                std::io::copy(&mut f, zip)?;
            }
        }
    }
    Ok(())
}

fn clear_build_and_release_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let build_dir = Path::new(".ksmm/build");
    let release_dir = Path::new(".ksmm/release");

    // 清空 build 目录
    if build_dir.exists() {
        fs::remove_dir_all(build_dir)?;
        println!("{} 清空 build 目录", "[+]".green());
    }

    // 清空 release 目录
    if release_dir.exists() {
        fs::remove_dir_all(release_dir)?;
        println!("{} 清空 release 目录", "[+]".green());
    }

    Ok(())
}

fn check_and_sign_release(module_info: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} 开始检查签名", "🔍");
    let key_dir = Path::new(".ksmm/key");

    // 检查是否有.pem文件
    let has_pem_files = if key_dir.exists() {
        fs::read_dir(key_dir)?
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "pem")
                    .unwrap_or(false)
            })
    } else {
        false
    };

    if has_pem_files {
        println!("{} 检测到PEM密钥文件", "🔑");
    } else {
        println!("{} 未检测到PEM密钥文件，跳过签名", "ℹ️");
        return Ok(());
    }

    // 获取模块信息用于签名
    let id = module_info.get("id").unwrap_or(&"unknown".to_string()).clone();
    let version_code = module_info.get("versionCode").unwrap_or(&"1".to_string()).clone();
    let zip_filename = format!("{}-{}.zip", id, version_code);

    let release_dir = Path::new(".ksmm/release");
    let zip_path = release_dir.join(&zip_filename);

    if !zip_path.exists() {
        return Err("ZIP文件不存在，无法签名".into());
    }

    // 构建签名命令
    let zip_path_str = zip_path.to_string_lossy().to_string();
    let signed_filename = format!("{}_signed.zip", zip_filename.trim_end_matches(".zip"));
    let signed_path = release_dir.join(&signed_filename);

    // 调用sign命令 (从系统调用)
    let ksmm_path = std::env::current_exe()?.parent().unwrap().join("ksmm");
    let sign_output = Command::new(ksmm_path)
        .args(&["sign", &zip_path_str])
        .output()?;


    if sign_output.status.success() {
        println!("{} 签名成功", "✅");
        // 移动签名后的文件到release目录
        let signed_source_name = format!("{}_signed.zip", zip_path_str.trim_end_matches(".zip"));
        let signed_source = Path::new(&signed_source_name);
        if signed_source.exists() {
            fs::rename(&signed_source, &signed_path)?;
            println!("{} 创建 .ksmm/release/{}", "[+]".green(), signed_filename);
        } else {
            println!("{} 签名完成，但未找到签名文件: {}", "⚠️", signed_source_name);
        }
    } else {
        let stderr = String::from_utf8_lossy(&sign_output.stderr);
        return Err(format!("签名失败: {}", stderr).into());
    }

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

    // 前先清空build目录和release目录
    if let Err(e) = clear_build_and_release_dirs() {
        println!("{} 清空目录失败: {}", "❌", e);
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

    // 创建 release 目录
    let release_dir = Path::new(".ksmm/release");
    if let Err(e) = fs::create_dir_all(release_dir) {
        println!("{} 创建 release 目录失败: {}", "❌", e);
        return;
    }

    // 生成 update.json
    if let Err(e) = generate_update_json(&module_info, &short_commit, &release_dir) {
        println!("{} 生成 update.json 失败: {}", "❌", e);
        return;
    }

    // 复制文件到构建目录
    let build_dir = Path::new(".ksmm/build");
    if let Err(e) = copy_files_to_build(&build_dir) {
        println!("{} 复制文件到构建目录失败: {}", "❌", e);
        return;
    }

    println!("{} 创建 .ksmm/release/update.json", "[+]".green());
    println!("{} 模块构建完成!", "✅");

    // 打包构建产物为ZIP
    if let Err(e) = package_build_to_zip(&build_dir, &module_info) {
        println!("{} 打包ZIP失败: {}", "❌", e);
        return;
    }

    // 检查并签名
    if let Err(e) = check_and_sign_release(&module_info) {
        println!("{} 签名过程失败: {}", "❌", e);
        return;
    }
}