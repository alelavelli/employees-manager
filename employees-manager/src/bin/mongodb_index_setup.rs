use bson::{doc, Document};
use employees_manager::{model::db_entities, service::db::DatabaseDocument};
use tracing::{error, info};

async fn create_index<T: 'static + DatabaseDocument>(keys: Document) {
    info!("Creating indexes for {} collection", T::collection_name());
    if let Err(e) = T::set_indexes(keys).await {
        error!("Got error {}", e);
    } else {
        info!("Index created");
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(true)
        .init();

    create_index::<db_entities::UserCompanyAssignment>(doc! {"company_id": 1, "user_id": 1}).await;
}
