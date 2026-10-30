use crate::db;
use crate::error::Result;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let conn = db::open()?;
        Ok(Self {
            db: Mutex::new(conn),
        })
    }
}
