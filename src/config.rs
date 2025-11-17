use crate::types::Config;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::fs;
use md5;

/// Configuration metadata for tracking file changes
#[derive(Debug, Clone)]
struct ConfigMetadata {
    path: Option<PathBuf>,
    md5_hash: Option<String>,
}

/// Lazy loading configuration manager with hot reload support
#[derive(Debug)]
pub struct LazyConfigManager {
    config: RwLock<Option<Config>>,
    metadata: RwLock<ConfigMetadata>,
}

impl LazyConfigManager {
    /// Create a new lazy config manager
    pub fn new() -> Self {
        Self {
            config: RwLock::new(None),
            metadata: RwLock::new(ConfigMetadata {
                path: None,
                md5_hash: None,
            }),
        }
    }

    /// Get configuration, loading it only if necessary (first time access)
    pub fn get_config(&self) -> Result<Config> {
        // Return a clone of the current config
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

    /// Check if configuration should be reloaded
    fn should_reload(&self) -> Result<bool> {
        let metadata_guard = self.metadata.read().map_err(|e| {
            anyhow::anyhow!("Failed to acquire read lock on metadata: {}", e)
        })?;

        let config_guard = self.config.read().map_err(|e| {
            anyhow::anyhow!("Failed to acquire read lock on config: {}", e)
        })?;

        // If no config is loaded, we need to load
        if config_guard.is_none() {
            return Ok(true);
        }

        // If we have a config file path, check if file still exists and if hash changed
        if let Some(config_path) = &metadata_guard.path {
            if !config_path.exists() {
                log::warn!("Config file {:?} no longer exists, will use defaults", config_path);
                return Ok(true);
            }

            // Calculate current MD5 hash
            match self.calculate_file_hash(config_path) {
                Ok(current_hash) => {
                    if metadata_guard.md5_hash.as_ref() != Some(&current_hash) {
                        log::info!("Config file {:?} has changed, will reload", config_path);
                        return Ok(true);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to calculate hash for config file {:?}: {}", config_path, e);
                    return Ok(true); // Reload on hash calculation error
                }
            }
        }

        Ok(false)
    }

    /// Reload configuration from file
    fn reload_config(&self) -> Result<Config> {
        log::info!("Loading configuration...");

        // Find and load config file
        let (config, config_path, current_hash) = self.find_and_load_config()?;

        // Update both config and metadata atomically
        {
            let mut config_guard = self.config.write().map_err(|e| {
                anyhow::anyhow!("Failed to acquire write lock on config: {}", e)
            })?;
            *config_guard = Some(config.clone());
        }

        {
            let mut metadata_guard = self.metadata.write().map_err(|e| {
                anyhow::anyhow!("Failed to acquire write lock on metadata: {}", e)
            })?;
            *metadata_guard = ConfigMetadata {
                path: config_path,
                md5_hash: current_hash,
            };
        }

        // Apply environment variable overrides
        let mut final_config = config;
        final_config.apply_env_overrides();

        log::info!("Configuration loaded successfully");
        log::info!("Server port: {}", final_config.server_port);
        log::info!("Feishu webhooks configured: {}", final_config.feishu_webhooks.len());

        Ok(final_config)
    }

    /// Find configuration file and load it
    fn find_and_load_config(&self) -> Result<(Config, Option<PathBuf>, Option<String>)> {
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
                        match self.calculate_file_hash(&path) {
                            Ok(hash) => {
                                return Ok((config, Some(path), Some(hash)));
                            }
                            Err(e) => {
                                log::warn!("Failed to calculate hash for config file {:?}: {}, skipping", path, e);
                                // Continue to try other config files
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to load config file {:?}: {}, skipping", path, e);
                        // Continue to try other config files
                    }
                }
            }
        }

        log::info!("No valid config file found, will use defaults and environment variables");
        Ok((Config::default(), None, None))
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

    /// Calculate MD5 hash of a file
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let content = fs::read(path)?;
        Ok(format!("{:x}", md5::compute(content)))
    }

    /// Force reload configuration (useful for testing or manual triggers)
    pub fn force_reload(&self) -> Result<Config> {
        log::info!("Force reloading configuration...");

        // Check if we should reload (file changed)
        if self.should_reload()? {
            self.reload_config()
        } else {
            log::info!("Configuration unchanged, returning current version");
            // Return current config
            let config_guard = self.config.read().map_err(|e| {
                anyhow::anyhow!("Failed to acquire read lock on config: {}", e)
            })?;

            match config_guard.as_ref() {
                Some(config) => Ok(config.clone()),
                None => {
                    // This shouldn't happen, but handle it gracefully
                    drop(config_guard);
                    self.reload_config()
                }
            }
        }
    }

    /// Get current configuration metadata (for debugging)
    pub fn get_metadata(&self) -> Result<(Option<PathBuf>, Option<String>)> {
        let metadata_guard = self.metadata.read().map_err(|e| {
            anyhow::anyhow!("Failed to acquire read lock on metadata: {}", e)
        })?;

        Ok((metadata_guard.path.clone(), metadata_guard.md5_hash.clone()))
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
    pub fn load() -> Result<Config> {
        let lazy_manager = LazyConfigManager::new();
        lazy_manager.get_config()
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