use chrono::Utc;
use entity::image::Entity as Image;
use entity::image::Model as ImageModel;
use entity::user;
use entity::user::ActiveModel;
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

pub struct UpdateUserInfoParams {
    pub nickname: String,
    pub self_info: String,
    pub face_id: Option<String>,
}

type QueryOptionReturnType = Result<Option<(Model, Option<ImageModel>)>, DbErr>;
type QueryOptionNoRelatedReturnType = Result<Option<Model>, DbErr>;
type QueryReturnType = Result<Model, DbErr>;
type UpdateReturnType = Result<ActiveModel, DbErr>;

impl<'a> UserModel<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self(&conn)
    }

    pub async fn find_one_user_by_username_no_related(
        &self,
        username: &str,
    ) -> QueryOptionNoRelatedReturnType {
        User::find()
            .filter(user::Column::Username.eq(username))
            .one(self.0)
            .await
    }

    pub async fn find_one_user_by_id_no_related(&self, id: &i32) -> QueryOptionNoRelatedReturnType {
        User::find()
            .filter(user::Column::Id.eq(id.clone()))
            .one(self.0)
            .await
    }

    pub async fn find_one_user_by_username(&self, username: &str) -> QueryOptionReturnType {
        User::find()
            .find_also_related(Image)
            .filter(user::Column::Username.eq(username))
            .one(self.0)
            .await
    }

    pub async fn find_one_user_by_id(&self, id: &i32) -> QueryOptionReturnType {
        User::find()
            .find_also_related(Image)
            .filter(user::Column::Id.eq(id.clone()))
            .one(self.0)
            .await
    }

    pub async fn insert_user(&self, params: CreateUserParams) -> QueryReturnType {
        let new_user = user::ActiveModel {
            id: NotSet,
            username: Set(params.username),
            salt: Set(params.salt),
            email: Set(Some(params.email)),
            face_id: NotSet,
            password_hash: Set(params.password_hash),
            nickname: Set(params.nickname),
            self_info: Set(Some("".to_string())),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        new_user.insert(self.0).await
    }

    pub async fn update_user_info(
        &self,
        mut active_model: ActiveModel,
        params: UpdateUserInfoParams,
    ) -> UpdateReturnType {
        active_model.nickname = Set(params.nickname);
        active_model.self_info = Set(Some(params.self_info));

        if let Some(face_id) = params.face_id {
            active_model.face_id = Set(Some(face_id));
        }

        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.save(self.0).await
    }
}
