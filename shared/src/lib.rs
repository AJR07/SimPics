use chrono::{DateTime, Utc};
use optional_struct::optional_struct;
use serde::{Deserialize, Serialize};

#[optional_struct]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenshotEntry {
    pub name: String,
    pub date: DateTime<Utc>,
    pub path: String
}