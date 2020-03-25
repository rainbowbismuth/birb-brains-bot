use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Output {
    pub error: Option<String>
}