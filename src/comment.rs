#[derive(Debug)]
pub struct Comment {
    pub author: Option<String>, // null is Anonymous
    pub text: String
}
