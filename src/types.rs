use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// GlitchTip webhook structures based on OpenAPI 3.1 spec
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "GlitchTip Webhook", description = "GlitchTip Slack format webhook payload")]
pub struct GlitchTipSlackWebhook {
    /// Webhook alias or source name
    pub alias: String,
    /// Main message text
    pub text: String,
    /// List of attachments containing error details
    pub attachments: Vec<SlackAttachment>,
    /// List of activity sections
    pub sections: Vec<ActivitySection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Slack Attachment", description = "Attachment containing error details")]
pub struct SlackAttachment {
    /// Color indicator for the attachment
    pub color: String,
    /// List of fields with key-value information
    pub fields: Vec<AttachmentField>,
    /// Optional image URL
    pub image_url: Option<String>,
    /// Markdown formatting options
    pub mrkdown_in: Option<Vec<String>>,
    /// Optional attachment text
    pub text: Option<String>,
    /// Attachment title
    pub title: String,
    /// Link to the issue/error details
    pub title_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Attachment Field", description = "Key-value field within an attachment")]
pub struct AttachmentField {
    /// Field title
    pub title: String,
    /// Field value
    pub value: String,
    /// Whether to display the field in compact format
    pub short: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Activity Section", description = "Activity information section")]
pub struct ActivitySection {
    /// Activity title
    #[serde(rename = "activityTitle")]
    pub activity_title: String,
    /// Activity subtitle
    #[serde(rename = "activitySubtitle")]
    pub activity_subtitle: String,
}

// Feishu webhook structures
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Webhook", description = "Feishu message format")]
pub struct FeishuWebhook {
    /// Message type (text, post, interactive)
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Message content
    pub content: Option<FeishuContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Interactive card content
    pub card: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Content", description = "Feishu message content")]
pub struct FeishuContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Text content
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rich text content
    pub post: Option<FeishuPost>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Post", description = "Feishu rich text post")]
pub struct FeishuPost {
    /// Chinese content
    pub zh_cn: FeishuPostContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Post Content", description = "Feishu post content")]
pub struct FeishuPostContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Post title
    pub title: Option<String>,
    /// Post content elements
    pub content: Vec<Vec<FeishuPostElement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Post Element", description = "Feishu post content element")]
#[serde(tag = "tag")]
pub enum FeishuPostElement {
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String
    },
    #[serde(rename = "a")]
    Link {
        /// Link text
        text: String,
        /// Link URL
        href: String
    },
    #[serde(rename = "at")]
    At {
        /// User ID to mention
        #[serde(rename = "user_id")]
        user_id: String,
        /// User name (optional)
        #[serde(rename = "user_name")]
        user_name: Option<String>,
    },
}

// Response structures for API endpoints
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Config Response", description = "Configuration information response")]
pub struct ConfigResponse {
    /// Server port
    pub server_port: u16,
    /// List of configured webhooks
    pub feishu_webhooks: Vec<FeishuWebhookInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Webhook Info", description = "Feishu webhook configuration (sanitized)")]
pub struct FeishuWebhookInfo {
    /// Webhook name
    pub name: String,
    /// Webhook URL (masked for security)
    pub url: String,
    /// Whether webhook is enabled
    pub enabled: bool,
    /// Whether webhook has secret configured
    pub has_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Webhook Response", description = "Webhook processing response")]
pub struct WebhookResponse {
    /// Response status
    pub status: String,
    /// Response message
    pub message: String,
    /// Optional errors list
    pub errors: Option<Vec<String>>,
}

// ============================================================================
// Platform Types (v2.0 - YAML Configuration)
// ============================================================================

/// Feishu robot message configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Config", description = "Feishu robot message configuration")]
pub struct FeishuConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<Button>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_mapping: Option<std::collections::HashMap<String, String>>,
}

/// WeChat Work (Enterprise WeChat) configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Wecom Config", description = "WeChat Work webhook configuration")]
pub struct WecomConfig {
    pub corp_id: String,
    pub corp_secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_user: Option<String>,
}

/// DingTalk configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Dingtalk Config", description = "DingTalk webhook configuration")]
pub struct DingtalkConfig {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_mobiles: Option<Vec<String>>,
}

/// Custom button for interactive messages
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Button", description = "Custom button configuration")]
pub struct Button {
    pub text: String,
    pub url: String,
}

/// Forwarding configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Forward Config", description = "Message forwarding configuration")]
#[serde(tag = "type")]
pub enum ForwardConfig {
    #[serde(rename = "feishu_robot_msg")]
    FeishuRobotMsg(FeishuConfig),
    #[serde(rename = "wecom_webhook")]
    WecomWebhook(WecomConfig),
    #[serde(rename = "dingtalk_webhook")]
    DingtalkWebhook(DingtalkConfig),
}

/// Webhook runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Webhook Config", description = "Webhook runtime configuration")]
pub struct WebhookRuntimeConfig {
    /// Number of parallel requests (<1 = sequential, 1 = serial, >1 = max concurrent)
    #[serde(default = "default_parallel", skip_serializing_if = "is_default_parallel")]
    pub n_par: i32,
    #[serde(default = "default_timeout", skip_serializing_if = "is_default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_retry", skip_serializing_if = "is_default_retry")]
    pub retry: u32,
}

fn default_parallel() -> i32 { 1 }
fn is_default_parallel(v: &i32) -> bool { *v == 1 }
fn default_timeout() -> u64 { 30 }
fn is_default_timeout(v: &u64) -> bool { *v == 30 }
fn default_retry() -> u32 { 3 }
fn is_default_retry(v: &u32) -> bool { *v == 3 }

/// Main webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Webhook", description = "Webhook configuration")]
pub struct WebhookConfig {
    /// Webhook name (must be unique)
    pub name: String,
    /// List of webhook URLs (supports multiple endpoints)
    pub url: Vec<String>,
    /// Whether this webhook is enabled
    #[serde(default = "default_enabled", skip_serializing_if = "is_default_enabled")]
    pub enabled: bool,
    /// Forwarding configuration
    pub forward_config: ForwardConfig,
    /// Runtime configuration
    #[serde(default = "default_runtime_config", skip_serializing_if = "is_default_runtime_config")]
    pub config: WebhookRuntimeConfig,
}

fn default_enabled() -> bool { true }
fn is_default_enabled(v: &bool) -> bool { *v == true }
fn default_runtime_config() -> WebhookRuntimeConfig {
    WebhookRuntimeConfig {
        n_par: 1,
        timeout: 30,
        retry: 3,
    }
}
fn is_default_runtime_config(v: &WebhookRuntimeConfig) -> bool {
    v.n_par == 1 && v.timeout == 30 && v.retry == 3
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Config", description = "Application configuration")]
pub struct Config {
    /// Server port
    #[serde(default = "default_server_port", skip_serializing_if = "is_default_server_port")]
    pub server_port: u16,
    /// Server host address
    #[serde(default = "default_server_host", skip_serializing_if = "is_default_server_host")]
    pub server_host: String,
    /// Template directory path (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_dir: Option<String>,
    /// List of webhook configurations
    #[serde(default = "default_webhooks", skip_serializing_if = "is_default_webhooks")]
    pub webhooks: Vec<WebhookConfig>,
}

fn default_server_port() -> u16 { 7876 }
fn is_default_server_port(v: &u16) -> bool { *v == 7876 }
fn default_server_host() -> String { "127.0.0.1".to_string() }
fn is_default_server_host(v: &str) -> bool { v == "127.0.0.1" }
fn default_webhooks() -> Vec<WebhookConfig> { vec![] }
fn is_default_webhooks(v: &Vec<WebhookConfig>) -> bool { v.is_empty() }

/// Legacy Feishu Webhook Config (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Legacy Feishu Webhook Config", description = "Legacy Feishu webhook configuration")]
pub struct FeishuWebhookConfig {
    /// Webhook name
    pub name: String,
    /// Webhook URL
    pub url: String,
    /// Optional secret for signature verification
    pub secret: Option<String>,
    /// Whether webhook is enabled
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_port: 7876,
            server_host: "127.0.0.1".to_string(),
            template_dir: None,
            webhooks: vec![],
        }
    }
}