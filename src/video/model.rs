use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub share: Option<Vec<String>>,
}
