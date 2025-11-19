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
        let example_config = r#"# GlitchTip to Feishu Webhook Relay Configuration (v2.0)
# Only YAML format is supported. TOML is no longer supported.

# Server configuration
server_port: 7876

# Optional: Template directory for custom message templates
# template_dir: "/path/to/templates"

# Webhook configurations
webhooks:
  # Example 1: Minimal configuration (using all defaults)
  # This example shows the simplest possible configuration.
  # All optional fields use their default values.
  - name: minimal_example
    url:
      - "https://open.feishu.cn/open-apis/bot/v2/hook/MINIMAL_EXAMPLE"
    enabled: true
    forward_config:
      type: feishu_robot_msg
    # config section is completely omitted - defaults to:
    #   n_par: 1
    #   timeout: 30
    #   retry: 3

  # Example 2: Configuration with some defaults
  # Only specifying non-default values
  - name: partial_defaults
    url:
      - "https://open.feishu.cn/open-apis/bot/v2/hook/PARTIAL_DEFAULTS"
    enabled: true
    forward_config:
      type: feishu_robot_msg
      feishu_robot_msg:
        card_theme: "blue"
    config:
      n_par: 5  # Only overriding n_par, timeout and retry use defaults

  # Example 3: Full configuration
  # Specifying all available options
  - name: full_example
    url:
      - "https://open.feishu.cn/open-apis/bot/v2/hook/MAIN"
      - "https://open.feishu.cn/open-apis/bot/v2/hook/BACKUP"
    enabled: true
    forward_config:
      type: feishu_robot_msg
      feishu_robot_msg:
        card_theme: "red"
        mention_all: true
        buttons:
          - text: "View Details"
            url: "https://your-domain.com/details"
          - text: "Emergency"
            url: "https://your-domain.com/emergency"
        color_mapping:
          error: "red"
          warning: "orange"
          info: "blue"
    config:
      n_par: 10         # Max 10 concurrent requests
      timeout: 60       # Request timeout in seconds
      retry: 5          # Number of retries on failure

  # Example 4: Disabled webhook
  # Returns HTTP 200 but doesn't forward the message
  - name: disabled_example
    url:
      - "https://open.feishu.cn/open-apis/bot/v2/hook/DISABLED"
    enabled: false

  # Real-world examples:
  - name: dev_team
    url:
      - "https://open.feishu.cn/open-apis/bot/v2/hook/DEV_TEAM_WEBHOOK"
    enabled: true
    forward_config:
      type: feishu_robot_msg
      feishu_robot_msg:
        card_theme: "blue"
        mention_all: false
    config:
      n_par: 1          # Sequential for dev team
      timeout: 30
      retry: 3

  - name: production
    url:
      - "https://open.feishu.cn/open-apis/bot/v2/hook/PROD_MAIN"
      - "https://open.feishu.cn/open-apis/bot/v2/hook/PROD_BACKUP"
      - "https://open.feishu.cn/open-apis/bot/v2/hook/PROD_ARCHIVE"
    enabled: true
    forward_config:
      type: feishu_robot_msg
      feishu_robot_msg:
        card_theme: "red"
        mention_all: true
    config:
      n_par: 10         # High concurrency for production
      timeout: 60
      retry: 5

  # Future: Enterprise WeChat support
  # - name: wecom_team
  #   url:
  #     - "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=xxx"
  #   enabled: true
  #   forward_config:
  #     type: wecom_webhook
  #     wecom_webhook:
  #       corp_id: "your_corp_id"
  #       corp_secret: "your_corp_secret"
  #   config:
  #     n_par: 5

  # Future: DingTalk support
  # - name: dingtalk_team
  #   url:
  #     - "https://oapi.dingtalk.com/robot/send?access_token=xxx"
  #   enabled: true
  #   forward_config:
  #     type: dingtalk_webhook
  #     dingtalk_webhook:
  #       access_token: "your_access_token"
  #       secret: "your_secret"
  #   config:
  #     n_par: 1

# Environment Variables (Optional)
# You can override config values with environment variables:
#
# PORT - Server port (e.g., PORT=9000)
# FEISHU_WEBHOOK_URL - Primary Feishu webhook URL
# FEISHU_WEBHOOK_SECRET - Secret for signature verification
# ENABLE_HASH_COLORS - Enable/disable dynamic color generation
#   * true (default): Generate colors based on hash values
#   * false: Use fixed colors
#
# Examples:
#   ENABLE_HASH_COLORS=true   # Dynamic colors (default)
#   ENABLE_HASH_COLORS=false  # Fixed colors
#   PORT=9000                 # Custom port
"#;

        std::fs::write("config.example.yaml", example_config)?;
        println!("Example YAML configuration saved to config.example.yaml");
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