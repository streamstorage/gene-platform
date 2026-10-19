use deadpool_redis::{Config, Pool, Runtime};
use std::env;

pub struct Cache {
    pub url: String,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            url: env::var("REDIS_URL").expect("REDIS_URL is not set in .env file"),
        }
    }
}

impl Cache {
    pub fn init_pool(&self) -> Pool {
        let redis_cfg = Config::from_url(&self.url);
        redis_cfg
            .create_pool(Some(Runtime::Tokio1))
            .expect("redis pool")
    }
}
