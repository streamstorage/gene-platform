use diesel::Connection as _;
use diesel::mysql::MysqlConnection;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, deadpool};
use diesel_async::{AsyncConnection, AsyncMysqlConnection};
use std::env;

pub type Connection = AsyncMysqlConnection;

pub type Pool = deadpool::Pool<AsyncMysqlConnection>;

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
        let config =
            AsyncDieselConnectionManager::<diesel_async::AsyncMysqlConnection>::new(&self.url);
        deadpool::Pool::builder(config)
            .build()
            .expect("create db pool.")
    }

    pub fn init_conn(&self) -> MysqlConnection {
        MysqlConnection::establish(&self.url).expect("create db conn")
    }

    pub async fn init_async_conn(&self) -> Connection {
        AsyncMysqlConnection::establish(&self.url)
            .await
            .expect("create async db conn")
    }
}
