use async_redis_session::RedisSessionStore;

use crate::constants::ENVS;

pub fn get_session_store() -> RedisSessionStore {
    let db_url = &ENVS.redis_url;

    let session_store =
        RedisSessionStore::new(db_url.clone()).expect("Create redis session store error");

    session_store
}
