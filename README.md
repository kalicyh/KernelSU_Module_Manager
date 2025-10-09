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
