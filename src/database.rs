use rusqlite::{params, Connection, Result}; 
use crate::comment::{Comment, CommentSend};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE comment (
                id     INTEGER PRIMARY KEY,
                email  TEXT,
                author TEXT,
                text   TEXT NOT NULL
            )",
            params![]
        )?;
        Ok(Self { conn })
    }

    pub fn get_comments(&self) -> Result<Vec<Comment>> {
        self.conn
            .prepare("SELECT author, email, text FROM comment")?
            .query_map([], |row| {
                Ok(Comment {
                    author: row.get(0)?,
                    email: row.get(1)?,
                    text: row.get(2)?,
                })
            })?
            .collect()
    }

    pub fn get_send_comments(&self) -> Result<Vec<CommentSend>> {
        self.conn
            .prepare("SELECT author, email, text FROM comment")?
            .query_map([], |row| {
                Ok(CommentSend {
                    author: row.get(0)?,
                    gravatar: match row.get::<usize, Option<String>>(1)? {
                        Some(email) => Some(format!("{:x}", md5::compute(email.to_lowercase()))),
                        None => None,
                    },
                    text: row.get(2)?,
                })
            })?
            .collect()
    }
    
    pub fn create_comment(&self, comment: &Comment) -> Result<()> {
        self.conn.execute(
            "INSERT INTO comment (author, email, text) VALUES (?1, ?2, ?3)",
            params![&comment.author, &comment.email, &comment.text],
        )?;
        Ok(())
    }
}
