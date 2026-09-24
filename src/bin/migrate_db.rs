use diesel::mysql::MysqlConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use gene_plarform::db;

// 1. Embed migrations into the binary
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(conn: &mut MysqlConnection) {
    // 2. Run pending migrations
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

fn main() {
    dotenv::dotenv().ok();

    let mut conn = db::DB::default().init_conn();

    run_migrations(&mut conn);
}
