use std::path::Path;

use crate::api_error::ApiError;
use crate::database::postgresql::{PgPool, PgPooled};
use crate::database::schemas::uploads::dsl as uploads_dsl;
use crate::utils::parse_key;
use actix_web::http::{StatusCode, header};
use actix_web::web::ThinData;
use actix_web::{HttpResponse, post, web};
use diesel::dsl::exists;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::{ExpressionMethods, select};
use diesel_async::RunQueryDsl;
use infer::Infer;
use tokio::fs::remove_file;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Info {
    pub token: String,
    pub uuid: String,
}

#[post("/give_upload")]
pub async fn give_upload(
    info: web::Json<Info>,
    ThinData(pool): ThinData<PgPool>,
) -> Result<HttpResponse, ApiError> {
    if info.token != parse_key::<String>("TOKEN")? {
        return Err(ApiError::new_empty(StatusCode::UNAUTHORIZED));
    }

    let uuid = Uuid::parse_str(&info.uuid)?;
    let mut conn: PgPooled = pool.get().await?;

    let exist: bool = select(exists(
        uploads_dsl::uploads.filter(uploads_dsl::uuid.eq(uuid)),
    ))
    .get_result(&mut conn)
    .await?;

    if !exist {
        return Err(ApiError::new_empty(StatusCode::NOT_FOUND));
    }

    let (expiration, getted): (i64, bool) = uploads_dsl::uploads
        .select((uploads_dsl::expiration, uploads_dsl::getted))
        .filter(uploads_dsl::uuid.eq(uuid))
        .get_result(&mut conn)
        .await?;

    if chrono::Utc::now().timestamp() > expiration {
        return Err(ApiError::new_empty(StatusCode::NOT_FOUND));
    }

    if !getted {
        return Ok(HttpResponse::Accepted().into());
    }

    let file_path = Path::new("apps/files").join(uuid.to_string());

    let file_content = tokio::fs::read(&file_path).await?;

    let Some(mime_type) = Infer::new().get(&file_content) else {
        return Err(ApiError::new_empty(StatusCode::UNSUPPORTED_MEDIA_TYPE));
    };

    remove_file(file_path).await?;

    diesel::delete(uploads_dsl::uploads.filter(uploads_dsl::uuid.eq(uuid)))
        .execute(&mut conn)
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header((
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", uuid),
        ))
        .content_type(mime_type.to_string())
        .body(file_content))
}
