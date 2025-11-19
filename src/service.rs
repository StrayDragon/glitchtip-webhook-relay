use crate::config::LazyConfigManager;
use crate::converter::Converter;
use crate::routes::Routes;
use crate::types::*;
use actix_web::{HttpResponse, Result, web};
use std::sync::Arc;
use utoipa;

pub struct WebhookService {
    config_manager: Arc<LazyConfigManager>,
}

impl WebhookService {
    pub fn new(config_manager: Arc<LazyConfigManager>) -> Self {
        Self { config_manager }
    }

    /// Forward webhook to specific endpoint
    pub async fn forward_to_endpoint(
        &self,
        glitchtip: &GlitchTipSlackWebhook,
        endpoint_id: &str,
        config: &Config,
    ) -> Result<Vec<String>, String> {
        log::info!("Forwarding webhook to endpoint: {}", endpoint_id);

        // Look up webhook configuration by name
        let webhook_config = config
            .webhooks
            .iter()
            .find(|w| w.name == endpoint_id)
            .ok_or_else(|| format!("Webhook '{}' not found in configuration", endpoint_id))?;

        // Check if webhook is enabled
        if !webhook_config.enabled {
            log::info!("Webhook '{}' is disabled, skipping", endpoint_id);
            return Ok(vec![]);
        }

        // Check if webhook has URLs
        if webhook_config.url.is_empty() {
            return Err(format!("Webhook '{}' has no URLs configured", endpoint_id));
        }

        // Forward based on n_par setting
        let errors = self
            .forward_with_parallelism(glitchtip, webhook_config)
            .await;

        Ok(errors)
    }

    /// Forward with parallelism control
    async fn forward_with_parallelism(
        &self,
        glitchtip: &GlitchTipSlackWebhook,
        webhook_config: &WebhookConfig,
    ) -> Vec<String> {
        let urls = &webhook_config.url;
        let n_par = webhook_config.config.n_par;

        // Single URL or sequential mode
        if urls.len() == 1 || n_par <= 1 {
            return self
                .forward_sequential(glitchtip, urls, webhook_config)
                .await;
        }

        // Parallel mode with limit
        let limit = n_par as usize;
        self.forward_parallel(glitchtip, urls, webhook_config, limit)
            .await
    }

    /// Sequential forwarding
    async fn forward_sequential(
        &self,
        glitchtip: &GlitchTipSlackWebhook,
        urls: &[String],
        webhook_config: &WebhookConfig,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let converter = Converter::new(None);

        for url in urls {
            match self
                .send_by_platform(glitchtip, url, webhook_config, &converter)
                .await
            {
                Ok(_) => {
                    log::info!("Successfully sent to {} ({})", webhook_config.name, url);
                }
                Err(e) => {
                    log::error!("Failed to send to {} ({}): {}", webhook_config.name, url, e);
                    errors.push(format!("{} ({}): {}", webhook_config.name, url, e));
                }
            }
        }

        errors
    }

    /// Parallel forwarding with limit
    async fn forward_parallel(
        &self,
        glitchtip: &GlitchTipSlackWebhook,
        urls: &[String],
        webhook_config: &WebhookConfig,
        max_concurrent: usize,
    ) -> Vec<String> {
        use futures::stream::{self, StreamExt};

        let converter = Converter::new(None);

        stream::iter(urls)
            .map(|url| {
                let webhook_config = webhook_config.clone();
                let converter_ref = &converter;
                async move {
                    match Self::send_by_platform_impl(
                        glitchtip,
                        url,
                        &webhook_config,
                        converter_ref,
                    )
                    .await
                    {
                        Ok(_) => {
                            log::info!("Successfully sent to {} ({})", webhook_config.name, url);
                            None
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to send to {} ({}): {}",
                                webhook_config.name,
                                url,
                                e
                            );
                            Some(format!("{} ({}): {}", webhook_config.name, url, e))
                        }
                    }
                }
            })
            .buffer_unordered(max_concurrent)
            .filter_map(|result| async move { result })
            .collect()
            .await
    }

    /// Send message based on platform type
    async fn send_by_platform(
        &self,
        glitchtip: &GlitchTipSlackWebhook,
        url: &str,
        webhook_config: &WebhookConfig,
        converter: &Converter,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Self::send_by_platform_impl(glitchtip, url, webhook_config, converter).await
    }

    async fn send_by_platform_impl(
        glitchtip: &GlitchTipSlackWebhook,
        url: &str,
        webhook_config: &WebhookConfig,
        converter: &Converter,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &webhook_config.forward_config {
            ForwardConfig::FeishuRobotMsg(_) => {
                // Convert to Feishu interactive card format
                let feishu_msg =
                    converter
                        .glitchtip_to_feishu_card(glitchtip)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Failed to convert to Feishu interactive card format")
                        })?;
                send_http_request(url, &feishu_msg).await
            }
            ForwardConfig::WecomWebhook(_) => {
                // TODO: Implement WeCom webhook support
                Err("WeCom webhook not implemented yet".into())
            }
            ForwardConfig::DingtalkWebhook(_) => {
                // TODO: Implement DingTalk webhook support
                Err("DingTalk webhook not implemented yet".into())
            }
        }
    }
}

/// Send HTTP request with proper error handling
async fn send_http_request(
    url: &str,
    message: &FeishuWebhook,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
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

/// Reload configuration from file
///
/// Reloads the configuration from the default config file location.
/// Returns 200 on success (no content), 400 with error message on failure.
#[utoipa::path(
    get,
    path = Routes::INTERNAL_CONFIG_RELOAD,
    tag = "Internal API",
    responses(
        (status = 200, description = "Configuration reloaded successfully"),
        (status = 400, description = "Failed to reload configuration")
    )
)]
pub async fn manage_config(
    config_manager: web::Data<Arc<LazyConfigManager>>,
) -> Result<HttpResponse> {
    log::info!("Received configuration reload request");

    match config_manager.force_reload() {
        Ok(_) => {
            log::info!("Configuration reloaded successfully");
            Ok(HttpResponse::Ok().finish())
        }
        Err(e) => {
            log::error!("Failed to reload configuration: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })))
        }
    }
}

/// Receive GlitchTip webhook for specific endpoint
///
/// Receives a GlitchTip webhook and forwards it to the configured endpoint's webhooks.
/// This endpoint supports dynamic routing via URL path parameter.
///
/// # Path Parameters
/// * `endpoint_id` - The endpoint identifier from configuration
///
/// # Example
/// ```http
/// POST /i/team_alerts
/// POST /i/project_a
/// POST /i/production
/// ```
#[utoipa::path(
    post,
    path = Routes::ENDPOINT_WEBHOOK,
    tag = "Endpoint API",
    request_body = GlitchTipSlackWebhook,
    responses(
        (status = 200, description = "Webhook processed successfully", body = WebhookResponse),
        (status = 404, description = "Endpoint not found", body = WebhookResponse),
        (status = 500, description = "Internal server error", body = WebhookResponse)
    )
)]
pub async fn receive_endpoint_webhook(
    webhook: web::Json<GlitchTipSlackWebhook>,
    path: web::Path<(String,)>,
    config_manager: web::Data<Arc<LazyConfigManager>>,
) -> Result<HttpResponse> {
    let endpoint_id = path.into_inner().0;
    log::info!("Received GlitchTip webhook for endpoint: {}", endpoint_id);

    let service = WebhookService::new(config_manager.get_ref().clone());
    let config = service.config_manager.get_config().map_err(|e| {
        log::error!("Failed to get configuration: {}", e);
        actix_web::error::ErrorInternalServerError("Configuration error")
    })?;

    match service
        .forward_to_endpoint(&webhook, &endpoint_id, &config)
        .await
    {
        Ok(errors) => {
            if errors.is_empty() {
                Ok(HttpResponse::Ok().json(WebhookResponse {
                    status: "success".to_string(),
                    message: format!(
                        "Webhook forwarded successfully to endpoint '{}'",
                        endpoint_id
                    ),
                    errors: None,
                }))
            } else {
                log::warn!(
                    "Some webhooks failed for endpoint '{}': {:?}",
                    endpoint_id,
                    errors
                );
                Ok(HttpResponse::Ok().json(WebhookResponse {
                    status: "partial_success".to_string(),
                    message: format!("Some webhooks failed for endpoint '{}'", endpoint_id),
                    errors: Some(errors),
                }))
            }
        }
        Err(e) => {
            log::error!("Failed to forward to endpoint '{}': {}", endpoint_id, e);
            Ok(HttpResponse::NotFound().json(WebhookResponse {
                status: "error".to_string(),
                message: e,
                errors: None,
            }))
        }
    }
}
