use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use validator::Validate;

// Master comment type that is stored in database
#[derive(Serialize, Deserialize, Validate)]
pub struct Comment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>, // None is Anonymous
    #[serde(rename(serialize = "gravatar"))]
    #[serde(serialize_with = "serialize_gravatar")]
    #[validate(email)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[validate(length(min = 1))]
    pub text: String,
    #[serde(default)]
    #[serde(with = "ts_seconds_option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

fn serialize_gravatar<S>(email: &Option<String>, s: S) -> Result<S::Ok, S::Error>
where S: Serializer,
{
    match email {              
        Some(email) => s.serialize_some(&format!("{:x}", md5::compute(email.to_lowercase()))),   
        None => s.serialize_none(),
    }
}
