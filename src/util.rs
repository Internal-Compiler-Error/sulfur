use derive_more::{Display, Error};
use diesel::{r2d2, Connection, r2d2::CustomizeConnection, SqliteConnection};

#[derive(Debug, Display, Error)]
/// An error type for stuff that never fails.
pub struct Never {}

no_arg_sql_function!(last_insert_rowid, diesel::sql_types::Integer);


#[derive(Debug)]
pub struct EnableForeignKeys;

impl EnableForeignKeys {
    pub fn new() -> Self {
        Self
    }
}

impl CustomizeConnection<SqliteConnection, r2d2::Error> for EnableForeignKeys {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), r2d2::Error> {
        conn.execute("PRAGMA foreign_keys = ON;").expect("Failed to enable foreign keys");
        Ok(())
    }
}
