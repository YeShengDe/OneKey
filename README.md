# ONEKEY TUI 项目设置说明

## 项目文件结构

请按照以下结构创建项目文件：

```bash
# 创建项目
cargo new vps-tui
cd vps-tui

# 创建目录结构
mkdir -p src/handlers
mkdir -p src/utils

# 复制主要文件
# 将提供的代码复制到对应文件中：
# - Cargo.toml (替换原有文件)
# - src/main.rs
# - src/app.rs
# - src/ui.rs
# - src/menu.rs
# - src/events.rs
# - src/types.rs

# 创建 handlers 模块文件
# - src/handlers/mod.rs
# - src/handlers/system_info.rs
# - src/handlers/disk_test.rs
# - src/handlers/cpu_test.rs (使用模板)
# - src/handlers/network_test.rs (使用模板)
# - src/handlers/sing_box.rs (使用模板)
# - src/handlers/xray.rs (使用模板)
# - src/handlers/port_manager.rs (使用模板)
# - src/handlers/k3s.rs (使用模板)
# - src/handlers/k8s.rs (使用模板)
# - src/handlers/tcp_optimizer.rs (使用模板)

# 创建 utils 模块文件
# - src/utils/mod.rs
# - src/utils/command.rs
# - src/utils/formatter.rs

# 创建项目文档
# - README.md
```

## 快速实现剩余的处理器模块

对于还未实现的处理器模块（cpu_test.rs, network_test.rs 等），可以使用我提供的模板代码。每个模块都遵循相同的模式：

```rust
pub fn get_info() -> String {
    let mut content = String::from("=== 功能标题 ===\n\n");
    // 添加功能说明和命令
    content
}
```

## 编译和运行

```bash
# 编译项目
cargo build

# 运行调试版本
cargo run

# 编译优化版本（体积更小）
cargo build --release

# 运行优化版本
./target/release/vps-tui
```

## 扩展功能

### 1. 实现实际的系统命令执行

取消注释 Cargo.toml 中的依赖项，然后在处理器中使用：

```rust
use crate::utils::command::CommandRunner;

pub fn get_info() -> String {
    if let Ok(output) = CommandRunner::run("uname", &["-a"]) {
        // 处理实际输出
    }
    // ...
}
```

### 2. 添加异步支持

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### 3. 添加配置文件支持

```toml
# Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

然后创建 config.rs 模块来管理配置。

## 下一步

1. **完善功能实现**：将模拟数据替换为实际系统命令
2. **添加错误处理**：更好的错误提示和恢复机制
3. **增加交互功能**：如输入框、确认对话框等
4. **性能优化**：缓存常用数据，减少系统调用
5. **国际化**：支持多语言界面

## 故障排除

如果遇到编译错误：

1. 确保 Rust 版本 >= 1.70：`rustc --version`
2. 更新依赖：`cargo update`
3. 清理缓存：`cargo clean`
4. 检查所有文件是否正确创建

如果运行时显示异常：

1. 确保终端支持 UTF-8
2. 尝试不同的终端模拟器
3. 检查终端窗口大小（至少 80x24）