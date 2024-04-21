use chrono::Utc;
use entity::application::{self, ActiveModel, Entity, Model};
use entity::image::{Entity as ImageEntity, Model as ImageModel};
use sea_orm::DbErr;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct ApplicationModel<'a>(&'a DatabaseConnection);

pub struct CreateApplicationParams {
    pub id: String,
    pub name: String,
    pub icon_id: String,
    pub description: Option<String>,
    pub homepage_url: String,
    pub authorization_callback_url: String,
    pub creator_id: i32,
}

pub struct UpdateApplicationParams {
    pub name: String,
    pub icon_id: String,
    pub description: Option<String>,
    pub homepage_url: String,
    pub authorization_callback_url: String,
}

type QueryOptionReturnType = Result<Option<(Model, Option<ImageModel>)>, DbErr>;
type QueryVecReturnType = Result<Vec<(Model, Option<ImageModel>)>, DbErr>;
type QueryOptionNoRelatedReturnType = Result<Option<Model>, DbErr>;
type QueryReturnType = Result<Model, DbErr>;
type UpdateReturnType = Result<ActiveModel, DbErr>;

impl<'a> ApplicationModel<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self(&conn)
    }

    pub async fn find_one_application_by_id(&self, id: &str) -> QueryOptionReturnType {
        Entity::find()
            .find_also_related(ImageEntity)
            .filter(application::Column::Id.eq(id))
            .one(self.0)
            .await
    }

    pub async fn find_applications_by_user_id(&self, user_id: &i32) -> QueryVecReturnType {
        Entity::find()
            .find_also_related(ImageEntity)
            .filter(application::Column::CreatorId.eq(user_id.clone()))
            .all(self.0)
            .await
    }

    pub async fn insert_application(&self, params: CreateApplicationParams) -> QueryReturnType {
        let new_application = ActiveModel {
            id: Set(params.id),
            name: Set(params.name),
            icon_id: Set(params.icon_id),
            description: Set(params.description),
            homepage_url: Set(params.homepage_url),
            authorization_callback_url: Set(params.authorization_callback_url),
            creator_id: Set(params.creator_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        new_application.insert(self.0).await
    }

    pub async fn update_application(
        &self,
        mut active_model: ActiveModel,
        params: UpdateApplicationParams,
    ) -> UpdateReturnType {
        active_model.name = Set(params.name);
        active_model.icon_id = Set(params.icon_id);
        active_model.authorization_callback_url = Set(params.authorization_callback_url);
        active_model.homepage_url = Set(params.homepage_url);

        if let Some(description) = params.description {
            active_model.description = Set(Some(description));
        }

        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.save(self.0).await
    }
}
