use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Comment {
    pub author: Option<String>, // None/null is Anonymous
    pub text: String
}
