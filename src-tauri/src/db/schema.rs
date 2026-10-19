use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ScreenshotEntry")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub name: String,
    pub date: Date
}

impl ActiveModelBehavior for ActiveModel {}