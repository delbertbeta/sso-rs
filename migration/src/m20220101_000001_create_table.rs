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
    PasswordHash,
    FaceUrl,
    SelfInfo,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Rsa {
    Table,
    Id,
    UserId,
    RsaPrivateKey,
    RsaPublicKey,
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
            .col(ColumnDef::new(User::FaceUrl).string())
            .col(ColumnDef::new(User::Nickname).string().not_null())
            .col(ColumnDef::new(User::PasswordHash).string().not_null())
            .col(
                ColumnDef::new(User::SelfInfo)
                    .string()
                    .default(Value::String(Some(Box::new(String::from(""))))),
            )
            .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
            .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
            .to_owned();

        println!("{:?}", user_table.to_string(MysqlQueryBuilder));
        manager.create_table(user_table).await?;

        let rsa_table = Table::create()
            .table(Rsa::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Rsa::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Rsa::RsaPrivateKey).string().not_null())
            .col(ColumnDef::new(Rsa::RsaPublicKey).string().not_null())
            .col(ColumnDef::new(Rsa::UpdatedAt).date_time().not_null())
            .col(ColumnDef::new(Rsa::CreatedAt).date_time().not_null())
            .col(ColumnDef::new(Rsa::UserId).integer().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("UserId_FK")
                    .from(Rsa::Table, Rsa::UserId)
                    .to(User::Table, User::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned();

        println!("{:?}", rsa_table.to_string(MysqlQueryBuilder));
        manager.create_table(rsa_table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Rsa::Table).to_owned())
            .await
    }
}
