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
    pub redirect_uris: String,
    pub grant_types: String,
    pub creator_id: i32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct UpdateApplicationParams {
    pub name: String,
    pub icon_id: String,
    pub description: Option<String>,
    pub homepage_url: String,
    pub redirect_uris: String,
}

type QueryOptionReturnType = Result<Option<(Model, Option<ImageModel>)>, DbErr>;
type QueryVecReturnType = Result<Vec<(Model, Option<ImageModel>)>, DbErr>;
#[allow(dead_code)]
type QueryOptionNoRelatedReturnType = Result<Option<Model>, DbErr>;
type QueryReturnType = Result<Model, DbErr>;
#[allow(dead_code)]
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
            redirect_uris: Set(serde_json::to_string(&vec![params.redirect_uris]).unwrap()),
            grant_types: Set(serde_json::to_string(&vec![params.grant_types]).unwrap()),
            creator_id: Set(params.creator_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        new_application.insert(self.0).await
    }

    #[allow(dead_code)]
    pub async fn update_application(
        &self,
        mut active_model: ActiveModel,
        params: UpdateApplicationParams,
    ) -> UpdateReturnType {
        active_model.name = Set(params.name);
        active_model.icon_id = Set(params.icon_id);
        active_model.redirect_uris = Set(serde_json::to_string(&vec![params.redirect_uris]).unwrap());
        active_model.homepage_url = Set(params.homepage_url);

        if let Some(description) = params.description {
            active_model.description = Set(Some(description));
        }

        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.save(self.0).await
    }
}
