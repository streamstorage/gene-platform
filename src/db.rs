use diesel::Connection as _;
use diesel::pg::PgConnection;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, deadpool};
use diesel_async::{AsyncConnection, AsyncPgConnection};
use std::env;

pub type SyncConnection = PgConnection;

pub type Connection = AsyncPgConnection;

pub type Pool = deadpool::Pool<AsyncPgConnection>;

pub struct DB {
    pub url: String,
}

// schema
// DATABASE_URL=mysql://username:password@localhost:3306/database_name
impl Default for DB {
    fn default() -> Self {
        Self {
            url: env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file"),
        }
    }
}

impl DB {
    pub fn init_pool(&self) -> Pool {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&self.url);
        deadpool::Pool::builder(config)
            .max_size(50)
            .build()
            .expect("create db pool.")
    }

    pub fn init_conn(&self) -> PgConnection {
        PgConnection::establish(&self.url).expect("create db conn")
    }

    pub async fn init_async_conn(&self) -> Connection {
        AsyncPgConnection::establish(&self.url)
            .await
            .expect("create async db conn")
    }
}
