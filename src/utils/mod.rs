use crate::api_error::ApiError;
use actix_web::http::StatusCode;
use std::str::FromStr;

pub mod db;
pub mod files;
pub mod logger;
pub mod templates;

pub fn parse_key<T: FromStr>(key: &str) -> Result<T, ApiError>
where
    <T as FromStr>::Err: std::fmt::Display,
{
    std::env::var(key)?.parse::<T>().map_err(|error| {
        ApiError::new_log(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Failed to parse `{key}` as `{}`: {error}",
                std::any::type_name::<T>(),
            ),
        )
    })
}
