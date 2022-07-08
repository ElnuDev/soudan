#[derive(Debug, serde::Serialize)]
pub struct Comment {
    pub author: Option<String>, // null is Anonymous
    pub text: String
}
