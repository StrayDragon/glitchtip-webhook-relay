mod types;
mod converter;
mod service;
mod config;

use actix_web::{App, HttpResponse, HttpServer, web};
use std::env;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

// Import our modules
use service::{health_check, get_config, receive_webhook};
use config::ConfigManager;
use types::{GlitchTipSlackWebhook, HealthResponse, ConfigResponse, WebhookResponse};

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
        crate::service::health_check,
        crate::service::get_config,
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

    // Load configuration
    let mut config = ConfigManager::load()
        .expect("Failed to load configuration");

    // Apply environment variable overrides
    config.apply_env_overrides();

    log::info!("Starting GlitchTip to Feishu Webhook Relay");
    log::info!("Server will listen on port {}", config.server_port);
    log::info!("Configured Feishu webhooks: {}", config.feishu_webhooks.len());

    // Print example if requested
    if env::args().any(|arg| arg == "--example-config") {
        ConfigManager::save_example_config().unwrap();
        return Ok(());
    }

    // Print help if requested
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let config_data = web::Data::new(config);
    let port = config_data.server_port;

    // Start HTTP server
    HttpServer::new(move || {
        let config_clone = config_data.clone();
        App::new()
            .app_data(config_clone)
            // OpenAPI documentation endpoint (Scalar UI)
            .service(Scalar::with_url("/docs", ApiDoc::openapi()))
            // Webhook endpoint
            .route("/webhook/glitchtip", web::post().to(receive_webhook))
            // Health check
            .route("/health", web::get().to(health_check))
            // Configuration info
            .route("/config", web::get().to(get_config))
            // OpenAPI JSON endpoint
            .route("/api-docs/openapi.json", web::get().to(openapi_json))
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
            "health": "/health",
            "config": "/config",
            "docs": "/docs",
            "openapi": "/api-docs/openapi.json"
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
