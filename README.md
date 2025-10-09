# KernelSU_Module_Manager
开发与管理 KernelSU 模块

## 命令

```bash
ksmm help      # 显示帮助信息
ksmm list       # 列出已安装模块
ksmm install x  # 安装模块 x
ksmm remove y   # 移除模块 y
ksmm version   # 显示版本信息
```
## 模块结构

``` plaintext
$MODID/                        # 模块目录名，与模块 ID 相同
│
├── module.prop                # 模块配置文件
├── system/                    # 将被挂载到 /system（除非存在 skip_mount）
│   └── (空目录或你的修改内容)
│
├── skip_mount                 # [可选] 若存在，则不挂载 /system
├── disable                    # [可选] 若存在，则禁用模块
├── remove                     # [可选] 若存在，则下次重启后删除模块
│
├── post-fs-data.sh            # [可选] 在 post-fs-data 阶段运行
├── post-mount.sh              # [可选] 在 post-mount 阶段运行
├── service.sh                 # [可选] 在 late_start 阶段运行
├── boot-completed.sh          # [可选] 在系统启动完成后运行
├── uninstall.sh               # [可选] 模块卸载时执行
│
├── system.prop                # [可选] 开机时通过 resetprop 应用的系统属性
├── sepolicy.rule              # [可选] 自定义 SELinux 策略规则
│
├── vendor/                    # 自动生成：若 /system/vendor 是符号链接
├── product/                   # 自动生成：若 /system/product 是符号链接
├── system_ext/                # 自动生成：若 /system/system_ext 是符号链接
│
└── ...                        # 其他自定义文件或文件夹
```

## 开发者调试

```bash
cargo build               # 编译项目
cargo run <command>    # 运行项目并传递命令
./target/debug/ksmm help      # 显示帮助信息
```

## 预期使用的依赖

| 功能                | 推荐库                                               |
| ----------------- | ------------------------------------------------- |
| 命令行参数解析           | [clap](https://crates.io/crates/clap)             |
| 配置文件解析（YAML/TOML） | [serde + toml/yaml](https://serde.rs/)            |
| 日志输出              | [tracing](https://crates.io/crates/tracing)       |
| 彩色输出              | [owo-colors](https://crates.io/crates/owo-colors) |
| 进度条               | [indicatif](https://crates.io/crates/indicatif)   |
| 命令行交互             | [dialoguer](https://crates.io/crates/dialoguer)   |
| 异步 CLI            | [tokio](https://crates.io/crates/tokio)           |
