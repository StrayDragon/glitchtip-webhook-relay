# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GlitchTip to Feishu Webhook Relay is a Rust-based webhook forwarding service that converts GlitchTip error alerts into Feishu (Lark) interactive card messages. It supports multiple endpoints, parallel forwarding, and customizable message templates.

**Tech Stack:**
- Rust 2024 edition with Actix Web framework
- YAML-based configuration (TOML no longer supported in v2.0+)
- Jinja2 templates for message formatting
- OpenAPI 3.1 with Utoipa for API documentation

## Common Development Commands

```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Generate example YAML configuration
cargo run -- --example-config

# Run release build
cargo run --release

# Check for errors without building
cargo check

# View documentation
cargo doc --open

# Run specific test (if tests exist)
cargo test -- <test_name>

# Watch for changes and rebuild (requires cargo-watch)
cargo watch -x run
```

## Configuration

The service uses YAML configuration files only (TOML support removed in v2.0+).

**Configuration file locations (in order of precedence):**
1. `config.yaml` (project root)
2. `config.yml` (project root)
3. `/etc/glitchtip-relay/config.yaml`
4. `/etc/glitchtip-relay/config.yml`

**Generate example config:**
```bash
cargo run -- --example-config
```

**Environment Variables:**
- `PORT` - Server port override
- `FEISHU_WEBHOOK_URL` - Single webhook URL for quick testing
- `FEISHU_WEBHOOK_SECRET` - Secret for signature verification
- `ENABLE_HASH_COLORS` - Enable/disable dynamic color generation (true/false, default: true)

## Architecture Overview

### Core Components

**1. Main Entry Point (`src/main.rs`)**
- Initializes the HTTP server on configured port
- Sets up LazyConfigManager for hot-reload configuration
- Registers routes with Actix Web
- Provides OpenAPI documentation endpoints

**2. Configuration Management (`src/config.rs`)**
- `LazyConfigManager`: Thread-safe config with lazy loading and hot-reload
- Supports YAML format only
- Environment variable overrides
- Automatic config file discovery
- Force reload capability via `/internal/config/reload`

**3. Routing (`src/routes.rs`)**
- Centralized route path constants
- `/i/{endpoint_id}` - Endpoint-based webhook routing (v2.0)
- `/internal/config/reload` - Internal config reload endpoint
- `/dev/openapi-ui/scalar` - OpenAPI UI
- `/dev/openapi.json` - OpenAPI JSON spec

**4. Service Layer (`src/service.rs`)**
- `WebhookService`: Core business logic
- Parallelism control (sequential vs. concurrent forwarding)
- Platform-specific message forwarding (Feishu, WeCom, DingTalk)
- Error handling and logging
- Retry logic and timeout management

**5. Conversion Engine (`src/converter.rs`)**
- Converts GlitchTip Slack format to Feishu interactive cards
- Dynamic template loading (external or embedded)
- Jinja2 template support with metadata extraction
- Hash-based dynamic color generation
- Fallback card construction if template loading fails

**6. Type System (`src/types.rs`)**
- All data structures and enums
- Serde-based serialization/deserialization
- OpenAPI schema generation
- Platform-specific configurations (Feishu, WeCom, DingTalk)

### Data Flow

```
GlitchTip Webhook → /i/{endpoint_id} → WebhookService
                                             ↓
                                      Converter (extract metadata)
                                             ↓
                                      Render Jinja2 template
                                             ↓
                                      Send to Feishu Webhook(s)
```

**Parallelism Options:**
- `n_par = 1`: Sequential forwarding
- `n_par > 1`: Parallel forwarding with max concurrency limit
- Multiple URLs: Always sent to all configured endpoints

## Key Configuration Structure

```yaml
server_port: 7876
template_dir: "/path/to/templates"  # optional

webhooks:
  - name: "production"              # unique identifier
    url:
      - "https://open.feishu.cn/open-apis/bot/v2/hook/MAIN"
      - "https://open.feishu.cn/open-apis/bot/v2/hook/BACKUP"
    enabled: true
    forward_config:
      type: feishu_robot_msg        # or wecom_webhook, dingtalk_webhook
      feishu_robot_msg:
        card_theme: "red"           # optional
        mention_all: true           # optional
        buttons:                    # optional
          - text: "View Details"
            url: "https://..."
        color_mapping:              # optional
          error: "red"
          warning: "orange"
          info: "blue"
    config:
      n_par: 10                     # parallel requests (default: 1)
      timeout: 60                   # seconds (default: 30)
      retry: 5                      # retry count (default: 3)
```

## Message Templates

**Template Location:** `templates/feishu/default.json.jinja2`

Templates support Jinja2 syntax with the following metadata variables:
- `webhook_alias` - Webhook source name
- `issue_identifier` - Issue identifier from GlitchTip
- `exception_class_name` - Error class name
- `full_error_message` - Complete error message
- `issue_url` - Link to error details
- `project_id` - Project name
- `environment_name` - Environment (dev, prod, etc.)
- `hostname` - Server name
- `commit_hash` - Release version
- `current_timestamp` - Current time
- `project_bg_color`, `project_fg_color` - Dynamic colors
- `env_bg_color`, `env_fg_color` - Dynamic colors
- `host_bg_color`, `host_fg_color` - Dynamic colors
- `element_id_1`, `element_id_2`, `element_id_3` - Random IDs

**Template Loading Priority:**
1. External template from `template_dir` (if configured)
2. Embedded fallback template

## API Endpoints

**Webhook Ingestion:**
- `POST /i/{endpoint_id}` - Receive and forward webhooks
  - Path parameter: endpoint_id matches webhook config name
  - Request body: GlitchTip Slack format JSON
  - Response: 200 on success, 404 if endpoint not found

**Configuration Management:**
- `GET /internal/config/reload` - Force reload configuration
  - Response: 200 on success, 400 on error

**Development/Testing:**
- `GET /` - Root endpoint (health check)
- `GET /dev/openapi.json` - OpenAPI JSON specification
- `GET /dev/openapi-ui/scalar` - Interactive API documentation

## Testing

**Test with sample webhook:**
```bash
# The service must be running first (cargo run)
curl -X POST http://localhost:7876/i/your_endpoint_name \
  -H "Content-Type: application/json" \
  -d @path/to/glitchtip.webhook.json
```

**Test configuration reload:**
```bash
curl http://localhost:7876/internal/config/reload
```

## Development Notes

**Build Requirements:**
- Rust stable toolchain
- Internet connection for dependencies (reqwest, actix-web, etc.)

**Configuration Changes:**
- Hot-reload enabled via `/internal/config/reload` endpoint
- No server restart required for config changes
- Environment variables override file-based config

**Template System:**
- Jinja2-powered with metadata extraction
- Dynamic color generation based on string hashes
- Embedded template as fallback
- External template directory support

**Supported Platforms:**
- ✅ Feishu (Lark) - Fully implemented
- ⏳ WeCom (WeChat Work) - Stubbed
- ⏳ DingTalk - Stubbed

**Concurrency Model:**
- Async/await throughout
- Config loading is thread-safe (RwLock)
- Parallel webhook forwarding with configurable limits
- Buffer management for high-throughput scenarios

## Important Files

- `src/main.rs` - Server initialization and route registration
- `src/config.rs` - Configuration management with lazy loading
- `src/service.rs` - Webhook forwarding logic
- `src/converter.rs` - Message format conversion
- `src/types.rs` - Data structures
- `src/routes.rs` - Route definitions
- `templates/feishu/default.json.jinja2` - Default message template
- `config.example.yaml` - Configuration example

## Known Limitations

- YAML only (TOML removed in v2.0)
- WeCom and DingTalk support not yet implemented
- Template directory must be configured manually
- No built-in authentication/authorization
- No metrics or monitoring endpoints
