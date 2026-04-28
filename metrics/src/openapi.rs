//! OpenAPI/Swagger documentation for the metrics exporter API.

use utoipa::OpenApi;

/// OpenAPI schema for the router-metrics-exporter API.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::server::metrics_handler,
        crate::server::health_handler,
        crate::server::ready_handler,
    ),
    components(
        schemas()
    ),
    info(
        title = "Router Metrics Exporter API",
        description = "Prometheus metrics exporter for the stellar-router suite",
        version = "0.1.0",
        contact(
            name = "Stellar Router",
            url = "https://github.com/Maki-Zeninn/stellar-router"
        ),
        license(
            name = "MIT",
            url = "https://github.com/Maki-Zeninn/stellar-router/blob/main/LICENSE"
        )
    ),
    servers(
        (url = "http://localhost:9090", description = "Local development server"),
        (url = "http://localhost:3000", description = "Docker Compose default"),
    ),
    tags(
        (name = "metrics", description = "Prometheus metrics endpoints"),
        (name = "health", description = "Health check endpoints"),
    )
)]
pub struct ApiDoc;
