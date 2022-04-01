use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn get_mysql_db_conn() -> DatabaseConnection {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let is_prod = env::var("PROD").map_or(false, |_| true);
    let conn = Database::connect(db_url)
        .await
        .expect("Connect to db error");

    if is_prod {
        Migrator::up(&conn, None).await.expect("Migrator up failed");
    }

    conn
}
