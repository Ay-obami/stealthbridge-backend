use axum::{
    extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router,
};
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::{env, error::Error, sync::Arc, time::Duration};
use tracing::warn;
use uuid::Uuid;
use crate::transaction::{self,TransactionObservation};

/// The only network accepted until privacy, compliance and security gates are satisfied.
const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

#[derive(Clone)]
pub struct AppState {
    http: Client,
    rpc_url: String,
    db: Option<PgPool>,
}

impl AppState {
    pub async fn from_env() -> Result<Self, Box<dyn Error>> {
        let rpc_url = env::var("STELLAR_RPC_URL")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_owned());
        let url = reqwest::Url::parse(&rpc_url)?;
        if url.scheme() != "https" {
            return Err("STELLAR_RPC_URL must use HTTPS".into());
        }
        let http = Client::builder().timeout(Duration::from_secs(8)).build()?;
        let db = match env::var("DATABASE_URL") {
            Ok(database_url) if !database_url.is_empty() => {
                // Do not automatically apply migrations in the application process.
                Some(PgPoolOptions::new().max_connections(10).connect(&database_url).await?)
            }
            _ => None,
        };
        Ok(Self {http, rpc_url, db})
    }

    #[cfg(test)]
    pub fn without_db() -> Self {
        Self {
            http: Client::new(),
            rpc_url: "https://soroban-testnet.stellar.org".to_owned(),
            db: None,
        }
    }

    async fn rpc(&self, method: &str) -> Result<Value, ()> {
        self.rpc_with_params(method, None).await
    }

    async fn rpc_with_params(&self, method: &str, params: Option<Value>) -> Result<Value, ()> {
        let response = self.http.post(&self.rpc_url)
            .json(&json!({"jsonrpc":"2.0","id":"stealthbridge-observer","method":method,"params":params.unwrap_or(json!({}))}))
            .send().await.map_err(|_| ())?;
        if !response.status().is_success() {return Err(());}
        let payload: Value = response.json().await.map_err(|_| ())?;
        if payload.get("error").is_some() {return Err(());}
        payload.get("result").cloned().ok_or(())
    }

    async fn network(&self) -> Result<NetworkStatus, ()> {
        let network = self.rpc("getNetwork").await?;
        let actual_passphrase = network.get("passphrase").and_then(Value::as_str).ok_or(())?;
        if actual_passphrase != TESTNET_PASSPHRASE {
            warn!("Configured Stellar RPC did not identify as testnet");
            return Err(());
        }
        let latest = self.rpc("getLatestLedger").await?;
        Ok(NetworkStatus {
            network: "testnet",
            passphrase: actual_passphrase.to_owned(),
            protocol_version: latest.get("protocolVersion").and_then(Value::as_u64).ok_or(())?,
            ledger_sequence: latest.get("sequence").and_then(Value::as_u64).ok_or(())?,
            ledger_closed_at_unix: latest.get("closeTime").and_then(Value::as_str).ok_or(())?.to_owned(),
            ledger_hash: latest.get("id").and_then(Value::as_str).ok_or(())?.to_owned(),
            source: "stellar-rpc",
        })
    }
}

#[derive(Serialize)]
pub struct NetworkStatus {
    network: &'static str,
    passphrase: String,
    protocol_version: u64,
    ledger_sequence: u64,
    ledger_closed_at_unix: String,
    ledger_hash: String,
    source: &'static str,
}

#[derive(Serialize)]
pub struct Capabilities {
    payments_enabled: bool,
    confidential_token_verified: bool,
    private_payments_verified: bool,
    fiat_payouts_enabled: bool,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize, FromRow)]
pub struct Corridor {
    id: Uuid,
    origin_country: String,
    destination_country: String,
    asset_code: String,
    asset_issuer: Option<String>,
    privacy_rail: String,
}

/// This endpoint is not evidence of chain connectivity; /v1/network verifies live RPC.
async fn health() -> Json<Health> {
    Json(Health {status:"ok", service:"stealthbridge-backend"})
}
async fn network(State(state): State<Arc<AppState>>) -> Result<Json<NetworkStatus>, StatusCode> {
    state.network().await.map(Json).map_err(|_| StatusCode::BAD_GATEWAY)
}
async fn capabilities() -> Json<Capabilities> {
    // No real payment handlers or privacy verifications have been implemented.
    // These flags remain false until code and independently reproducible evidence exist.
    Json(Capabilities{
        payments_enabled:false,
        confidential_token_verified:false,
        private_payments_verified:false,
        fiat_payouts_enabled:false,
    })
}
async fn corridors(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Corridor>>, StatusCode> {
    let pool = state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let rows = sqlx::query_as::<_, Corridor>(
        "SELECT id, origin_country, destination_country, asset_code, asset_issuer, privacy_rail \
         FROM corridors WHERE enabled = TRUE ORDER BY origin_country, destination_country, id"
    ).fetch_all(pool).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(rows))
}

#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    message: &'static str,
}
async fn disabled() -> (StatusCode, Json<ApiError>) {
    (StatusCode::NOT_IMPLEMENTED, Json(ApiError {
        code:"NOT_AVAILABLE",
        message:"Confidential settlement is not enabled. No transaction was submitted.",
    }))
}


async fn public_transaction(
    Path(hash): Path<String>, State(state): State<Arc<AppState>>,
) -> Result<Json<TransactionObservation>, StatusCode> {
    if !transaction::valid_hash(&hash) { return Err(StatusCode::BAD_REQUEST); }
    // Fail closed against accidentally pointing a deployment at another network.
    let network = state.rpc("getNetwork").await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    if network.get("passphrase").and_then(Value::as_str) != Some(TESTNET_PASSPHRASE) {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let result = state.rpc_with_params("getTransaction", Some(json!({"hash":hash.to_ascii_lowercase()})))
        .await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    transaction::parse_result(&hash,&result)
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health",get(health))
        .route("/v1/network",get(network))
        .route("/v1/capabilities",get(capabilities))
        .route("/v1/corridors",get(corridors))
        .route("/v1/transactions/{hash}",get(public_transaction))
        .route("/v1/settlements",post(disabled))
        .with_state(Arc::new(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn router_constructs_without_database_or_credentials() {
        let _ = router(AppState::without_db());
    }
    #[test]
    fn testnet_network_passphrase_is_explicit() {
        assert_eq!(TESTNET_PASSPHRASE, "Test SDF Network ; September 2015");
    }
}
