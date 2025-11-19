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
/// let path = Routes::ENDPOINT_WEBHOOK;
/// ```

pub struct Routes;

impl Routes {
    /// Endpoint-based webhook routing (v2.0)
    /// Pattern: /i/{endpoint_id}
    pub const ENDPOINT_WEBHOOK: &'static str = "/i/{endpoint_id}";

    /// Internal configuration reload endpoint
    pub const INTERNAL_CONFIG_RELOAD: &'static str = "/internal/config/reload";

    /// OpenAPI UI (Scalar) endpoint
    pub const DEV_OPENAPI_UI: &'static str = "/dev/openapi-ui/scalar";

    /// OpenAPI JSON specification endpoint
    pub const DEV_OPENAPI_JSON: &'static str = "/dev/openapi.json";

    /// Root endpoint
    pub const ROOT: &'static str = "/";
}

