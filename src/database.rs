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
                id         INTEGER PRIMARY KEY,
                email      TEXT,
                author     TEXT,
                text       TEXT NOT NULL,
                timestamp  DATETIME DEFAULT CURRENT_TIMESTAMP,
                content_id TEXT NOT NULL,
                parent     INTEGER
            )",
            params![],
        )?;
        Ok(Self { conn })
    }

    pub fn get_comments(&self, content_id: &str) -> Result<Vec<Comment>> {
        self.conn
            .prepare(&format!("SELECT id, author, email, text, timestamp FROM comment WHERE content_id='{content_id}' AND parent IS NULL ORDER BY timestamp DESC"))?
            .query_map([], |row| {
                let id = row.get::<usize, Option<i64>>(0)?.unwrap();
                let replies = self.conn
                    .prepare(&format!("SELECT id, author, email, text, timestamp FROM comment WHERE parent={id} ORDER BY timestamp DESC"))?
                    .query_map([], |row| {
                        Ok(Comment {
                            id: row.get(0)?,
                            author: row.get(1)?,
                            email: row.get(2)?,
                            text: row.get(3)?,
                            timestamp: row.get(4)?,
                            content_id: content_id.to_owned(),
                            parent: Some(id),
                            replies: Vec::new(), // no recursion
                        })
                    })?
                    .collect::<Result<Vec<Comment>>>()?;
                Ok(Comment {
                    id: Some(id),
                    author: row.get(1)?,
                    email: row.get(2)?,
                    text: row.get(3)?,
                    timestamp: row.get(4)?,
                    content_id: content_id.to_owned(),
                    parent: None,
                    replies,
                })
            })?
            .collect()
    }

    pub fn create_comment(&self, comment: &Comment) -> Result<()> {
        self.conn.execute(
            "INSERT INTO comment (author, email, text, content_id) VALUES (?1, ?2, ?3, ?4)",
            params![
                &comment.author,
                &comment.email,
                &comment.text,
                &comment.content_id
            ],
        )?;
        Ok(())
    }
}
