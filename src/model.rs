use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::{associations::Identifiable, deserialize::Queryable, prelude::Insertable};

use crate::schema::aiode_supporter;

#[derive(Clone, Debug, Identifiable, Insertable, Queryable)]
#[diesel(table_name = aiode_supporter)]
#[diesel(primary_key(user_id))]
pub struct AiodeSupporter {
    // postgres does not have an unsigned 64 bit integer type, therefore numeric is used
    pub user_id: BigDecimal,
    pub creation_timestamp: DateTime<Utc>,
}
