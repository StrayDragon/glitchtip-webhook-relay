mod config;
mod converter;
mod routes;
mod service;
mod types;

use actix_web::{App, HttpResponse, HttpServer, web};
use std::env;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use config::LazyConfigManager;
use routes::Routes;
use service::{manage_config, receive_webhook};
use types::{ConfigResponse, GlitchTipSlackWebhook, WebhookResponse};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "GlitchTip to Feishu Webhook Relay",
        description = "A service that converts GlitchTip webhooks to Feishu message format",
        version = "0.1.0",
        contact(
            name = "GitHub Repository",
            url = "https://github.com/yourusername/glitchtip-webhook-relay"
        )
    ),
    paths(
        crate::service::receive_webhook,
        crate::service::manage_config,
    ),
    components(
        schemas(
            GlitchTipSlackWebhook,
            ConfigResponse,
            WebhookResponse,
            types::SlackAttachment,
            types::AttachmentField,
            types::ActivitySection,
            types::FeishuWebhook,
            types::FeishuWebhookConfig,
            types::FeishuWebhookInfo,
        )
    ),
    tags(
        (name = "webhook", description = "Webhook processing endpoints"),
        (name = "config", description = "Configuration endpoints")
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init();

    // Create lazy config manager
    let config_manager = Arc::new(LazyConfigManager::new());

    // Print example if requested
    if env::args().any(|arg| arg == "--example-config") {
        config::ConfigManager::save_example_config().unwrap();
        return Ok(());
    }

    // Load config initially to get the port (this will be lazy-loaded)
    let config = match config_manager.get_config() {
        Ok(config) => config,
        Err(e) => {
            log::warn!("Failed to load configuration: {}, using defaults", e);
            crate::types::Config::default()
        }
    };

    log::info!("Starting GlitchTip to Feishu Webhook Relay with lazy loading");
    log::info!("Server will listen on port {}", config.server_port);
    log::info!(
        "Configured Feishu webhooks: {}",
        config.feishu_webhooks.len()
    );

    let port = config.server_port;

    HttpServer::new(move || {
        let config_manager_clone = Arc::clone(&config_manager);
        App::new()
            .app_data(web::Data::new(config_manager_clone))
            .service(Scalar::with_url(
                Routes::DEV_OPENAPI_UI,
                ApiDoc::openapi(),
            ))
            .route(Routes::WEBHOOK_GLITCHTIP, web::post().to(receive_webhook))
            .route(Routes::INTERNAL_CONFIG_RELOAD, web::get().to(manage_config))
            .route(Routes::DEV_OPENAPI_JSON, web::get().to(openapi_json))
            .route(Routes::ROOT, web::get().to(root_info))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn root_info() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn openapi_json() -> HttpResponse {
    HttpResponse::Ok().json(ApiDoc::openapi())
}
