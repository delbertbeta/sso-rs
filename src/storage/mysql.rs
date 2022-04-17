use crate::constants::ENVS;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub async fn get_mysql_db_conn() -> DatabaseConnection {
    let db_url = &ENVS.database_url;
    let is_prod = ENVS.prod;
    let conn = Database::connect(db_url)
        .await
        .expect("Connect to db error");

    if is_prod {
        Migrator::up(&conn, None).await.expect("Migrator up failed");
    }

    conn
}
