use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_create_table::User, m20220807_132032_create_applications::Application};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250831_000001_add_oidc_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create AuthorizationCode table
        let authorization_code_table = Table::create()
            .table(AuthorizationCode::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(AuthorizationCode::Code)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(AuthorizationCode::ApplicationId).string().not_null())
            .col(ColumnDef::new(AuthorizationCode::UserId).integer().not_null())
            .col(ColumnDef::new(AuthorizationCode::Scopes).json().not_null())
            .col(ColumnDef::new(AuthorizationCode::RedirectUri).string().not_null())
            .col(ColumnDef::new(AuthorizationCode::ExpiresAt).date_time().not_null())
            .col(ColumnDef::new(AuthorizationCode::CreatedAt).date_time().not_null())
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-authcode-to-app-id")
                    .from_tbl(AuthorizationCode::Table)
                    .from_col(AuthorizationCode::ApplicationId)
                    .to(Application::Table, Application::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-authcode-to-user-id")
                    .from_tbl(AuthorizationCode::Table)
                    .from_col(AuthorizationCode::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(authorization_code_table).await?;

        // Create Token table
        let token_table = Table::create()
            .table(Token::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Token::Id)
                    .integer()
                    .auto_increment()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Token::ApplicationId).string().not_null())
            .col(ColumnDef::new(Token::UserId).integer().not_null())
            .col(ColumnDef::new(Token::AccessToken).string().not_null().unique_key())
            .col(ColumnDef::new(Token::RefreshToken).string().not_null().unique_key())
            .col(ColumnDef::new(Token::Scopes).json().not_null())
            .col(ColumnDef::new(Token::ExpiresAt).date_time().not_null())
            .col(ColumnDef::new(Token::CreatedAt).date_time().not_null())
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-token-to-app-id")
                    .from_tbl(Token::Table)
                    .from_col(Token::ApplicationId)
                    .to(Application::Table, Application::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-token-to-user-id")
                    .from_tbl(Token::Table)
                    .from_col(Token::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(token_table).await?;

        // Alter Application table
        manager.alter_table(
            Table::alter()
                .table(Application::Table)
                .drop_column(Alias::new("authorization_callback_url"))
                .add_column(ColumnDef::new(ApplicationOidc::RedirectUris).json().not_null())
                .add_column(ColumnDef::new(ApplicationOidc::GrantTypes).json().not_null())
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuthorizationCode::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Token::Table).to_owned())
            .await?;
        manager.alter_table(
            Table::alter()
                .table(Application::Table)
                .drop_column(ApplicationOidc::RedirectUris)
                .drop_column(ApplicationOidc::GrantTypes)
                .add_column(ColumnDef::new(Alias::new("authorization_callback_url")).string().not_null())
                .to_owned()
        ).await
    }
}

#[derive(Iden)]
enum AuthorizationCode {
    Table,
    Code,
    ApplicationId,
    UserId,
    Scopes,
    RedirectUri,
    ExpiresAt,
    CreatedAt,
}

#[derive(Iden)]
enum Token {
    Table,
    Id,
    ApplicationId,
    UserId,
    AccessToken,
    RefreshToken,
    Scopes,
    ExpiresAt,
    CreatedAt,
}

#[derive(Iden)]
enum ApplicationOidc {
    RedirectUris,
    GrantTypes,
}
