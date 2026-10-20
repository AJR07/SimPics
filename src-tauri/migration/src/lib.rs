pub use sea_orm_migration::prelude::*;

mod m20261020_141047_date_to_datetimeutc;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261020_141047_date_to_datetimeutc::Migration),
        ]
    }
}
