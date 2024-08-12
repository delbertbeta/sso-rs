use crate::constants::ENVS;
use sea_orm::{Database, DatabaseConnection};

pub async fn get_mysql_db_conn() -> DatabaseConnection {
    let db_url = &ENVS.database_url;
    let conn = Database::connect(db_url)
        .await
        .expect("Connect to db error");

    conn
}
