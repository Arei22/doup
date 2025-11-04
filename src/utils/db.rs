use std::path::Path;

use diesel::{
    ExpressionMethods,
    query_dsl::methods::{FilterDsl, SelectDsl},
};
use diesel_async::RunQueryDsl;
use tokio::fs::remove_file;
use uuid::Uuid;

use crate::{
    api_error::ApiError,
    database::{postgresql::PgPooled, schemas::uploads::dsl as uploads_dsl},
};

pub async fn update_upload(conn: &mut PgPooled<'_>) -> Result<(), ApiError> {
    let uuids: Vec<Uuid> = uploads_dsl::uploads
        .select(uploads_dsl::uuid)
        .filter(uploads_dsl::expiration.lt(chrono::Utc::now().timestamp()))
        .get_results(conn)
        .await?;

    for uuid in uuids {
        let path = Path::new("apps/files").join(uuid.to_string());

        if path.exists() {
            remove_file(path).await?;
        }
    }

    diesel::delete(
        uploads_dsl::uploads.filter(uploads_dsl::expiration.lt(chrono::Utc::now().timestamp())),
    )
    .execute(conn)
    .await?;

    Ok(())
}
