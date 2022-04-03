use chrono::Utc;
use entity::user;
use sea_orm::DbErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, NotSet, QueryFilter, Set,
};
use user::Entity as User;
use user::Model;

pub struct UserModel<'a>(&'a DatabaseConnection);

pub struct CreateUserParams {
    pub username: String,
    pub salt: String,
    pub email: String,
    pub nickname: String,
    pub password_hash: String,
}

type QueryOptionReturnType = Result<Option<Model>, DbErr>;
type QueryReturnType = Result<Model, DbErr>;

impl<'a> UserModel<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self(&conn)
    }

    pub async fn find_one_user(&self, username: &str) -> QueryOptionReturnType {
        User::find()
            .filter(user::Column::Username.eq(username.clone()))
            .one(self.0)
            .await
    }

    pub async fn insert_user(&self, params: CreateUserParams) -> QueryReturnType {
        let new_user = user::ActiveModel {
            id: NotSet,
            username: Set(params.username),
            salt: Set(params.salt),
            email: Set(Some(params.email)),
            face_url: NotSet,
            password_hash: Set(params.password_hash),
            nickname: Set(params.nickname),
            self_info: Set(Some("".to_string())),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        new_user.insert(self.0).await
    }
}
