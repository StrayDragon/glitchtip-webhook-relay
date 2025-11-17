mod config;
mod converter;
mod service;
mod types;

use actix_web::{App, HttpResponse, HttpServer, web};
use std::env;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

// Import our modules
use config::LazyConfigManager;
use service::{receive_webhook, manage_config};
use types::{ConfigResponse, GlitchTipSlackWebhook, HealthResponse, WebhookResponse};

/// OpenAPI documentation for the GlitchTip to Feishu Webhook Relay
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
    ),
    components(
        schemas(
            GlitchTipSlackWebhook,
            HealthResponse,
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
        (name = "health", description = "Health check endpoints"),
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

    // Print help if requested
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
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

    // Start HTTP server
    HttpServer::new(move || {
        let config_manager_clone = Arc::clone(&config_manager);
        App::new()
            .app_data(web::Data::new(config_manager_clone))
            // OpenAPI documentation endpoint (Scalar UI)
            .service(Scalar::with_url("/dev/openapi-ui/scalar", ApiDoc::openapi()))
            // Webhook endpoint
            .route("/webhook/glitchtip", web::post().to(receive_webhook))
            // Unified config API endpoint (GET for view, POST for reload)
            .route("/internal/config", web::route().to(manage_config))
            // OpenAPI JSON endpoint
            .route("/dev/openapi.json", web::get().to(openapi_json))
            // Root endpoint with basic info
            .route("/", web::get().to(root_info))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn root_info() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "GlitchTip to Feishu Webhook Relay",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "webhook": "/webhook/glitchtip",
            "config": "/internal/config",
            "config_info": "GET /internal/config for view, POST /internal/config for reload",
            "docs": "/dev/openapi-ui/scalar",
            "openapi": "/dev/openapi.json"
        },
        "documentation": "/docs"
    }))
}

/// OpenAPI JSON endpoint
async fn openapi_json() -> HttpResponse {
    HttpResponse::Ok().json(ApiDoc::openapi())
}

fn print_help() {
    println!("GlitchTip to Feishu Webhook Relay");
    println!();
    println!("USAGE:");
    println!("    {} [OPTIONS]", env::args().next().unwrap_or_default());
    println!();
    println!("OPTIONS:");
    println!("    --example-config    Generate example configuration file");
    println!("    -h, --help          Show this help message");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    PORT                    Server port (default: 8080)");
    println!("    FEISHU_WEBHOOK_URL      Feishu webhook URL");
    println!("    FEISHU_WEBHOOK_SECRET   Optional webhook secret");
    println!();
    println!("CONFIGURATION:");
    println!("    Create config.toml or config.json in the current directory");
    println!("    Run --example-config to generate a template");
}
