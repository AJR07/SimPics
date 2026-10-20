use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

#[derive(DeriveIden)]
enum ScreenshotEntry {
    #[sea_orm(iden = "ScreenshotEntry")]
    Table,
    Date,
    DateNew,
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261020_141047_date_to_datetimeutc"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. add the new column (nullable, since SQLite can't add NOT NULL without a default)
        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .add_column(timestamp_with_time_zone_null(ScreenshotEntry::DateNew))
                    .to_owned(),
            )
            .await?;

        // 2. convert existing data: '2026-10-20' -> '2026-10-20 00:00:00'
        manager
            .get_connection()
            .execute_unprepared("UPDATE ScreenshotEntry SET date_new = datetime(date)")
            .await?;

        // 3. drop the old column
        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .drop_column(ScreenshotEntry::Date)
                    .to_owned(),
            )
            .await?;

        // 4. rename the new column to the old name
        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .rename_column(ScreenshotEntry::DateNew, ScreenshotEntry::Date)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .add_column(date_null(ScreenshotEntry::DateNew))
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("UPDATE ScreenshotEntry SET date_new = date(date)")
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .drop_column(ScreenshotEntry::Date)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .rename_column(ScreenshotEntry::DateNew, ScreenshotEntry::Date)
                    .to_owned(),
            )
            .await
    }
}