use rusqlite::{params, Connection, Result}; 
use crate::comment::Comment;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE comment (
                id     INTEGER PRIMARY KEY,
                author TEXT,
                text   TEXT NOT NULL
            )",
            params![]
        )?;
        Ok(Self { conn })
    }

    pub fn get_comments(&self) -> Result<Vec<Comment>> {
        self.conn
            .prepare("SELECT author, text FROM comment")?
            .query_map([], |row| {
                Ok(Comment {
                    author: row.get(0)?,
                    text: row.get(1)?,
                })
            })?
            .collect()
    }

    pub fn create_comment(&self, comment: &Comment) -> Result<()> {
        self.conn.execute(
            "INSERT INTO comment (author, text) VALUES (?1, ?2)",
            params![&comment.author, &comment.text],
        )?;
        Ok(())
    }
}
