use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

pub async fn init(config: &crate::config::Config) -> anyhow::Result<Arc<Database>> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.postgres_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(Arc::new(Database {
        pool: Arc::new(pool),
    }))
}
