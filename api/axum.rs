//! Stateless Vercel Rust Function adapter for the existing Axum router.
//! Does not run the persistent ledger observer (serverless has no worker lease).
use tower::ServiceBuilder;
use vercel_runtime::{axum::VercelLayer,Error};

#[tokio::main]
async fn main()->Result<(),Error>{
    let state=stealthbridge_backend::server::AppState::from_env().await
        .map_err(|err|std::io::Error::other(err.to_string()))?;
    let app=ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(stealthbridge_backend::server::router(state));
    vercel_runtime::run(app).await
}
