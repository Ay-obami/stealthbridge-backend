//! Unfunded read-only testnet API scaffold. Payment commands intentionally disabled.
use axum::{routing::{get, post}, http::StatusCode, Json, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Serialize)]
struct Health { service: &'static str, status: &'static str, network: &'static str }
#[derive(Serialize)]
struct Capabilities { payments_enabled: bool, privacy_integration_verified: bool, fiat_payouts_enabled: bool }
#[derive(Serialize)]
struct Corridor { id: &'static str, from: &'static str, to: &'static str, settlement_asset: &'static str, simulation: bool }
#[derive(Serialize)]
struct ErrorResponse { code: &'static str, message: &'static str }

async fn health() -> Json<Health> {
    Json(Health {service:"stealthbridge-backend",status:"ok",network:"testnet"})
}
async fn capabilities() -> Json<Capabilities> {
    Json(Capabilities{payments_enabled:false,privacy_integration_verified:false,fiat_payouts_enabled:false})
}
async fn corridors() -> Json<Vec<Corridor>> {
    Json(vec![Corridor {id:"ng-ke-demo",from:"NG",to:"KE",settlement_asset:"DEMO-USD",simulation:true}])
}
async fn payment_disabled() -> (StatusCode,Json<ErrorResponse>) {
    (StatusCode::NOT_IMPLEMENTED,Json(ErrorResponse {
        code:"PAYMENTS_NOT_ENABLED",
        message:"No transactions can be initiated during the architecture and privacy feasibility phase.",
    }))
}
pub fn app() -> Router {
    Router::new()
        .route("/health",get(health))
        .route("/v1/capabilities",get(capabilities))
        .route("/v1/corridors",get(corridors))
        .route("/v1/settlements",post(payment_disabled))
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))).init();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    tracing::info!("StealthBridge local read-only API listening on 127.0.0.1:8080");
    axum::serve(listener,app()).await?;
    Ok(())
}

#[cfg(test)]
mod api_tests {
  use super::*;
  #[test]
  fn creates_router_without_external_config() { let _=app(); }
}
