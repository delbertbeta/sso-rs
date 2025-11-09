use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AuthorizationCode::Table)
                    .add_column(
                        ColumnDef::new(AuthorizationCode::CodeChallenge)
                            .string()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(AuthorizationCode::CodeChallengeMethod)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AuthorizationCode::Table)
                    .drop_column(AuthorizationCode::CodeChallenge)
                    .drop_column(AuthorizationCode::CodeChallengeMethod)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AuthorizationCode {
    Table,
    CodeChallenge,
    CodeChallengeMethod,
}
