use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use warp::{reject::Rejection, reply::Reply};

use crate::{acquire_db_connection, error::Error, model::AiodeSupporter, schema::aiode_supporter};

#[derive(Serialize)]
pub struct CheckIsAiodeSupporterResponse {
    pub is_supporter: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supporter_since: Option<DateTime<Utc>>,
}

pub async fn check_is_aiode_supporter_handler(user_id: u64) -> Result<impl Reply, Rejection> {
    let mut connection = acquire_db_connection().await?;

    let supporter = aiode_supporter::table
        .filter(aiode_supporter::user_id.eq::<BigDecimal>(user_id.into()))
        .get_result::<AiodeSupporter>(&mut connection)
        .await
        .optional()
        .map_err(Error::from)?;

    Ok(warp::reply::json(&CheckIsAiodeSupporterResponse {
        is_supporter: supporter.is_some(),
        supporter_since: supporter.map(|s| s.creation_timestamp),
    }))
}
