use crate::comment::{Comment, CommentSend};
use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(testing: bool) -> Result<Self> {
        if !testing {
            unimplemented!("Persistent databases unimplemented!");
        }
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE comment (
                id        INTEGER PRIMARY KEY,
                email     TEXT,
                author    TEXT,
                text      TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        )?;
        Ok(Self { conn })
    }

    pub fn get_send_comments(&self) -> Result<Vec<CommentSend>> {
        self.conn
            .prepare("SELECT author, email, text, timestamp FROM comment")?
            .query_map([], |row| {
                let timestamp: NaiveDateTime = row.get(3)?;
                let timestamp = timestamp.timestamp();
                Ok(CommentSend {
                    author: row.get(0)?,
                    gravatar: match row.get::<usize, Option<String>>(1)? {
                        Some(email) => Some(format!("{:x}", md5::compute(email.to_lowercase()))),
                        None => None,
                    },
                    text: row.get(2)?,
                    timestamp: timestamp,
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
