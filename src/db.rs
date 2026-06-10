use sqlx::{postgres::PgPoolOptions, PgPool};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::env;
use std::str::FromStr;
use std::time::Duration;

pub async fn init_db_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut options:PgConnectOptions = PgConnectOptions::from_str(&db_url)
        .expect("Invalid DATABASE_URL format");
    options = options.statement_cache_capacity(0);

    let u = db_url.to_lowercase();
    if u.contains("supabase.co") {
        options = options.ssl_mode(PgSslMode::Require);
    }

    PgPoolOptions::new()
        .max_connections(5)
        .min_connections(0)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect_lazy_with(options)
}