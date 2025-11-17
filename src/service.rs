use actix_web::{web, HttpResponse, Result};
use reqwest::Client;
use utoipa;
use crate::types::*;
use crate::converter::Converter;

pub struct WebhookService {
    client: Client,
    config: web::Data<Config>,
}

impl WebhookService {
    pub fn new(config: web::Data<Config>) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn receive_glitchtip_webhook(
        webhook: web::Json<GlitchTipSlackWebhook>,
        config: web::Data<Config>,
    ) -> Result<HttpResponse> {
        log::info!("Received GlitchTip webhook: {}", webhook.alias);

        let service = Self::new(config);
        let errors = service.forward_to_feishu(&webhook).await;

        if errors.is_empty() {
            Ok(HttpResponse::Ok().json(WebhookResponse {
                status: "success".to_string(),
                message: "Webhook forwarded successfully".to_string(),
                errors: None,
            }))
        } else {
            log::error!("Some webhooks failed: {:?}", errors);
            Ok(HttpResponse::InternalServerError().json(WebhookResponse {
                status: "partial_success".to_string(),
                message: "Some webhooks failed".to_string(),
                errors: Some(errors),
            }))
        }
    }

    async fn forward_to_feishu(&self, glitchtip: &GlitchTipSlackWebhook) -> Vec<String> {
        let mut errors = Vec::new();

        for webhook_config in &self.config.feishu_webhooks {
            if !webhook_config.enabled {
                continue;
            }

            // Convert to different Feishu formats
            let feishu_messages = vec![
                Converter::glitchtip_to_feishu_text(glitchtip),
                Converter::glitchtip_to_feishu_rich_text(glitchtip),
                Converter::glitchtip_to_feishu_card(glitchtip),
            ];

            // Try each format (text, rich text, card)
            for (i, feishu_msg) in feishu_messages.iter().enumerate() {
                match self.send_to_feishu(&webhook_config.url, feishu_msg).await {
                    Ok(_) => {
                        log::info!("Successfully sent to {} using format {}",
                                 webhook_config.name,
                                 ["text", "rich_text", "card"][i]);
                        break; // Success, no need to try other formats
                    }
                    Err(e) => {
                        log::warn!("Failed to send to {} using format {}: {}",
                                  webhook_config.name,
                                  ["text", "rich_text", "card"][i],
                                  e);
                        if i == feishu_messages.len() - 1 {
                            // All formats failed
                            errors.push(format!("{}: {}", webhook_config.name, e));
                        }
                    }
                }
            }
        }

        errors
    }

    async fn send_to_feishu(&self, url: &str, message: &FeishuWebhook) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .post(url)
            .header("Content-Type", "application/json")
            .json(message)
            .send()
            .await?;

        if response.status().is_success() {
            log::debug!("Successfully sent webhook to {}", url);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(format!("HTTP {}: {}", status, body).into())
        }
    }
}

/// Receive GlitchTip webhook
///
/// Receives a GlitchTip webhook in Slack format and forwards it to configured Feishu webhooks.
#[utoipa::path(
    post,
    path = "/webhook/glitchtip",
    tag = "webhook",
    request_body = GlitchTipSlackWebhook,
    responses(
        (status = 200, description = "Webhook processed successfully", body = WebhookResponse),
        (status = 500, description = "Internal server error", body = WebhookResponse)
    )
)]
pub async fn receive_webhook(
    webhook: web::Json<GlitchTipSlackWebhook>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    WebhookService::receive_glitchtip_webhook(webhook, config).await
}

/// Health check endpoint
///
/// Returns the current health status and timestamp of the service.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Get configuration information
///
/// Returns current configuration information with sensitive data masked.
#[utoipa::path(
    get,
    path = "/config",
    tag = "config",
    responses(
        (status = 200, description = "Configuration information", body = ConfigResponse)
    )
)]
pub async fn get_config(config: web::Data<Config>) -> Result<HttpResponse> {
    // Return sanitized config (without secrets)
    let webhooks: Vec<FeishuWebhookInfo> = config.feishu_webhooks.iter().map(|w| {
        FeishuWebhookInfo {
            name: w.name.clone(),
            url: if w.url.is_empty() { "".to_string() } else { "***".to_string() },
            enabled: w.enabled,
            has_secret: w.secret.is_some(),
        }
    }).collect();

    Ok(HttpResponse::Ok().json(ConfigResponse {
        server_port: config.server_port,
        feishu_webhooks: webhooks,
    }))
}