# KernelSU_Module_Manager
开发与管理 KernelSU 模块

## 命令

```bash
ksmm help      # 显示帮助信息
ksmm init      # 初始化模块
ksmm build     # 构建模块
ksmm sign <file> # 签名文件
ksmm key new <name> # 创建新密钥
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

## 依赖

| 功能                | 推荐库                                               |
| ----------------- | ------------------------------------------------- |
| 命令行参数解析           | [clap](https://crates.io/crates/clap)             |
| 彩色输出              | [owo-colors](https://crates.io/crates/owo-colors) |
| 日期时间处理            | [chrono](https://crates.io/crates/chrono)         |
| 正则表达式             | [regex](https://crates.io/crates/regex)           |
| 命令行交互             | [dialoguer](https://crates.io/crates/dialoguer)   |
