use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use optional_struct::*;

//? UPDATE update_entry if u change properties here

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ScreenshotEntry")]
#[optional_struct]
#[derive(Serialize, Deserialize)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub name: String,
    pub date: DateTime<Utc>
}

impl ActiveModelBehavior for ActiveModel {}