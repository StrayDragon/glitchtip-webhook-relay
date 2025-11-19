# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GlitchTip to Feishu Webhook Relay is a Rust HTTP service that converts GlitchTip webhooks into Feishu (Lark) message formats. It acts as a relay service between error tracking and Chinese messaging platforms.

## Architecture

The service follows a modular architecture with clear separation of concerns:

- **HTTP Server** (`main.rs`): Actix-web server with OpenAPI documentation
- **Configuration** (`config.rs`): TOML/JSON config with environment variable support
- **Types** (`types.rs`): Webhook structure definitions and OpenAPI schemas
- **Converter** (`converter.rs`): Three-strategy message conversion (text → rich text → card)
- **Service** (`service.rs`): Webhook processing with error handling and retry logic

## Essential Commands

```bash
# Development
cargo run                    # Start development server
cargo build                  # Build debug version
cargo test                   # Run tests (currently no test suite)
cargo check                  # Quick syntax/type checking

# Production
cargo build --release        # Build optimized binary (11.8MB)
./target/release/glitchtip-webhook-relay  # Run production binary

# Configuration
cargo run -- --example-config  # Generate example config
cargo run -- --help           # Show all options

# Logging
RUST_LOG=debug cargo run     # Debug logging
RUST_LOG=error cargo run     # Error-only logging
```

## Configuration Setup

The service supports multiple configuration methods:

1. **Config File**: Create `config.toml` from `config.example.toml`
2. **Environment Variables**: `PORT`, `FEISHU_WEBHOOK_URL`, `FEISHU_WEBHOOK_SECRET`
3. **Command Line**: `--port`, `--config` arguments

Default locations checked: `./config.toml`, `/etc/glitchtip-relay/config.toml`

## API Endpoints

- `GET /` - Service information and endpoint list
- `GET /config` - Current configuration (sanitized, URLs masked)
- `POST /webhook/glitchtip` - Main webhook processing endpoint
- `GET /docs` - Interactive OpenAPI documentation (Scalar UI)
- `GET /api-docs/openapi.json` - OpenAPI specification

## Development Workflow

1. Generate example config: `cargo run -- --example-config`
2. Configure Feishu webhook URLs in `config.toml` or environment
3. Start development server: `cargo run`
4. Test with `curl -X POST http://localhost:8080/webhook/glitchtip`
5. Access docs at `http://localhost:8080/docs`

## Key Features

- **Multi-format conversion**: Text → Rich Text → Interactive Card formats
- **Multiple webhooks**: Forward to multiple Feishu endpoints simultaneously
- **Fallback handling**: Automatic format fallback on conversion failures
- **Interactive docs**: Complete OpenAPI documentation with Scalar UI

## Testing Status

**No test suite currently exists**. The project is in MVP stage and would benefit from:
- Unit tests for conversion functions in `converter.rs`
- Integration tests for webhook processing in `service.rs`
- Configuration loading tests
- Error handling and retry logic tests

## Message Conversion

The converter implements three strategies with automatic fallback:
1. **Text messages**: Basic formatted text
2. **Rich text**: Markdown formatting with structured elements
3. **Interactive cards**: Structured cards with buttons and metadata

## Error Handling

- Comprehensive error logging with configurable levels
- Automatic fallback between message formats
- HTTP client retry logic for external webhook calls
- Sensitive data masking in configuration responses