use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

// Master comment type that is stored in database
pub struct Comment {
    pub author: Option<String>, // None/null is Anonymous
    pub email: Option<String>,
    pub text: String,
    pub timestamp: Option<NaiveDateTime>,
}

impl Comment {
    pub fn send(&self) -> CommentSend {
        CommentSend {
            author: self.author.clone(),
            gravatar: match self.email.clone() {
                Some(email) => Some(format!("{:x}", md5::compute(email.to_lowercase()))),
                None => None,
            },
            text: self.text.clone(),
            timestamp: self.timestamp.unwrap().timestamp(),
        }
    }
}

// Comment type for API responses
#[derive(Serialize)]
pub struct CommentSend {
    pub author: Option<String>,
    pub gravatar: Option<String>,
    pub text: String,
    pub timestamp: i64,
}

// Comment type received containing new comment data
#[derive(Deserialize, Validate)]
pub struct CommentReceive {
    pub author: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub text: String,
}

impl CommentReceive {
    pub fn to_master(&self) -> Comment {
        Comment {
            author: self.author.clone(),
            email: self.email.clone(),
            text: self.text.clone(),
            timestamp: None,
        }
    }
}
