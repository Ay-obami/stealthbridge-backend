//! Real PostgreSQL integration tests; use isolated GitHub Actions service.
//! No fiat providers, wallet secrets, testnet accounts or funded transfers.

use sqlx::{postgres::PgPoolOptions, Row};
use stealthbridge_backend::{
    settlement::SettlementState,
    store::{advance_intent, register_intent, JournalError},
};
use uuid::Uuid;

#[tokio::test]
async fn idempotency_is_organization_scoped_and_history_is_append_only() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for integration test");
    let pool = PgPoolOptions::new().max_connections(6).connect(&url).await.expect("postgres");
    sqlx::migrate!("./migrations").run(&pool).await.expect("schema migration");

    // Synthetic values are limited to this isolated integration test.
    let corridor = Uuid::new_v4();
    sqlx::query("INSERT INTO corridors (id,origin_country,destination_country,asset_code,privacy_rail) \
                 VALUES ($1,'AA','BB','INTEGRATION_TEST','confidential-token')")
        .bind(corridor).execute(&pool).await.expect("fixture corridor");

    let organization = Uuid::new_v4();
    let another_organization = Uuid::new_v4();
    let key = format!("integration-test-{}", Uuid::new_v4());
    let digest = "a".repeat(64);

    let (first,second) = tokio::join!(
        register_intent(&pool, organization, corridor, &key, &digest),
        register_intent(&pool, organization, corridor, &key, &digest)
    );
    let first=first.expect("first registration");
    let second=second.expect("second registration");
    assert_eq!(first.id, second.id);
    assert_ne!(first.created, second.created);

    assert!(matches!(
        register_intent(&pool,organization,corridor,&key,&"b".repeat(64)).await,
        Err(JournalError::PayloadConflict)
    ));

    let other = register_intent(&pool,another_organization,corridor,&key,&digest)
        .await.expect("different tenant is independent");
    assert_ne!(other.id,first.id);
    assert!(other.created);

    assert!(matches!(
        advance_intent(&pool,another_organization,first.id,0,SettlementState::Quoted).await,
        Err(JournalError::NotFound)
    ));
    assert_eq!(advance_intent(&pool,organization,first.id,0,SettlementState::Quoted)
        .await.expect("transition"),1);
    assert!(matches!(
        advance_intent(&pool,organization,first.id,0,SettlementState::Authorized).await,
        Err(JournalError::VersionConflict)
    ));
    assert!(matches!(
        advance_intent(&pool,organization,first.id,1,SettlementState::PayoutCompleted).await,
        Err(JournalError::InvalidTransition)
    ));

    let rows=sqlx::query("SELECT previous_state,next_state,version \
        FROM settlement_transition_log WHERE organization_id=$1 AND settlement_id=$2 ORDER BY version")
        .bind(organization).bind(first.id).fetch_all(&pool).await.expect("history");
    assert_eq!(rows.len(),2);
    let initial:Option<String>=rows[0].try_get("previous_state").expect("initial state");
    let draft:String=rows[0].try_get("next_state").expect("initial state");
    let second:String=rows[1].try_get("next_state").expect("later state");
    assert_eq!(initial,None);
    assert_eq!(draft,"draft");
    assert_eq!(second,"quoted");

    // The database in CI is disposable. These cleanups are scoped to fixtures.
    sqlx::query("DELETE FROM settlement_transition_log WHERE settlement_id IN ($1,$2)")
        .bind(first.id).bind(other.id).execute(&pool).await.expect("fixture cleanup");
    sqlx::query("DELETE FROM settlement_intents WHERE id IN ($1,$2)")
        .bind(first.id).bind(other.id).execute(&pool).await.expect("fixture cleanup");
    sqlx::query("DELETE FROM corridors WHERE id=$1")
        .bind(corridor).execute(&pool).await.expect("fixture cleanup");
}
