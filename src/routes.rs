/// API route definitions
///
/// This struct centralizes all API route paths using associated constants,
/// providing a clean namespace and avoiding duplication between utoipa
/// annotations and actix-web route definitions.
///
/// # Example
/// ```
/// use crate::routes::Routes;
///
/// let path = Routes::WEBHOOK_GLITCHTIP;
/// ```

pub struct Routes;

impl Routes {
    /// Webhook endpoint for receiving GlitchTip webhooks
    pub const WEBHOOK_GLITCHTIP: &'static str = "/webhook/glitchtip";

    /// Internal configuration reload endpoint
    pub const INTERNAL_CONFIG_RELOAD: &'static str = "/internal/config/reload";

    /// OpenAPI UI (Scalar) endpoint
    pub const DEV_OPENAPI_UI: &'static str = "/dev/openapi-ui/scalar";

    /// OpenAPI JSON specification endpoint
    pub const DEV_OPENAPI_JSON: &'static str = "/dev/openapi.json";

    /// Root endpoint
    pub const ROOT: &'static str = "/";
}

