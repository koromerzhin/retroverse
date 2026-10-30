use crate::db;
use crate::error::Result;
use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub cancel_scan: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let conn = db::open()?;
        Ok(Self {
            db: Mutex::new(conn),
            cancel_scan: Arc::new(AtomicBool::new(false)),
        })
    }
}
