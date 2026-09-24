use actix_session::{SessionMiddleware, config::PersistentSession, storage::RedisSessionStore};
use actix_web::cookie::{Key, time::Duration};
use actix_web::{App, HttpServer, middleware, web};
use gene_plarform::{cache, db, routers, utils};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // initialize environment
    dotenv::dotenv().ok();

    // initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let db_pool = db::DB::default().init_pool();
    let redis_pool = cache::Cache::default().init_pool();
    let redis_store = RedisSessionStore::new_pooled(redis_pool)
        .await
        .expect("Redis session store");
    let secret_key = Key::from(&utils::get_secret_key());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                    .build(),
            )
            .configure(routers::config)
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
