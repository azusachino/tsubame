pub mod mysql;

pub struct Redis;

pub struct GlobalState {
    pub db: mysql::Db,
    pub redis: Redis,
}

impl GlobalState {
    pub async fn new() -> Self {
        let db = mysql::Db::new().await;
        let redis = Redis {};
        Self { db, redis }
    }
}
