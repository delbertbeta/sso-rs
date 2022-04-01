use async_redis_session::RedisSessionStore;
use std::env;

pub fn get_session_store() -> RedisSessionStore {
    let db_url = env::var("REDIS_URL").expect("REDIS_URL is not set in .env file");

    let session_store = RedisSessionStore::new(db_url).expect("Create redis session store error");

    session_store
}
