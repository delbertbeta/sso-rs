use sea_schema::migration::{
    prelude::*,
    sea_query::{self},
};

pub struct Migration;

#[derive(Iden)]
pub enum Image {
    Table,
    Id,
    Path,
    Uploaded,
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
            .col(
                ColumnDef::new(Image::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Image::Path).string().not_null())
            .col(
                ColumnDef::new(Image::Uploaded)
                    .boolean()
                    .default(Value::Bool(Some(false))),
            )
            .col(ColumnDef::new(Image::CreatedAt).date_time().not_null())
            .col(ColumnDef::new(Image::UpdatedAt).date_time().not_null())
            .to_owned();

        println!("{:?}", image_table.to_string(MysqlQueryBuilder));
        manager.create_table(image_table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await
    }
}
