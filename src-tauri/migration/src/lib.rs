pub use sea_orm_migration::prelude::*;

mod m20261020_141047_date_to_datetimeutc;
mod m20261101_132141_add_path;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261020_141047_date_to_datetimeutc::Migration),
            Box::new(m20261101_132141_add_path::Migration),
        ]
    }
}
