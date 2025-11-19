use crate::types::*;
use minijinja::{Environment, context, Value};
use std::collections::HashMap;
use std::path::Path;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Embedded default template (auto-embedded with include_bytes!)
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

pub struct Converter {
    template_dir: Option<String>,
    enable_hash_colors: bool,
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

        Self {
            template_dir: template_dir.map(|s| s.to_string()),
            enable_hash_colors,
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

    /// Extract metadata from GlitchTip webhook
    fn extract_metadata(&self, glitchtip: &GlitchTipSlackWebhook) -> Value {
        let mut metadata = HashMap::new();

        metadata.insert("webhook_alias".to_string(), Value::from_serialize(&glitchtip.alias));

        // Extract issue identifier from sections[0].activitySubtitle (e.g., "View Issue SERVICE_WECOM-63")
        let issue_identifier = glitchtip.sections
            .first()
            .map(|section| section.activity_subtitle.strip_prefix("View Issue ").unwrap_or(&section.activity_subtitle))
            .unwrap_or("Unknown");
        metadata.insert("issue_identifier".to_string(), Value::from_serialize(&issue_identifier));

        if let Some(attachment) = glitchtip.attachments.first() {
            // Extract exception class name from title (e.g., "ProgrammingError: ...")
            let exception_class_name = attachment.title.split(':').next().unwrap_or(&attachment.title);
            metadata.insert("exception_class_name".to_string(), Value::from_serialize(&exception_class_name));
            metadata.insert("full_error_message".to_string(), Value::from_serialize(&attachment.title));
            metadata.insert("issue_url".to_string(), Value::from_serialize(&attachment.title_link));

            // Extract fields with descriptive names
            let mut project_id = "Unknown".to_string();
            let mut environment_name = "Unknown".to_string();
            let mut hostname = "Unknown".to_string();
            let mut commit_hash = "Unknown".to_string();

            for field in &attachment.fields {
                match field.title.to_lowercase().as_str() {
                    "project" => project_id = field.value.clone(),
                    "environment" => environment_name = field.value.clone(),
                    "server name" => hostname = field.value.clone(),
                    "release" => commit_hash = field.value.clone(),
                    _ => {}
                }
            }

            metadata.insert("project_id".to_string(), Value::from_serialize(&project_id));
            metadata.insert("environment_name".to_string(), Value::from_serialize(&environment_name));
            metadata.insert("hostname".to_string(), Value::from_serialize(&hostname));
            metadata.insert("commit_hash".to_string(), Value::from_serialize(&commit_hash));

            // Generate colors based on configuration
            let (project_bg, project_fg, env_bg, env_fg, host_bg, host_fg) = if self.enable_hash_colors {
                // Generate dynamic colors based on field values
                let (project_bg, project_fg) = hash_to_color(&project_id);
                let (env_bg, env_fg) = hash_to_color(&environment_name);
                let (host_bg, host_fg) = hash_to_color(&hostname);
                (project_bg, project_fg, env_bg, env_fg, host_bg, host_fg)
            } else {
                // Use fixed colors
                ("red-50".to_string(), "red".to_string(),
                 "carmine-50".to_string(), "carmine".to_string(),
                 "orange-50".to_string(), "orange".to_string())
            };

            metadata.insert("project_bg_color".to_string(), Value::from_serialize(&project_bg));
            metadata.insert("project_fg_color".to_string(), Value::from_serialize(&project_fg));
            metadata.insert("env_bg_color".to_string(), Value::from_serialize(&env_bg));
            metadata.insert("env_fg_color".to_string(), Value::from_serialize(&env_fg));
            metadata.insert("host_bg_color".to_string(), Value::from_serialize(&host_bg));
            metadata.insert("host_fg_color".to_string(), Value::from_serialize(&host_fg));
        } else {
            metadata.insert("exception_class_name".to_string(), Value::from_serialize("Unknown"));
            metadata.insert("full_error_message".to_string(), Value::from_serialize("No error details"));
            metadata.insert("issue_url".to_string(), Value::from_serialize(""));
            metadata.insert("project_id".to_string(), Value::from_serialize("Unknown"));
            metadata.insert("environment_name".to_string(), Value::from_serialize("Unknown"));
            metadata.insert("hostname".to_string(), Value::from_serialize("Unknown"));
            metadata.insert("commit_hash".to_string(), Value::from_serialize("Unknown"));

            // Default colors for unknown values
            let default_colors = if self.enable_hash_colors {
                ("grey-50".to_string(), "grey".to_string(),
                 "grey-50".to_string(), "grey".to_string(),
                 "grey-50".to_string(), "grey".to_string())
            } else {
                ("red-50".to_string(), "red".to_string(),
                 "carmine-50".to_string(), "carmine".to_string(),
                 "orange-50".to_string(), "orange".to_string())
            };

            metadata.insert("project_bg_color".to_string(), Value::from_serialize(&default_colors.0));
            metadata.insert("project_fg_color".to_string(), Value::from_serialize(&default_colors.1));
            metadata.insert("env_bg_color".to_string(), Value::from_serialize(&default_colors.2));
            metadata.insert("env_fg_color".to_string(), Value::from_serialize(&default_colors.3));
            metadata.insert("host_bg_color".to_string(), Value::from_serialize(&default_colors.4));
            metadata.insert("host_fg_color".to_string(), Value::from_serialize(&default_colors.5));
        }

        // Add current timestamp
        let current_timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        metadata.insert("current_timestamp".to_string(), Value::from_serialize(&current_timestamp));

        // Add random element IDs for template uniqueness
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let element_id_1 = format!("{:x}", rng.r#gen::<u64>());
        let element_id_2 = format!("{:x}", rng.r#gen::<u64>());
        let element_id_3 = format!("{:x}", rng.r#gen::<u64>());
        metadata.insert("element_id_1".to_string(), Value::from_serialize(&element_id_1));
        metadata.insert("element_id_2".to_string(), Value::from_serialize(&element_id_2));
        metadata.insert("element_id_3".to_string(), Value::from_serialize(&element_id_3));

        Value::from_serialize(&metadata)
    }

    /// Convert GlitchTip webhook to Feishu interactive card format
    pub fn glitchtip_to_feishu_card(&self, glitchtip: &GlitchTipSlackWebhook) -> Option<FeishuWebhook> {
        // Load template content dynamically
        if let Some(template_content) = self.load_template_content() {
            // Create a temporary environment for this template
            let mut temp_env = Environment::new();
            if temp_env.add_template("feishu_card", &template_content).is_ok() {
                if let Ok(template) = temp_env.get_template("feishu_card") {
                    let metadata = self.extract_metadata(glitchtip);
                    if let Ok(rendered) = template.render(context!(metadata)) {
                        if let Ok(parsed) = serde_json::from_str(&rendered) {
                            return parsed;
                        }
                    }
                }
            }
        }
        None
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new(None)
    }
}
