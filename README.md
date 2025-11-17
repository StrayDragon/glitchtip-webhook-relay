# GlitchTip to Feishu Webhook Relay

一个用于将 GlitchTip webhooks 转换为飞书消息格式的中继服务。

## 功能特性

- 解析 GlitchTip Slack 格式的 webhook
- 转换为多种飞书消息格式（文本、富文本、卡片）
- 支持多个飞书 webhook 配置
- 支持环境变量配置
- 健康检查端点
- 详细的日志记录

## 快速开始

### 1. 生成示例配置

```bash
cargo run -- --example-config
```

这将创建 `config.example.toml` 文件。

### 2. 配置飞书 Webhook

复制示例配置文件并根据需要修改：

```bash
cp config.example.toml config.toml
```

编辑 `config.toml`：

```toml
server_port = 8080

[[feishu_webhooks]]
name = "main_feishu"
url = "https://open.feishu.cn/open-apis/bot/v2/hook/YOUR_WEBHOOK_URL_HERE"
enabled = true
# secret = "your_secret_here"  # 可选：签名验证
```

### 3. 启动服务

```bash
cargo run
```

服务将在 `http://localhost:8080` 启动。

### 4. 配置 GlitchTip

在 GlitchTip 中设置 webhook URL 为：
```
http://your-server:8080/webhook/glitchtip
```

## 环境变量配置

你也可以使用环境变量进行配置：

```bash
export PORT=8080
export FEISHU_WEBHOOK_URL="https://open.feishu.cn/open-apis/bot/v2/hook/YOUR_WEBHOOK_URL_HERE"
export FEISHU_WEBHOOK_SECRET="your_secret_here"  # 可选

cargo run
```

## API 端点

### 主页
```
GET /
```

返回服务信息和可用端点。

### 健康检查
```
GET /health
```

返回服务健康状态。

### 配置信息
```
GET /config
```

返回当前的配置信息（敏感信息已隐藏）。

### Webhook 接收
```
POST /webhook/glitchtip
```

接收 GlitchTip webhook 并转发到配置的飞书 webhook。

## 支持的飞书消息格式

服务会自动尝试以下格式（按优先级）：

1. **文本消息** - 简单的文本格式，包含基本的错误信息
2. **富文本消息** - 包含格式化的错误详情和链接
3. **卡片消息** - 结构化的卡片格式，包含操作按钮

## 日志

使用环境变量控制日志级别：

```bash
# 默认信息级别
cargo run

# 调试级别
RUST_LOG=debug cargo run

# 错误级别
RUST_LOG=error cargo run
```

## 示例 GlitchTip Webhook

服务期望接收类似以下格式的 GlitchTip webhook：

```json
{
  "alias": "GlitchTip",
  "attachments": [
    {
      "color": "#e52b50",
      "fields": [
        { "short": true, "title": "Project", "value": "my_project" },
        { "short": true, "title": "Environment", "value": "production" }
      ],
      "title": "Error: Something went wrong",
      "title_link": "https://glitchtip.example.com/issues/123"
    }
  ],
  "text": "GlitchTip Alert"
}
```

## 故障排除

### 1. Webhook 未被接收

- 检查服务器日志：`RUST_LOG=debug cargo run`
- 验证 GlitchTip 配置的 URL 是否正确
- 确认防火墙设置允许访问

### 2. 飞书消息未发送

- 检查飞书 webhook URL 是否有效
- 验证机器人是否在目标群组中
- 查看服务器日志中的错误信息

### 3. 消息格式问题

服务支持多种消息格式，会自动选择最合适的格式。如果遇到问题，请查看服务器日志了解具体的格式尝试过程。

## 开发

### 构建

```bash
cargo build
```

### 运行测试

```bash
cargo test
```

### 依赖项

- `actix-web` - HTTP 服务器框架
- `serde` - JSON 序列化/反序列化
- `reqwest` - HTTP 客户端
- `toml` - TOML 配置文件解析
- `chrono` - 时间处理
- `env_logger` - 日志记录

## 许可证

MIT License