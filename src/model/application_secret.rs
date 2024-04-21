use chrono::Utc;
use entity::application_secret::{self, ActiveModel, Entity, Model};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Set,
};

type QueryVecReturnType = Result<Vec<Model>, DbErr>;
type QueryReturnType = Result<Model, DbErr>;

pub struct ApplicationSecretModel<'a>(&'a DatabaseConnection);

pub struct CreateSecretParams {
    pub secret: String,
    pub app_id: String,
    pub creator_id: i32,
}

impl<'a> ApplicationSecretModel<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self(&conn)
    }

    pub async fn get_secrets_by_application_id(&self, id: &str) -> QueryVecReturnType {
        Entity::find()
            .filter(application_secret::Column::AppId.eq(id))
            .all(self.0)
            .await
    }

    pub async fn create_secret_by_application_id(
        &self,
        params: CreateSecretParams,
    ) -> QueryReturnType {
        let new_secret = ActiveModel {
            id: NotSet,
            app_id: Set(params.app_id),
            secret: Set(params.secret),
            creator_id: Set(params.creator_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        new_secret.insert(self.0).await
    }
}
