use rusqlite::{params, Connection, Result};

#[derive(Debug)]
struct Comment {
    author: Option<String>, // null is Anonymous
    text: String
}

struct Database {
    conn: Connection
}

impl Database {
    fn new() -> Result<Self> {
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

    fn get_comments(&self) -> Result<Vec<Comment>> {
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

    fn create_comment(&self, comment: &Comment) -> Result<()> {
        self.conn.execute(
            "INSERT INTO comment (author, text) VALUES (?1, ?2)",
            params![&comment.author, &comment.text],
        )?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let db = Database::new()?;
    let comment = Comment {
        author: Some("Elnu".to_string()), // None for anonymous
        text: "This is a test comment by yours truly!".to_string(),
    };
    db.create_comment(&comment)?;
    for comment in db.get_comments()?.iter() {
        println!("Found comment {:?}", comment);
    }
    Ok(())
}
