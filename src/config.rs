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

        let (config, config_path) = self.find_and_load_config()?;

        {
            let mut config_guard = self.config.write().map_err(|e| {
                anyhow::anyhow!("Failed to acquire write lock on config: {}", e)
            })?;
            *config_guard = Some(config.clone());
        }

        let mut final_config = config;
        final_config.apply_env_overrides();

        log::info!("Configuration loaded successfully");
        if let Some(path) = config_path {
            log::info!("Loaded from: {:?}", path);
        }
        log::info!("Server host: {}", final_config.server_host);
        log::info!("Server port: {}", final_config.server_port);
        log::info!("Webhooks configured: {}", final_config.webhooks.len());

        Ok(final_config)
    }

    /// Find configuration file and load it
    fn find_and_load_config(&self) -> Result<(Config, Option<PathBuf>)> {
        let config_paths = vec![
            // YAML files (only supported format in v2.0+)
            PathBuf::from("config.yaml"),
            PathBuf::from("config.yml"),
            PathBuf::from("/etc/glitchtip-relay/config.yaml"),
            PathBuf::from("/etc/glitchtip-relay/config.yml"),
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
            // Only YAML is supported in v2.0+
            if path_str.ends_with(".yaml") || path_str.ends_with(".yml") {
                serde_yaml::from_str(&content).map_err(|e| {
                    anyhow::anyhow!("Failed to parse YAML from {:?}: {}", path, e)
                })
            } else {
                Err(anyhow::anyhow!("Unsupported config file format: {:?}. Only YAML (.yaml, .yml) is supported in v2.0+", path))
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
        let example_config = include_str!("../config.example.yaml");

        std::fs::write("config.example.yaml", example_config)?;
        log::info!("Example YAML configuration saved to config.example.yaml");
        Ok(())
    }
}

impl Config {
    pub fn apply_env_overrides(&mut self) {
        if let Ok(port) = std::env::var("GWR_PORT") {
            if let Ok(port) = port.parse::<u16>() {
                self.server_port = port;
            }
        }

        if let Ok(host) = std::env::var("GWR_SERVER_HOST") {
            self.server_host = host;
        }

        if let Ok(webhook_url) = std::env::var("GWR_FEISHU_WEBHOOK_URL") {
            if !webhook_url.is_empty() {
                self.webhooks.push(crate::types::WebhookConfig {
                    name: "env_webhook".to_string(),
                    url: vec![webhook_url],
                    enabled: true,
                    forward_config: crate::types::ForwardConfig::FeishuRobotMsg(
                        crate::types::FeishuConfig {
                            card_theme: None,
                            mention_all: None,
                            buttons: None,
                            color_mapping: None,
                        }
                    ),
                    config: crate::types::WebhookRuntimeConfig {
                        n_par: 1,
                        timeout: 30,
                        retry: 3,
                    },
                });
            }
        }
    }
}