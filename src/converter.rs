use crate::types::*;
use std::path::Path;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use minijinja::{Environment, context};

/// Embedded default template (feishu format)
const EMBEDDED_TEMPLATE: Option<&'static [u8]> = Some(include_bytes!("../templates/feishu/default.json.jinja2"));

/// Color pairs for background and text (Feishu supported colors)
/// Format: (background_style, text_color)
const COLOR_PAIRS: &[(&str, &str)] = &[
    ("red-50", "red"),
    ("orange-50", "orange"),
    ("yellow-50", "yellow"),
    ("green-50", "green"),
    ("blue-50", "blue"),
    ("purple-50", "purple"),
    ("carmine-50", "carmine"),
    ("violet-50", "violet"),
    ("indigo-50", "indigo"),
    ("wathet-50", "wathet"),
    ("turquoise-50", "turquoise"),
    ("lime-50", "lime"),
];

/// Generate color pair based on string hash
fn hash_to_color(text: &str) -> (String, String) {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    let index = (hash as usize) % COLOR_PAIRS.len();
    let (bg, fg) = COLOR_PAIRS[index];

    (bg.to_string(), fg.to_string())
}

/// Extracted metadata structure for template rendering
#[derive(Debug, Clone)]
pub struct WebhookMetadata {
    pub webhook_alias: String,
    pub issue_identifier: String,
    pub exception_class_name: String,
    pub full_error_message: String,
    pub issue_url: String,
    pub project_id: String,
    pub environment_name: String,
    pub hostname: String,
    pub commit_hash: String,
    pub current_timestamp: String,
    pub colors: ColorInfo,
    pub element_ids: ElementIds,
}

#[derive(Debug, Clone)]
pub struct ColorInfo {
    pub project_bg: String,
    pub project_fg: String,
    pub env_bg: String,
    pub env_fg: String,
    pub host_bg: String,
    pub host_fg: String,
}

#[derive(Debug, Clone)]
pub struct ElementIds {
    pub id_1: String,
    pub id_2: String,
    pub id_3: String,
}

pub struct Converter {
    template_dir: Option<String>,
    enable_hash_colors: bool,
    env: Environment<'static>,
}

impl Converter {
    pub fn new(template_dir: Option<&str>) -> Self {
        let enable_hash_colors = std::env::var("GWR_ENABLE_HASH_COLORS")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true); // 默认启用

        log::info!("Converter initialized with {} template and {} hash colors",
            if template_dir.is_some() { "custom" } else { "embedded" },
            if enable_hash_colors { "enabled" } else { "disabled" }
        );

        let mut env = Environment::new();
        env.set_debug(true);

        Self {
            template_dir: template_dir.map(|s| s.to_string()),
            enable_hash_colors,
            env,
        }
    }

    /// Extract refined metadata from GlitchTip webhook
    pub fn extract_metadata(&self, glitchtip: &GlitchTipSlackWebhook) -> WebhookMetadata {
        // Extract issue identifier
        let issue_identifier = glitchtip.sections
            .first()
            .and_then(|section| {
                let subtitle = &section.activity_subtitle;
                // Remove "View Issue " prefix first
                let cleaned = subtitle.strip_prefix("View Issue ").unwrap_or(subtitle);

                // Extract text from markdown link format [text](url)
                if let Some(start) = cleaned.find('[') {
                    if let Some(end) = cleaned.find(']') {
                        if end > start {
                            return Some(&cleaned[start + 1..end]);
                        }
                    }
                }

                // Fallback: if no markdown format, use cleaned text as-is
                Some(cleaned)
            })
            .unwrap_or("Unknown")
            .to_string();

        let webhook_alias = "GlitchTip".to_string();

        // Extract exception and error details
        let (exception_class_name, full_error_message, issue_url) = if let Some(attachment) = glitchtip.attachments.first() {
            let exception_class = attachment.title.split(':').next().unwrap_or(&attachment.title).to_string();
            let full_error = attachment.title.clone();
            let url = attachment.title_link.clone();
            (exception_class, full_error, url)
        } else {
            ("Unknown".to_string(), "Unknown error".to_string(), "".to_string())
        };

        // Extract project information
        let (project_id, environment_name, hostname, commit_hash) = if let Some(attachment) = glitchtip.attachments.first() {
            let mut project = "Unknown".to_string();
            let mut env = "Unknown".to_string();
            let mut host = "Unknown".to_string();
            let mut commit = "Unknown".to_string();

            for field in &attachment.fields {
                match field.title.as_str() {
                    "Project" => project = field.value.clone(),
                    "Environment" => env = field.value.clone(),
                    "Server Name" => host = field.value.clone(),
                    "Release" => commit = field.value.clone(),
                    _ => {}
                }
            }
            (project, env, host, commit)
        } else {
            ("Unknown".to_string(), "Unknown".to_string(), "Unknown".to_string(), "Unknown".to_string())
        };

        log::info!("Extracted metadata: project={}, environment={}, hostname={}, commit_hash={}",
                 project_id, environment_name, hostname, commit_hash);

        // Generate colors based on configuration
        let colors = if self.enable_hash_colors {
            ColorInfo {
                project_bg: hash_to_color(&project_id).0,
                project_fg: hash_to_color(&project_id).1,
                env_bg: hash_to_color(&environment_name).0,
                env_fg: hash_to_color(&environment_name).1,
                host_bg: hash_to_color(&hostname).0,
                host_fg: hash_to_color(&hostname).1,
            }
        } else {
            ColorInfo {
                project_bg: "red-50".to_string(),
                project_fg: "red".to_string(),
                env_bg: "carmine-50".to_string(),
                env_fg: "carmine".to_string(),
                host_bg: "orange-50".to_string(),
                host_fg: "orange".to_string(),
            }
        };

        // Add current timestamp
        let current_timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Add random element IDs for template uniqueness
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let element_ids = ElementIds {
            id_1: format!("{:x}", rng.r#gen::<u64>()),
            id_2: format!("{:x}", rng.r#gen::<u64>()),
            id_3: format!("{:x}", rng.r#gen::<u64>()),
        };

        WebhookMetadata {
            webhook_alias,
            issue_identifier,
            exception_class_name,
            full_error_message,
            issue_url,
            project_id,
            environment_name,
            hostname,
            commit_hash,
            current_timestamp,
            colors,
            element_ids,
        }
    }

    /// Load template content - priority: external > embedded
    fn load_template_content(&self) -> Option<String> {
        // Try external template first if configured
        if let Some(dir) = &self.template_dir {
            let template_path = Path::new(dir).join("feishu/default.json.jinja2");
            if template_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&template_path) {
                    log::info!("Loaded external template from: {}", template_path.display());
                    return Some(content);
                } else {
                    log::warn!("Failed to read external template file");
                }
            } else {
                log::warn!("External template not found at: {}, falling back to embedded", template_path.display());
            }
        }

        EMBEDDED_TEMPLATE.map(|bytes| {
            String::from_utf8_lossy(bytes).to_string()
        })
    }

    /// Render template using extracted metadata
    fn render_template(&self, template_content: &str, metadata: &WebhookMetadata) -> Result<String, minijinja::Error> {
        let template = self.env.template_from_str(template_content)?;

        let context = context! {
            webhook_alias => &metadata.webhook_alias,
            issue_identifier => &metadata.issue_identifier,
            exception_class_name => &metadata.exception_class_name,
            full_error_message => &metadata.full_error_message,
            issue_url => &metadata.issue_url,
            project_id => &metadata.project_id,
            environment_name => &metadata.environment_name,
            hostname => &metadata.hostname,
            commit_hash => &metadata.commit_hash,
            current_timestamp => &metadata.current_timestamp,

            // Color variables
            project_bg_color => &metadata.colors.project_bg,
            project_fg_color => &metadata.colors.project_fg,
            env_bg_color => &metadata.colors.env_bg,
            env_fg_color => &metadata.colors.env_fg,
            host_bg_color => &metadata.colors.host_bg,
            host_fg_color => &metadata.colors.host_fg,

            // Element IDs
            element_id_1 => &metadata.element_ids.id_1,
            element_id_2 => &metadata.element_ids.id_2,
            element_id_3 => &metadata.element_ids.id_3,
        };

        template.render(context)
    }

    /// Convert GlitchTip webhook to Feishu interactive card format using template system
    pub fn glitchtip_to_feishu_card(&self, glitchtip: &GlitchTipSlackWebhook) -> Option<FeishuWebhook> {
        // Step 1: Extract refined metadata
        let metadata = self.extract_metadata(glitchtip);
        log::info!("Extracted metadata for issue: {}", metadata.issue_identifier);
        log::debug!("Issue URL: '{}'", metadata.issue_url);

        // Step 2: Load template
        let template_content = self.load_template_content()?;
        log::info!("Loaded template content (length: {})", template_content.len());

        // Step 3: Render template with metadata
        let rendered_json = match self.render_template(&template_content, &metadata) {
            Ok(rendered) => {
                log::info!("Template rendered successfully");
                log::debug!("Rendered JSON: {}", rendered);
                rendered
            }
            Err(e) => {
                log::error!("Template rendering failed: {}", e);
                // Fallback to direct construction
                return Some(self.construct_fallback_feishu_webhook(&metadata));
            }
        };

        // Step 4: Parse rendered JSON
        let card_json: serde_json::Value = match serde_json::from_str::<serde_json::Value>(&rendered_json) {
            Ok(json) => {
                log::info!("Parsed rendered JSON successfully");
                json
            }
            Err(e) => {
                log::error!("Failed to parse rendered JSON: {}", e);
                log::error!("Rendered content: {}", rendered_json);
                // Fallback to direct construction
                return Some(self.construct_fallback_feishu_webhook(&metadata));
            }
        };

        // Step 5: Extract the card DSL from the template structure
        let card_content = if let Some(dsl) = card_json.get("dsl") {
            dsl.clone()
        } else {
            card_json.clone()
        };

        log::info!("Constructed Feishu webhook successfully using template");
        Some(FeishuWebhook {
            msg_type: "interactive".to_string(),
            card: Some(card_content),
            content: None,
        })
    }

    /// Fallback using embedded template when external template rendering fails
    fn construct_fallback_feishu_webhook(&self, metadata: &WebhookMetadata) -> FeishuWebhook {
        log::warn!("Using fallback embedded template for Feishu webhook");

        // Use embedded template content
        let embedded_template = EMBEDDED_TEMPLATE
            .and_then(|bytes| String::from_utf8_lossy(bytes).to_string().into())
            .expect("Embedded template should be available");

        match self.render_template(&embedded_template, metadata) {
            Ok(rendered_json) => {
                match serde_json::from_str::<serde_json::Value>(&rendered_json) {
                    Ok(card_json) => {
                        let card_content = if let Some(dsl) = card_json.get("dsl") {
                            dsl.clone()
                        } else {
                            card_json
                        };

                        FeishuWebhook {
                            msg_type: "interactive".to_string(),
                            card: Some(card_content),
                            content: None,
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse embedded template JSON: {}", e);
                        self.create_emergency_fallback_webhook(metadata)
                    }
                }
            }
            Err(e) => {
                log::error!("Embedded template rendering failed: {}", e);
                self.create_emergency_fallback_webhook(metadata)
            }
        }
    }

    /// Emergency minimal fallback when even embedded template fails
    fn create_emergency_fallback_webhook(&self, metadata: &WebhookMetadata) -> FeishuWebhook {
        log::error!("Using emergency minimal fallback for Feishu webhook");

        let card_json = serde_json::json!({
            "schema": "2.0",
            "body": {
                "elements": [
                    {
                        "tag": "div",
                        "text": {
                            "tag": "plain_text",
                            "content": format!("Error: {} - {}", metadata.exception_class_name, metadata.full_error_message)
                        }
                    }
                ]
            },
            "header": {
                "title": {
                    "tag": "plain_text",
                    "content": format!("GlitchTip Alert - {}", metadata.issue_identifier)
                },
                "template": "red"
            }
        });

        FeishuWebhook {
            msg_type: "interactive".to_string(),
            card: Some(card_json),
            content: None,
        }
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new(None)
    }
}
