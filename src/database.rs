use crate::Comment;
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

    pub fn get_comments(&self) -> Result<Vec<Comment>> {
        self.conn
            .prepare("SELECT author, email, text, timestamp FROM comment ORDER BY timestamp DESC")?
            .query_map([], |row| {
                Ok(Comment {
                    author: row.get(0)?,
                    email: row.get(1)?,
                    text: row.get(2)?,
                    timestamp: row.get(3)?,
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
