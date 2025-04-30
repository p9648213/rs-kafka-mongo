use axum::middleware;
use rs_kafka_mongo::{
    auth::{self},
    config::Config,
    message,
    products::{self},
    state::AppState,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(OpenApi)]
    #[openapi(
        info(description = "This project is a simple yet complete API built in Rust, demonstrating user authentication and full CRUD operations for products. It integrates MongoDB for persistent storage and uses Kafka to stream product-related events. The entire application is containerized with Docker for easy deployment, and Swagger UI is included to provide a clear and interactive interface for testing the API endpoints."),
        modifiers(&SecurityAddon),
        tags(
            (name = "product", description = "product api management"),
            (name = "message", description = "message api management"),
            (name = "user", description = "user api management")
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "token",
                    SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
                )
            }
        }
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "rust_api_mongo_kafka=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    let app_state = AppState::new(config.clone()).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/products", product_routes(app_state.clone()))
        .nest("/messages", message_routes(app_state.clone()))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::middleware::auth_middleware,
        ))
        .nest("/auth", auth_routes(app_state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .split_for_parts();

    let router =
        router.merge(SwaggerUi::new("/").url("/api-docs/openapi.json", api.clone()));

    let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
    tracing::info!("Server listening on {}", config.server_addr);
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

fn auth_routes(app_state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(auth::handlers::signup))
        .routes(routes!(auth::handlers::login))
        .with_state(app_state)
}

fn product_routes(app_state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(
            products::handlers::list_products,
            products::handlers::create_product,
        ))
        .routes(routes!(
            products::handlers::delete_product,
            products::handlers::update_product,
            products::handlers::get_product
        ))
        .with_state(app_state)
}

fn message_routes(app_state: AppState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(message::handlers::list_messages))
        .with_state(app_state)
}
