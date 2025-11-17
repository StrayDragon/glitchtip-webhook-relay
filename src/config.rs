use crate::types::Config;
use anyhow::Result;
use std::path::Path;

pub struct ConfigManager;

impl ConfigManager {
    pub fn load() -> Result<Config> {
        // Try to load from config.toml, config.json, or use defaults
        let config_paths = vec![
            "config.toml",
            "config.json",
            "/etc/glitchtip-relay/config.toml",
            "/etc/glitchtip-relay/config.json",
        ];

        for path in config_paths {
            if Path::new(path).exists() {
                log::info!("Loading config from {}", path);
                return Self::load_from_file(path);
            }
        }

        log::info!("No config file found, using defaults");
        Ok(Config::default())
    }

    fn load_from_file(path: &str) -> Result<Config> {
        let content = std::fs::read_to_string(path)?;

        if path.ends_with(".toml") {
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))
        } else if path.ends_with(".json") {
            serde_json::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
        } else {
            Err(anyhow::anyhow!("Unsupported config file format"))
        }
    }

    pub fn save_example_config() -> Result<()> {
        let example_config = r#"# GlitchTip to Feishu Webhook Relay Configuration

server_port = 8080

[[feishu_webhooks]]
name = "main_feishu"
url = "https://open.feishu.cn/open-apis/bot/v2/hook/YOUR_WEBHOOK_URL_HERE"
enabled = true
# secret = "your_secret_here"  # Optional: for signature verification

[[feishu_webhooks]]
name = "backup_feishu"
url = "https://open.feishu.cn/open-apis/bot/v2/hook/BACKUP_WEBHOOK_URL_HERE"
enabled = false
"#;

        std::fs::write("config.example.toml", example_config)?;
        println!("Example configuration saved to config.example.toml");
        Ok(())
    }
}

// Environment variable support
impl Config {
    pub fn apply_env_overrides(&mut self) {
        if let Ok(port) = std::env::var("PORT") {
            if let Ok(port) = port.parse::<u16>() {
                self.server_port = port;
            }
        }

        if let Ok(webhook_url) = std::env::var("FEISHU_WEBHOOK_URL") {
            if !webhook_url.is_empty() {
                self.feishu_webhooks.push(crate::types::FeishuWebhookConfig {
                    name: "env_webhook".to_string(),
                    url: webhook_url,
                    secret: std::env::var("FEISHU_WEBHOOK_SECRET").ok(),
                    enabled: true,
                });
            }
        }
    }
}