use crate::types::Config;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::fs;

#[derive(Debug)]
pub struct LazyConfigManager {
    config: RwLock<Option<Config>>,
}

impl LazyConfigManager {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(None),
        }
    }

    pub fn get_config(&self) -> Result<Config> {
        let config_guard = self.config.read().map_err(|e| {
            anyhow::anyhow!("Failed to acquire read lock on config: {}", e)
        })?;

        match config_guard.as_ref() {
            Some(config) => Ok(config.clone()),
            None => {
                // No config loaded yet, load it for the first time
                drop(config_guard);
                self.reload_config()
            }
        }
    }

    /// Reload configuration from file
    fn reload_config(&self) -> Result<Config> {
        log::info!("Loading configuration...");

        // Find and load config file
        let (config, config_path) = self.find_and_load_config()?;

        // Update config
        {
            let mut config_guard = self.config.write().map_err(|e| {
                anyhow::anyhow!("Failed to acquire write lock on config: {}", e)
            })?;
            *config_guard = Some(config.clone());
        }

        // Apply environment variable overrides
        let mut final_config = config;
        final_config.apply_env_overrides();

        log::info!("Configuration loaded successfully");
        if let Some(path) = config_path {
            log::info!("Loaded from: {:?}", path);
        }
        log::info!("Server port: {}", final_config.server_port);
        log::info!("Feishu webhooks configured: {}", final_config.feishu_webhooks.len());

        Ok(final_config)
    }

    /// Find configuration file and load it
    fn find_and_load_config(&self) -> Result<(Config, Option<PathBuf>)> {
        let config_paths = vec![
            PathBuf::from("config.toml"),
            PathBuf::from("config.json"),
            PathBuf::from("/etc/glitchtip-relay/config.toml"),
            PathBuf::from("/etc/glitchtip-relay/config.json"),
        ];

        for path in config_paths {
            if path.exists() {
                log::info!("Found config file: {:?}", path);

                match Self::load_from_file(&path) {
                    Ok(config) => {
                        return Ok((config, Some(path)));
                    }
                    Err(e) => {
                        log::warn!("Failed to load config file {:?}: {}, skipping", path, e);
                        // Continue to try other config files
                    }
                }
            }
        }

        log::info!("No valid config file found, will use defaults and environment variables");
        Ok((Config::default(), None))
    }

    /// Load configuration from a specific file
    fn load_from_file(path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)?;

        if let Some(path_str) = path.to_str() {
            if path_str.ends_with(".toml") {
                toml::from_str(&content).map_err(|e| {
                    anyhow::anyhow!("Failed to parse TOML from {:?}: {}", path, e)
                })
            } else if path_str.ends_with(".json") {
                serde_json::from_str(&content).map_err(|e| {
                    anyhow::anyhow!("Failed to parse JSON from {:?}: {}", path, e)
                })
            } else {
                Err(anyhow::anyhow!("Unsupported config file format: {:?}", path))
            }
        } else {
            Err(anyhow::anyhow!("Invalid path: {:?}", path))
        }
    }

    /// Force reload configuration from file
    pub fn force_reload(&self) -> Result<Config> {
        log::info!("Force reloading configuration...");
        self.reload_config()
    }
}

impl Default for LazyConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy ConfigManager for backward compatibility
pub struct ConfigManager;

impl ConfigManager {
    #[allow(dead_code)]
    pub fn load() -> Result<Config> {
        let lazy_manager = LazyConfigManager::new();
        lazy_manager.get_config()
    }

    pub fn save_example_config() -> Result<()> {
        let example_config = r#"# GlitchTip to Feishu Webhook Relay Configuration

server_port = 8080

# Optional: Template directory for custom message templates
# If not specified, embedded default templates will be used
# template_dir = "/path/to/templates"

# Feishu Webhook Configurations
# You can configure multiple Feishu webhooks to send messages to different groups

[[feishu_webhooks]]
name = "main_feishu"
url = "https://open.feishu.cn/open-apis/bot/v2/hook/YOUR_WEBHOOK_URL_HERE"
enabled = true
# secret = "your_secret_here"  # Optional: for signature verification

[[feishu_webhooks]]
name = "backup_feishu"
url = "https://open.feishu.cn/open-apis/bot/v2/hook/BACKUP_WEBHOOK_URL_HERE"
enabled = false

# Environment Variables (Optional)
# You can override config values with environment variables:
#
# PORT - Server port (e.g., PORT=9000)
# FEISHU_WEBHOOK_URL - Primary Feishu webhook URL
# FEISHU_WEBHOOK_SECRET - Secret for signature verification
# ENABLE_HASH_COLORS - Enable/disable dynamic color generation for project/environment/server fields
#   * true (default): Generate colors based on hash values (12 different colors)
#   * false: Use fixed colors (red, carmine, orange)
#
# Examples:
#   ENABLE_HASH_COLORS=true   # Dynamic colors (default)
#   ENABLE_HASH_COLORS=false  # Fixed colors
#   PORT=9000                 # Custom port
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