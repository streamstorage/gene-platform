pub mod user;

use diesel::prelude::*;
use diesel::sql_types::Integer;

#[derive(QueryableByName)]
pub struct Sequence {
    #[diesel(sql_type = Integer)]
    pub id: i32,
}
