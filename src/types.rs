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

// Configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Config", description = "Application configuration")]
pub struct Config {
    /// Server port
    pub server_port: u16,
    /// Template directory path (optional)
    pub template_dir: Option<String>,
    /// List of Feishu webhook configurations
    pub feishu_webhooks: Vec<FeishuWebhookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "Feishu Webhook Config", description = "Feishu webhook configuration")]
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
            server_port: 8080,
            template_dir: None,
            feishu_webhooks: vec![],
        }
    }
}