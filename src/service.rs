use actix_web::{web, HttpResponse, Result, HttpRequest};
use reqwest::Client;
use utoipa;
use crate::types::*;
use crate::converter::Converter;
use crate::config::LazyConfigManager;
use std::sync::Arc;

pub struct WebhookService {
    client: Client,
    config_manager: Arc<LazyConfigManager>,
}

impl WebhookService {
    pub fn new(config_manager: Arc<LazyConfigManager>) -> Self {
        Self {
            client: Client::new(),
            config_manager,
        }
    }

    pub async fn receive_glitchtip_webhook(
        webhook: web::Json<GlitchTipSlackWebhook>,
        config_manager: web::Data<Arc<LazyConfigManager>>,
    ) -> Result<HttpResponse> {
        log::info!("Received GlitchTip webhook: {}", webhook.alias);

        let service = Self::new(config_manager.get_ref().clone());
        let config = service.config_manager.get_config()
            .map_err(|e| {
                log::error!("Failed to get configuration: {}", e);
                actix_web::error::ErrorInternalServerError("Configuration error")
            })?;

        let errors = service.forward_to_feishu(&webhook, &config).await;

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

    async fn forward_to_feishu(&self, glitchtip: &GlitchTipSlackWebhook, config: &Config) -> Vec<String> {
        let mut errors = Vec::new();
        let converter = Converter::new(config.template_dir.as_deref());

        for webhook_config in &config.feishu_webhooks {
            if !webhook_config.enabled {
                continue;
            }

            // Convert to Feishu interactive card format
            let feishu_msg = converter.glitchtip_to_feishu_card(glitchtip);

            match self.send_to_feishu(&webhook_config.url, &feishu_msg).await {
                Ok(_) => {
                    log::info!("Successfully sent to {} using card format",
                             webhook_config.name);
                }
                Err(e) => {
                    log::error!("Failed to send to {}: {}", webhook_config.name, e);
                    errors.push(format!("{}: {}", webhook_config.name, e));
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
    config_manager: web::Data<Arc<LazyConfigManager>>,
) -> Result<HttpResponse> {
    WebhookService::receive_glitchtip_webhook(webhook, config_manager).await
}

/// Manage webhook configuration (reload or view)
///
/// This endpoint can both reload and view current configuration.
/// - POST: Force reload configuration from files (detects changes via MD5)
/// - GET: View current configuration status without reloading
#[utoipa::path(
    get,
    post,
    path = "/internal/config",
    tag = "config",
    responses(
        (status = 200, description = "Configuration viewed successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn manage_config(
    req: HttpRequest,
    config_manager: web::Data<Arc<LazyConfigManager>>,
) -> Result<HttpResponse> {
    match req.method().as_str() {
        "POST" => {
            log::info!("Received configuration reload request");

            match config_manager.force_reload() {
                Ok(config) => {
                    let metadata = config_manager.get_metadata().unwrap_or_default();
                    log::info!("Configuration reloaded successfully");
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "status": "success",
                        "message": "Configuration reloaded successfully",
                        "action": "reload",
                        "config": {
                            "server_port": config.server_port,
                            "webhook_count": config.feishu_webhooks.len(),
                            "enabled_webhooks": config.feishu_webhooks.iter()
                                .filter(|w| w.enabled)
                                .count(),
                            "config_file": metadata.0,
                            "md5_hash": metadata.1,
                            "webhooks": config.feishu_webhooks.iter()
                                .map(|w| serde_json::json!({
                                    "name": w.name,
                                    "enabled": w.enabled,
                                    "has_secret": w.secret.is_some(),
                                    "url_preview": if w.url.len() > 30 {
                                        format!("{}...{}", &w.url[..15], &w.url[w.url.len()-10..])
                                    } else {
                                        w.url.clone()
                                    }
                                }))
                                .collect::<Vec<_>>()
                        }
                    })))
                }
                Err(e) => {
                    log::error!("Failed to reload configuration: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": "Failed to reload configuration",
                        "action": "reload",
                        "error": e.to_string()
                    })))
                }
            }
        }
        "GET" => {
            log::debug!("Received configuration view request");

            match config_manager.get_metadata() {
                Ok((path, hash)) => {
                    match config_manager.get_config() {
                        Ok(config) => {
                            Ok(HttpResponse::Ok().json(serde_json::json!({
                                "status": "success",
                                "action": "view",
                                "config": {
                                    "server_port": config.server_port,
                                    "webhook_count": config.feishu_webhooks.len(),
                                    "enabled_webhooks": config.feishu_webhooks.iter()
                                        .filter(|w| w.enabled)
                                        .count(),
                                    "config_file": path,
                                    "md5_hash": hash,
                                    "webhooks": config.feishu_webhooks.iter()
                                        .map(|w| serde_json::json!({
                                            "name": w.name,
                                            "enabled": w.enabled,
                                            "has_secret": w.secret.is_some(),
                                            "url_preview": if w.url.len() > 30 {
                                                format!("{}...{}", &w.url[..15], &w.url[w.url.len()-10..])
                                            } else {
                                                w.url.clone()
                                            }
                                        }))
                                        .collect::<Vec<_>>()
                                }
                            })))
                        }
                        Err(e) => {
                            Ok(HttpResponse::Ok().json(serde_json::json!({
                                "status": "error",
                                "action": "view",
                                "config_file": path,
                                "md5_hash": hash,
                                "error": e.to_string()
                            })))
                        }
                    }
                }
                Err(e) => {
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "action": "view",
                        "error": e.to_string()
                    })))
                }
            }
        }
        _ => {
            Ok(HttpResponse::MethodNotAllowed().json(serde_json::json!({
                "status": "error",
                "message": "Method not allowed",
                "allowed_methods": ["GET", "POST"]
            })))
        }
    }
}

