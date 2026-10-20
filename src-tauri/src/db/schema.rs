use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ScreenshotEntry")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub name: String,
    pub date: DateTime<Utc>
}

impl ActiveModelBehavior for ActiveModel {}