use crate::api_error::ApiError;
use crate::database::postgresql::{PgPool, PgPooled};
use crate::database::schemas::uploads::dsl as uploads_dsl;
use crate::utils::db::update_upload;
use crate::utils::parse_key;
use actix_web::http::StatusCode;
use actix_web::web::ThinData;
use actix_web::{HttpResponse, post, web};
use diesel::{ExpressionMethods, insert_into};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Info {
    pub token: String,
}

#[post("/generate_upload")]
pub async fn generate_upload(
    info: web::Json<Info>,
    ThinData(pool): ThinData<PgPool>,
) -> Result<HttpResponse, ApiError> {
    if info.token != parse_key::<String>("TOKEN")? {
        return Err(ApiError::new_empty(StatusCode::UNAUTHORIZED));
    }

    let mut conn: PgPooled = pool.get().await?;

    update_upload(&mut conn).await?;

    let uuid: Uuid = insert_into(uploads_dsl::uploads)
        .values((
            uploads_dsl::expiration.eq(chrono::Utc::now().timestamp() + 60 * 10),
            uploads_dsl::getted.eq(false),
        ))
        .returning(uploads_dsl::uuid)
        .get_result::<Uuid>(&mut conn)
        .await?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(uuid.to_string()))
}
