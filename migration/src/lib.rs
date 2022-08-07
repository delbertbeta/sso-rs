pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20220417_000001_add_image;
mod m20220807_132032_create_applications;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20220417_000001_add_image::Migration),
            Box::new(m20220807_132032_create_applications::Migration),
        ]
    }
}
