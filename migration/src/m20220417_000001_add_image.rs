use crate::m20220101_000001_create_table::User;
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

pub struct Migration;

#[derive(Iden)]
pub enum Image {
    Table,
    Id,
    Path,
    Uploaded,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220417_000001_add_image"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let image_table = Table::create()
            .table(Image::Table)
            .if_not_exists()
            .col(ColumnDef::new(Image::Id).string().not_null().primary_key())
            .col(ColumnDef::new(Image::Path).string().not_null())
            .col(
                ColumnDef::new(Image::Uploaded)
                    .boolean()
                    .default(Value::Bool(Some(false))),
            )
            .col(ColumnDef::new(Image::UserId).integer().not_null())
            .col(ColumnDef::new(Image::CreatedAt).date_time().not_null())
            .col(ColumnDef::new(Image::UpdatedAt).date_time().not_null())
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-user-id-to-images")
                    .from_tbl(Image::Table)
                    .from_col(Image::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();

        println!("{:?}", image_table.to_string(MysqlQueryBuilder));

        manager.create_table(image_table).await?;

        let sql = "ALTER TABLE `user` ADD CONSTRAINT `fk-user-face-to-image-id` FOREIGN KEY (`face_id`) REFERENCES `image` (`id`) ON DELETE SET NULL ON UPDATE CASCADE";
        println!("{}", sql);

        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = "ALTER TABLE `user` DROP CONSTRAINT `fk-user-face-to-image-id`";
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())?;

        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await
    }
}
