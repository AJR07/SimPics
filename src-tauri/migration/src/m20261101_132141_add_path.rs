use sea_orm_migration::{prelude::*, schema::*};

const SCREENSHOT_DIR: &str = r"C:\Users\User\OneDrive\Pictures\Screenshots";

pub struct Migration;

#[derive(DeriveIden)]
enum ScreenshotEntry {
    #[sea_orm(iden = "ScreenshotEntry")]
    Table,
    Name,
    Path,
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261101_132141_add_path"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add the column. NOT NULL needs a default so existing rows are valid.
        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .add_column(string(ScreenshotEntry::Path).default(""))
                    .to_owned(),
            )
            .await?;

        // 2. Backfill: path = "<dir>\" || name
        let backfill = Query::update()
            .table(ScreenshotEntry::Table)
            .value(
                ScreenshotEntry::Path,
                Expr::val(format!("{SCREENSHOT_DIR}\\"))
                    .binary(BinOper::Custom("||"), Expr::col(ScreenshotEntry::Name)),
            )
            .to_owned();

        manager.exec_stmt(backfill).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ScreenshotEntry::Table)
                    .drop_column(ScreenshotEntry::Path)
                    .to_owned(),
            )
            .await
    }
}