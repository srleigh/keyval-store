use rusqlite::{Connection, Result};


pub struct DB{
    conn: Connection,
}

impl DB{
    pub fn new() -> Result<DB>{
        let conn = Connection::open("./db/v1msgs.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS msgs (
                channel TEXT PRIMARY KEY,
                msg TEXT
            )",
            (),
        )?;
        Ok(DB{conn})
    }

    pub fn write(&self, channel: &str, msg: &str) ->Result<()>{
        self.conn.execute(
            "INSERT OR REPLACE INTO msgs (channel, msg) VALUES (?1, ?2)",
            (channel, msg),
        )?;
        Ok(())
    }

    pub fn read(&self, channel: &str) -> Result<String>{
        self.conn.query_row(
            "SELECT msg FROM msgs WHERE channel = ?",
            [channel],
            |row| row.get(0),
        )
    }

    pub fn num_rows(&self) -> Result<i64>{
        self.conn.query_row(
            "SELECT COUNT(*) FROM msgs",
            [],
            |row| row.get(0),
        )
    }

    pub fn database_size(&self) -> Result<i64>{
        let page_count: i64 = self.conn.query_row(
            "PRAGMA PAGE_COUNT",
            [],
            |row| row.get(0),
        )?;
        let page_size: i64 = self.conn.query_row(
            "PRAGMA PAGE_SIZE",
            [],
            |row| row.get(0),
        )?;
        Ok(page_count * page_size)
    }
}
