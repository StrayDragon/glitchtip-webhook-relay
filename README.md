# GlitchTip to Feishu Webhook Relay

A Rust service that converts GlitchTip error alerts into Feishu interactive cards.

## Quick Start

### Build and Run
```bash
cargo build
cargo run
```

### Configuration
Generate example config:
```bash
cargo run -- --example-config
```

Edit `config.yaml` with your Feishu bot webhook URLs.

### Usage
Set GlitchTip webhook URL to:
```
http://your-server:7876/i/your_endpoint_name
```

Alerts will be automatically forwarded to configured Feishu bots.

### Hot Reload
Reload config without restarting:
```bash
curl http://localhost:7876/internal/config/reload
```

## Documentation
See [CLAUDE.md](./CLAUDE.md) for details
