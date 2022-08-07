use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_create_table::User, m20220417_000001_add_image::Image};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220807_132032_create_applications"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let application_table = Table::create()
            .table(Application::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Application::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Application::Name).string().not_null())
            .col(ColumnDef::new(Application::IconId).string().not_null())
            .col(ColumnDef::new(Application::Description).string())
            .col(ColumnDef::new(Application::HomepageUrl).string().not_null())
            .col(
                ColumnDef::new(Application::AuthorizationCallbackUrl)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(Application::CreatorId).integer().not_null())
            .col(
                ColumnDef::new(Application::CreatedAt)
                    .date_time()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Application::UpdatedAt)
                    .date_time()
                    .not_null(),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-app-icon-to-image-id")
                    .from_tbl(Application::Table)
                    .from_col(Application::IconId)
                    .to(Image::Table, Image::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-app-creator-to-user-id")
                    .from_tbl(Application::Table)
                    .from_col(Application::CreatorId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();

        println!("{:?}", application_table.to_string(MysqlQueryBuilder));

        manager.create_table(application_table).await?;

        let application_secret_table = Table::create()
            .table(ApplicationSecret::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ApplicationSecret::Id)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(ApplicationSecret::AppId).string().not_null())
            .col(
                ColumnDef::new(ApplicationSecret::CreatorId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ApplicationSecret::Secret)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ApplicationSecret::UpdatedAt)
                    .date_time()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ApplicationSecret::CreatedAt)
                    .date_time()
                    .not_null(),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-app-secret-creator-to-user-id")
                    .from_tbl(ApplicationSecret::Table)
                    .from_col(ApplicationSecret::CreatorId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-app-secret-to-app-id")
                    .from_tbl(ApplicationSecret::Table)
                    .from_col(ApplicationSecret::AppId)
                    .to(Application::Table, Application::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();

        println!(
            "{:?}",
            application_secret_table.to_string(MysqlQueryBuilder)
        );

        manager.create_table(application_secret_table).await?;

        let application_access_grant_table = Table::create()
            .table(ApplicationAccessGrant::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ApplicationAccessGrant::Id)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(ApplicationAccessGrant::AppId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ApplicationAccessGrant::UserId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ApplicationAccessGrant::Scopes)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ApplicationAccessGrant::UpdatedAt)
                    .date_time()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ApplicationAccessGrant::CreatedAt)
                    .date_time()
                    .not_null(),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-app-grant-creator-to-user-id")
                    .from_tbl(ApplicationAccessGrant::Table)
                    .from_col(ApplicationAccessGrant::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-app-grant-to-app-id")
                    .from_tbl(ApplicationAccessGrant::Table)
                    .from_col(ApplicationAccessGrant::AppId)
                    .to(Application::Table, Application::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();

        println!(
            "{:?}",
            application_access_grant_table.to_string(MysqlQueryBuilder)
        );

        manager.create_table(application_access_grant_table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Application::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ApplicationSecret::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ApplicationAccessGrant::Table)
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Application {
    Table,
    Id,
    Name,
    IconId,
    Description,
    HomepageUrl,
    AuthorizationCallbackUrl,
    CreatorId,
    UpdatedAt,
    CreatedAt,
}

#[derive(Iden)]
enum ApplicationSecret {
    Table,
    Id,
    CreatorId,
    AppId,
    Secret,
    UpdatedAt,
    CreatedAt,
}

#[derive(Iden)]
enum ApplicationAccessGrant {
    Table,
    Id,
    UserId,
    AppId,
    Scopes,
    UpdatedAt,
    CreatedAt,
}
