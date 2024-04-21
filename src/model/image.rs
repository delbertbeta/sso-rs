use chrono::Utc;
use entity::image;
use image::ActiveModel;
use image::Entity as Image;
use image::Model;
use sea_orm::DbErr;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct ImageModel<'a>(&'a DatabaseConnection);

pub struct CreateImageParams<'a> {
    pub id: &'a str,
    pub path: &'a str,
    pub user_id: i32,
}

type QueryOptionReturnType = Result<Option<Model>, DbErr>;
type QueryReturnType = Result<Model, DbErr>;
type UpdateReturnType = Result<ActiveModel, DbErr>;

impl<'a> ImageModel<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self(&conn)
    }

    pub async fn find_one_image_by_id(&self, id: &str) -> QueryOptionReturnType {
        Image::find()
            .filter(image::Column::Id.eq(id))
            .one(self.0)
            .await
    }

    pub async fn insert_image(&self, params: CreateImageParams<'a>) -> QueryReturnType {
        let new_image = ActiveModel {
            id: Set(params.id.to_owned()),
            path: Set(params.path.to_owned()),
            uploaded: Set(Some(false as i8)),
            user_id: Set(params.user_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        new_image.insert(self.0).await
    }

    pub async fn set_uploaded(
        &self,
        mut active_model: ActiveModel,
        uploaded: bool,
    ) -> UpdateReturnType {
        active_model.uploaded = Set(Some(uploaded as i8));
        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.save(self.0).await
    }
}
