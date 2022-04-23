use sea_schema::migration::{
    prelude::*,
    sea_query::{self},
};

pub struct Migration;

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Nickname,
    Username,
    Email,
    Salt,
    PasswordHash,
    FaceId,
    SelfInfo,
    CreatedAt,
    UpdatedAt,
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let user_table = Table::create()
            .table(User::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(User::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(User::Username).string().not_null())
            .col(ColumnDef::new(User::Email).string())
            .col(ColumnDef::new(User::FaceId).string())
            .col(ColumnDef::new(User::Nickname).string().not_null())
            .col(ColumnDef::new(User::PasswordHash).string().not_null())
            .col(ColumnDef::new(User::Salt).string().not_null())
            .col(
                ColumnDef::new(User::SelfInfo)
                    .string()
                    .default(Value::String(Some(Box::new(String::from(""))))),
            )
            .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
            .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
            .to_owned();

        println!("{:?}", user_table.to_string(MysqlQueryBuilder));
        manager.create_table(user_table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
