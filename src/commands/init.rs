use dialoguer::{Input, Confirm};
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::{Datelike, Timelike, Utc};
use regex::Regex;

fn get_git_info() -> (String, Option<String>, Option<String>, Option<String>, Option<String>) {
    // 获取分支信息
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output();

    let branch = if let Ok(output) = branch_output {
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    // 获取远程仓库URL
    let remote_output = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .output();

    let mut remote_url = None;
    let mut update_json = String::new();
    let mut username = None;

    if let Ok(output) = remote_output {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            remote_url = Some(url.clone());

            // 解析GitHub URL
            let github_regex = Regex::new(r"github\.com[\/:]([^\/]+)\/([^\/\.]+)").unwrap();

            if let Some(captures) = github_regex.captures(&url) {
                if let (Some(user), Some(repo)) = (captures.get(1), captures.get(2)) {
                    let user = user.as_str();
                    let repo = repo.as_str().trim_end_matches(".git");
                    update_json = format!("https://github.com/{}/{}/releases/latest/download/update.json", user, repo);
                    username = Some(user.to_string());
                }
            }
        }
    }

    // 如果无法从远程URL获取，尝试获取git用户名
    if username.is_none() {
        let user_output = Command::new("git")
            .args(&["config", "user.name"])
            .output();

        if let Ok(output) = user_output {
            if output.status.success() {
                let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !user.is_empty() {
                    update_json = format!("https://github.com/{}/ksmm/releases/latest/download/update.json", user);
                    username = Some(user);
                }
            }
        }
    }

    // 获取工作目录状态
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output();

    let is_clean = if let Ok(output) = status_output {
        output.stdout.is_empty()
    } else {
        false
    };

    let status = if is_clean { "工作目录清洁" } else { "工作目录有变更" };

    // 如果都获取不到，使用默认的ksmm
    if update_json.is_empty() {
        update_json = "https://github.com/ksmm/ksmm/releases/latest/download/update.json".to_string();
    }

    (update_json, username, branch, remote_url, Some(status.to_string()))
}

fn create_system_directory(base_path: &Path) {
    let system_path = base_path.join("system");
    if system_path.exists() {
        println!("{}", format!("  [!] system 目录已存在，跳过创建").dimmed());
    } else {
        fs::create_dir_all(&system_path).expect("无法创建 system 目录");
        println!("{} 创建 system 目录", "[+]".green());

        // 创建 system/etc 目录
        let etc_path = system_path.join("etc");
        fs::create_dir_all(&etc_path).expect("无法创建 system/etc 目录");

        // 创建示例配置文件
        let example_conf_path = etc_path.join("example.conf");
        let example_conf_content = "# 这是一个示例配置文件\n# 将此文件放置在system目录中，它会被挂载到 /system/etc/example.conf\n";
        fs::write(&example_conf_path, example_conf_content).expect("无法写入 example.conf");
        println!("{} 创建 system/etc/example.conf", "[+]".green());
    }
}

fn create_module_prop(base_path: &Path, id: &str, name: &str, version: &str, version_code: i32, author: &str, description: &str, update_json: &str) {
    let module_prop_path = base_path.join("module.prop");
    if module_prop_path.exists() {
        println!("{}", format!("  [!] module.prop 文件已存在，跳过创建").dimmed());
    } else {
        let module_prop_content = format!(
            "id={}\nname={}\nversion={}\nversionCode={}\nauthor={}\ndescription={}\nupdateJson={}\n",
            id, name, version, version_code, author, description, update_json
        );
        fs::write(&module_prop_path, module_prop_content).expect("无法写入 module.prop");
        println!("{} 创建 module.prop", "[+]".green());
    }
}

fn create_script_files(base_path: &Path) {
    let customize_content = r#"#!/system/bin/sh
# KernelSU 模块自定义安装脚本

# 检查设备信息
ui_print "- 设备架构: $ARCH"
ui_print "- Android API: $API"
ui_print "- KernelSU 版本: $KSU_VER"

# 根据设备架构进行不同的处理
case $ARCH in
    arm64)
        ui_print "- 64位ARM设备"
        ;;
    arm)
        ui_print "- 32位ARM设备"
        ;;
    x64)
        ui_print "- x86_64设备"
        ;;
    x86)
        ui_print "- x86设备"
        ;;
esac

# 根据Android版本进行处理
# 示例shellcheck 自动修复 $API -> "$API"
if [ $API -lt 29 ]; then
    ui_print "- Android 10以下版本"
else
    ui_print "- Android 10及以上版本"
fi

# 设置权限（如果需要）
# set_perm_recursive $MODPATH/system/bin 0 0 0755 0755
# set_perm $MODPATH/system/etc/example.conf 0 0 0644

# 示例：删除系统文件（取消注释以使用）
# REMOVE="
# /system/app/SomeSystemApp
# /system/etc/some_config_file
# "

# 示例：替换系统目录（取消注释以使用）
# REPLACE="
# /system/app/SomeSystemApp
# "

ui_print "- 模块安装完成"
"#;

    let scripts = [
        ("post-fs-data.sh", "#!/system/bin/sh\n# 在文件系统挂载后执行\n"),
        ("service.sh", "#!/system/bin/sh\n# 服务脚本\n"),
        ("customize.sh", customize_content),
    ];

    for (filename, content) in &scripts {
        let file_path = base_path.join(filename);
        if file_path.exists() {
            println!("{}", format!("  [!] {} 文件已存在，跳过创建", filename).dimmed());
        } else {
            fs::write(&file_path, content).expect(&format!("无法写入 {}", filename));
            println!("{} 创建 {}", "[+]".green(), filename);
        }
    }
}

fn create_changelog(base_path: &Path) {
    let changelog_path = base_path.join("CHANGELOG.md");
    if changelog_path.exists() {
        println!("{}", format!("  [!] CHANGELOG.md 文件已存在，跳过创建").dimmed());
    } else {
        let changelog_content = "# 更新日志\n## v0.1.0\n";
        fs::write(&changelog_path, changelog_content).expect("无法写入 CHANGELOG.md");
        println!("{} 创建 CHANGELOG.md", "[+]".green());
    }
}

fn create_action_script(base_path: &Path) {
    let action_path = base_path.join("action.sh");
    if action_path.exists() {
        println!("{}", format!("  [!] action.sh 文件已存在，跳过创建").dimmed());
    } else {
        fs::write(&action_path, "#!/system/bin/sh\n# 执行按钮脚本\n").expect("无法写入 action.sh");
        println!("{} 创建 action.sh", "[+]".green());
    }
}

fn create_webui(base_path: &Path) {
    let webroot_path = base_path.join("webroot");
    if webroot_path.exists() {
        println!("{}", format!("  [!] webroot 目录已存在，跳过创建").dimmed());
    } else {
        fs::create_dir_all(&webroot_path).expect("无法创建 webroot 目录");
        println!("{} 创建 webroot 目录", "[+]".green());
    }

    let index_html_path = webroot_path.join("index.html");
    if index_html_path.exists() {
        println!("{}", format!("  [!] index.html 文件已存在，跳过创建").dimmed());
    } else {
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
        fs::write(&index_html_path, index_html).expect("无法写入 index.html");
        println!("{} 创建 index.html", "[+]".green());
    }
}

fn create_ksmm_config(base_path: &Path) {
    let ksmm_path = base_path.join(".ksmm");
    if ksmm_path.exists() {
        println!("{}", format!("  [!] .ksmm 目录已存在，跳过创建").dimmed());
    } else {
        fs::create_dir_all(&ksmm_path).expect("无法创建 .ksmm 目录");
        println!("{} 创建 .ksmm 目录", "[+]".green());
    }

    // 创建构建配置文件
    let build_conf_path = ksmm_path.join("build.conf");
    if build_conf_path.exists() {
        println!("{}", format!("  [!] .ksmm/build.conf 文件已存在，跳过创建").dimmed());
    } else {
        let build_conf_content = r#"# KernelSU 模块构建配置文件
# 控制哪些文件被复制到构建目录
#
# 语法说明:
#   - 普通行: 忽略这些文件/目录 (支持通配符)
#   - ! 开头的行: 强制包括这些文件/目录 (即使在忽略列表中)
#   - # 开头的行: 注释
#   - 空行: 被忽略
#
# 示例:
#   *.log      # 忽略所有 .log 文件
#   build/     # 忽略 build 目录
#   !system/   # 强制包括 system 目录

# 版本控制文件
.git/
.gitignore
.github/

# 构建产物
build/
target/
*.zip
*.tar.gz

# 临时文件
*.tmp
*.bak
*~

# 日志文件
*.log

# IDE 和编辑器文件
.vscode/
.idea/
*.swp
*.swo

# 操作系统文件
.DS_Store
Thumbs.db

# 文档文件
README.md
CHANGELOG.md

# 强制包括的核心模块文件
!module.prop
!system/
!webroot/

# 强制包括脚本文件
!*.sh
!action.sh

# 强制包括配置文件
!system.prop
!sepolicy.rule
"#;
        fs::write(&build_conf_path, build_conf_content).expect("无法写入 .ksmm/build.conf");
        println!("{} 创建 .ksmm/build.conf", "[+]".green());
    }
}

pub fn execute() {
    println!("{} {}", "🚀", "初始化 KernelSU 模块...".cyan());

    // 输入创建地址
    let path: String = Input::new()
        .with_prompt("请输入创建地址 (默认当前目录: ksmm)")
        .default("ksmm".to_string())
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
        println!("{} 当前目录名称 '{}' 不符合模块ID格式要求。", "⚠️", project_name);
        project_name = Input::new()
            .with_prompt("请输入项目名称 (必须以字母开头，只能包含字母、数字、点、下划线和连字符)")
            .interact_text()
            .unwrap();
        
        // 再次验证用户输入
        if !id_regex.is_match(&project_name) {
            println!("{} 模块ID格式无效。必须以字母开头，只能包含字母、数字、点、下划线和连字符。", "❌");
            return;
        }
    }

    // 使用项目名称作为id和name
    let id = project_name.clone();
    let name = project_name;

    // 获取git仓库信息
    let (update_json, git_username, branch, remote_url, status) = get_git_info();

    // 输出git信息
    if remote_url.is_some() || branch.is_some() {
        println!("{} 检测到 Git 仓库", "🔍");
        if let Some(branch_name) = &branch {
            println!("  {}: {}", "分支".blue(), branch_name.green());
        }
        if let Some(url) = &remote_url {
            println!("  {}: {}", "远程仓库".blue(), url.green());
        }
        if let Some(user) = &git_username {
            println!("  {}: {}", "用户名".blue(), user.green());
        }
        if let Some(url) = &remote_url {
            let github_regex = Regex::new(r"github\.com[\/:]([^\/]+)\/([^\/\.]+)").unwrap();
            if let Some(captures) = github_regex.captures(url) {
                if let Some(repo) = captures.get(2) {
                    let repo_name = repo.as_str().trim_end_matches(".git");
                    println!("  {}: {}", "仓库".blue(), repo_name.green());
                }
            }
        }
        if let Some(work_status) = &status {
            println!("  {}: {}", "状态".blue(), work_status.green());
        }
        println!();
    }

    // 确定作者
    let author = if let Some(username) = git_username {
        username
    } else {
        println!("{} 无法获取git用户信息，使用默认作者: ksmm", "ℹ️".blue());
        "ksmm".to_string()
    };

    // 默认版本信息
    let version = "0.1.0".to_string();
    let now = Utc::now();
    let version_code_int = (now.year() * 1000000 + now.month() as i32 * 10000 + now.day() as i32 * 100 + now.hour() as i32) as i32;

    // 自动生成描述
    let description = format!("一个用ksmm创建的{}模块", name);

    // 创建 system 目录
    create_system_directory(base_path);

    // 创建 .ksmm 配置目录
    create_ksmm_config(base_path);

    // 创建 module.prop
    create_module_prop(base_path, &id, &name, &version, version_code_int, &author, &description, &update_json);

    // 创建脚本文件
    create_script_files(base_path);

    // 创建 CHANGELOG.md
    create_changelog(base_path);

    // 检查是否需要执行按钮
    let action_path = base_path.join("action.sh");
    if action_path.exists() {
        println!("{}", format!("  [!] action.sh 文件已存在，跳过执行按钮配置").dimmed());
    } else {
        let need_action = Confirm::new()
            .with_prompt("是否需要执行按钮?")
            .default(true)
            .interact()
            .unwrap();

        if need_action {
            create_action_script(base_path);
        }
    }

    // 检查是否需要 webui
    let webroot_path = base_path.join("webroot");
    if webroot_path.exists() {
        println!("{}", format!("  [!] webroot 目录已存在，跳过WebUI配置").dimmed());
    } else {
        let need_webui = Confirm::new()
            .with_prompt("是否需要 WebUI?")
            .default(true)
            .interact()
            .unwrap();

        if need_webui {
            create_webui(base_path);
        }
    }

    println!("{} {}", "✅", "模块初始化完成!".cyan());
    println!();
    println!("{} 项目路径: {}", "📁", base_path.canonicalize().unwrap_or(base_path.to_path_buf()).display().green());
    println!("{} 项目ID: {}", "🔧", id.green());
    println!();
    println!("{} 下一步:", "📋");
    println!("  1. 编辑 {} 目录，添加你要修改的系统文件", "system/".green());
    println!("  2. 根据需要修改 {} 安装脚本", "customize.sh".green());
    println!("  3. 运行 {} 构建模块", "'ksmm build'".green());
    println!("  4. 运行 {} 安装到设备测试", "'ksmm install <模块文件>'".green());
    println!();
    println!("{} 项目初始化成功!", "🎉");
}